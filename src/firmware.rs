use crate::error::{RadarError, Result};
use log::{debug, info, warn};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Firmware types supported by XM125
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FirmwareType {
    Distance,
    Presence,
    Breathing,
}

impl FirmwareType {
    /// Get the application ID expected for this firmware type
    pub fn application_id(self) -> u32 {
        match self {
            FirmwareType::Distance => 1,
            FirmwareType::Presence => 2,
            FirmwareType::Breathing => 3,
        }
    }

    /// Get the firmware binary filename
    pub fn binary_filename(self) -> &'static str {
        match self {
            FirmwareType::Distance => "i2c_distance_detector.bin",
            FirmwareType::Presence => "i2c_presence_detector.bin",
            FirmwareType::Breathing => "i2c_ref_app_breathing.bin",
        }
    }

    /// Get human-readable name
    pub fn display_name(self) -> &'static str {
        match self {
            FirmwareType::Distance => "Distance Detector",
            FirmwareType::Presence => "Presence Detector",
            FirmwareType::Breathing => "Breathing Monitor",
        }
    }
}

/// XM125 Firmware Manager
pub struct FirmwareManager {
    firmware_path: String,
    control_script: String,
    i2c_address: u16,
}

impl FirmwareManager {
    /// Create new firmware manager
    pub fn new(firmware_path: &str, control_script: &str, i2c_address: u16) -> Self {
        Self {
            firmware_path: firmware_path.to_string(),
            control_script: control_script.to_string(),
            i2c_address,
        }
    }

    /// Update firmware to the specified type
    pub async fn update_firmware(&self, firmware_type: FirmwareType) -> Result<()> {
        let binary_filename = firmware_type.binary_filename();
        let binary_path = format!("{}/{binary_filename}", self.firmware_path);

        info!(
            "Updating XM125 firmware to {} ({binary_filename})",
            firmware_type.display_name()
        );

        // Verify firmware binary exists
        if !Path::new(&binary_path).exists() {
            return Err(RadarError::DeviceError {
                message: format!("Firmware binary not found: {binary_path}"),
            });
        }

        // Step 1: Put device into bootloader mode
        self.enter_bootloader_mode()?;

        // Step 2: Flash firmware using stm32flash
        self.flash_firmware(&binary_path)?;

        // Step 3: Reset to run mode
        self.reset_to_run_mode()?;

        // Step 4: Verify firmware was flashed correctly
        tokio::time::sleep(Duration::from_millis(2000)).await; // Allow device to initialize
        self.verify_firmware(firmware_type).await?;

        info!(
            "Successfully updated firmware to {} (App ID: {})",
            firmware_type.display_name(),
            firmware_type.application_id()
        );

        Ok(())
    }

    /// Put XM125 into bootloader mode
    fn enter_bootloader_mode(&self) -> Result<()> {
        info!("Entering XM125 bootloader mode...");

        let output = Command::new(&self.control_script)
            .arg("--reset-bootloader")
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to execute control script: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RadarError::DeviceError {
                message: format!("Failed to enter bootloader mode: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("XM125 bootloader mode output: {stdout}");
        Ok(())
    }

    /// Flash firmware using stm32flash
    #[allow(clippy::unused_self)] // Self needed for future enhancements
    fn flash_firmware(&self, binary_path: &str) -> Result<()> {
        info!("Flashing firmware: {binary_path}");

        // Use stm32flash to program the firmware via I2C
        let output = Command::new("stm32flash")
            .args([
                "-w",
                binary_path, // Write binary file
                "-v",        // Verify after write
                "-g",
                "0x08000000", // Jump to application after flashing
                "-a",
                "0x48",       // I2C bus address (bootloader mode)
                "/dev/i2c-2", // I2C device
            ])
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to execute stm32flash: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(RadarError::DeviceError {
                message: format!("Firmware flashing failed:\nSTDOUT: {stdout}\nSTDERR: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("stm32flash output: {stdout}");

        // Check for successful flash indicators
        if stdout.contains("Starting execution at") || stdout.contains("Memory programmed") {
            info!("Firmware flashing completed successfully");
        } else {
            warn!("Firmware flashing may not have completed properly");
        }

        Ok(())
    }

    /// Reset XM125 to run mode
    fn reset_to_run_mode(&self) -> Result<()> {
        info!("Resetting XM125 to run mode...");

        let output = Command::new(&self.control_script)
            .arg("--reset-run")
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to execute control script: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RadarError::DeviceError {
                message: format!("Failed to reset to run mode: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("XM125 run mode output: {stdout}");
        Ok(())
    }

    /// Verify firmware was flashed correctly
    async fn verify_firmware(&self, expected_type: FirmwareType) -> Result<()> {
        info!("Verifying firmware installation...");

        // Give device time to fully initialize
        tokio::time::sleep(Duration::from_millis(3000)).await;

        // Try to read application ID using i2cget
        let output = Command::new("i2cget")
            .args([
                "-y",
                "2",                                    // I2C bus 2
                &format!("0x{:02X}", self.i2c_address), // Device address
                "0xFF",
                "0xFF", // Register address (big-endian)
                "i",    // I2C block read
            ])
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to verify firmware: {e}"),
            })?;

        if !output.status.success() {
            return Err(RadarError::DeviceError {
                message: "Failed to read application ID after firmware update".to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Application ID verification output: {stdout}");

        // Parse the application ID from i2cget output
        let expected_id = expected_type.application_id();
        if stdout.contains(&format!("0x{expected_id:02X}")) {
            info!("Firmware verification successful - Application ID matches");
            Ok(())
        } else {
            Err(RadarError::DeviceError {
                message: format!(
                    "Firmware verification failed - Expected App ID {expected_id}, got: {}",
                    stdout.trim()
                ),
            })
        }
    }

    /// Get MD5 checksum of currently flashed firmware
    pub fn get_firmware_checksum(&self) -> Result<String> {
        info!("Reading firmware checksum...");

        let output = Command::new(&self.control_script)
            .arg("--verify")
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to execute verification: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RadarError::DeviceError {
                message: format!("Firmware verification failed: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract MD5 checksum from output
        for line in stdout.lines() {
            if line.contains("MD5:") {
                if let Some(checksum) = line.split("MD5:").nth(1) {
                    return Ok(checksum.trim().to_string());
                }
            }
        }

        Err(RadarError::DeviceError {
            message: "Could not extract MD5 checksum from verification output".to_string(),
        })
    }

    /// Calculate MD5 checksum of a firmware binary file
    pub fn calculate_binary_checksum(&self, firmware_type: FirmwareType) -> Result<String> {
        let binary_filename = firmware_type.binary_filename();
        let binary_path = format!("{}/{binary_filename}", self.firmware_path);

        let output = Command::new("md5sum")
            .arg(&binary_path)
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to calculate MD5: {e}"),
            })?;

        if !output.status.success() {
            return Err(RadarError::DeviceError {
                message: "Failed to calculate binary MD5 checksum".to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(checksum) = stdout.split_whitespace().next() {
            Ok(checksum.to_string())
        } else {
            Err(RadarError::DeviceError {
                message: "Could not parse MD5 checksum output".to_string(),
            })
        }
    }

    /// Check if firmware update is needed
    #[allow(clippy::unnecessary_wraps)] // May return errors in future versions
    pub fn firmware_update_needed(
        &self,
        current_app_id: u32,
        desired_type: FirmwareType,
    ) -> Result<bool> {
        let expected_id = desired_type.application_id();

        if current_app_id != expected_id {
            info!(
                "Firmware update needed: Current App ID {current_app_id} != Expected {expected_id}"
            );
            return Ok(true);
        }

        // Optionally verify checksum for additional validation
        if let Ok(device_checksum) = self.get_firmware_checksum() {
            if let Ok(binary_checksum) = self.calculate_binary_checksum(desired_type) {
                if device_checksum == binary_checksum {
                    info!("Firmware checksum matches - no update needed");
                    Ok(false)
                } else {
                    info!("Firmware checksum mismatch - update needed\nDevice: {device_checksum}\nBinary: {binary_checksum}");
                    Ok(true)
                }
            } else {
                // If we can't verify checksum, assume no update needed if App ID matches
                debug!("Could not verify binary checksum, assuming firmware is correct");
                Ok(false)
            }
        } else {
            // If we can't read device checksum, assume no update needed if App ID matches
            debug!("Could not read device checksum, assuming firmware is correct");
            Ok(false)
        }
    }
}

impl Default for FirmwareManager {
    fn default() -> Self {
        Self::new("/lib/firmware/acconeer", "/home/fio/xm125-control.sh", 0x52)
    }
}
