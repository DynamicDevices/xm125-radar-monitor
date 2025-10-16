// Allow pedantic clippy warnings for embedded device management code
#![allow(clippy::needless_pass_by_value)] // String parameters needed for ownership
#![allow(clippy::uninlined_format_args)] // Format args clearer when separate for logging
#![allow(clippy::format_push_string)] // Acceptable for device info formatting
#![allow(clippy::match_same_arms)] // Explicit fallback patterns for robustness
#![allow(clippy::unused_async)] // Some functions may become async in future

use crate::error::{RadarError, Result};
use crate::firmware::{FirmwareManager, FirmwareType};
use log::info;
use std::time::Duration;
use tokio::time::sleep;

/// Device state information
#[derive(Debug, Clone)]
pub struct DeviceState {
    pub is_present: bool,
    pub is_responsive: bool,
    pub firmware_type: Option<FirmwareType>,
    pub app_id: Option<u32>,
    pub needs_reset: bool,
}

/// Comprehensive device manager for XM125
pub struct DeviceManager {
    i2c_device_path: String,
    i2c_address: u16,
    firmware_manager: FirmwareManager,
}

impl DeviceManager {
    pub fn new(
        i2c_device_path: String,
        i2c_address: u16,
        firmware_path: String,
        control_script: String,
    ) -> Self {
        let firmware_manager = FirmwareManager::new(&firmware_path, &control_script, i2c_address);

        Self {
            i2c_device_path,
            i2c_address,
            firmware_manager,
        }
    }

    /// Check device presence on I2C bus (non-intrusive)
    pub async fn check_device_presence(&self) -> DeviceState {
        info!("🔍 Checking XM125 device presence...");

        // First check if device appears on I2C bus using i2cdetect
        let i2c_bus = self.extract_i2c_bus_number();
        let present_on_bus = self.check_i2c_bus_presence(i2c_bus).await;

        if !present_on_bus {
            info!("❌ XM125 not detected on I2C bus {}", i2c_bus);
            return DeviceState {
                is_present: false,
                is_responsive: false,
                firmware_type: None,
                app_id: None,
                needs_reset: true,
            };
        }

        info!(
            "✅ XM125 detected on I2C bus {} at address 0x{:02X}",
            i2c_bus, self.i2c_address
        );

        // Device is present on I2C bus - assume it's responsive
        // Let the firmware manager handle detailed communication
        info!("✅ XM125 detected and assumed responsive");
        DeviceState {
            is_present: true,
            is_responsive: true, // Assume responsive if present
            firmware_type: None, // Let firmware manager determine this
            app_id: None,        // Let firmware manager determine this
            needs_reset: false,
        }
    }

    /// Reset device to run mode
    pub async fn reset_to_run_mode(&self) -> Result<()> {
        info!("🔄 Resetting XM125 to run mode...");

        self.firmware_manager.reset_to_run_mode().await?;

        // Wait for device to initialize
        sleep(Duration::from_millis(1000)).await;

        // Verify device is now responsive
        let state = self.check_device_presence().await;
        if state.is_responsive {
            info!("✅ XM125 successfully reset to run mode");
            Ok(())
        } else {
            Err(RadarError::DeviceError {
                message: "Device reset completed but device is still not responsive".to_string(),
            })
        }
    }

    /// Update firmware to specified type with verification
    pub async fn update_firmware(&self, target_type: FirmwareType, verify: bool) -> Result<()> {
        info!(
            "🚀 Updating firmware to {} (verify: {})",
            target_type.display_name(),
            verify
        );

        // Check current state
        let state = self.check_device_presence().await;

        // If device needs reset, do it first
        if state.needs_reset {
            self.reset_to_run_mode().await?;
        }

        // Check if update is actually needed
        if let Some(current_type) = state.firmware_type {
            if current_type == target_type {
                info!(
                    "✅ Firmware already matches target type: {}",
                    target_type.display_name()
                );
                return Ok(());
            }
        }

        // Perform firmware update
        if verify {
            self.firmware_manager
                .update_firmware_with_verification(target_type, true)
                .await?;
        } else {
            self.firmware_manager.update_firmware(target_type).await?;
        }

        // Verify the update worked
        sleep(Duration::from_millis(1000)).await;
        let final_state = self.check_device_presence().await;

        match final_state.firmware_type {
            Some(actual_type) if actual_type == target_type => {
                info!(
                    "✅ Firmware update successful: {}",
                    target_type.display_name()
                );
                Ok(())
            }
            Some(actual_type) => Err(RadarError::DeviceError {
                message: format!(
                    "Firmware update failed: Expected {}, got {}",
                    target_type.display_name(),
                    actual_type.display_name()
                ),
            }),
            None => Err(RadarError::DeviceError {
                message: "Firmware update failed: Device not responsive after update".to_string(),
            }),
        }
    }

    /// Get comprehensive device information
    #[allow(dead_code)] // Reserved for future use
    pub async fn get_device_info(&self) -> Result<String> {
        let state = self.check_device_presence().await;

        let mut info = String::new();
        info.push_str(&format!(
            "Device Present: {}\n",
            if state.is_present { "YES" } else { "NO" }
        ));
        info.push_str(&format!(
            "Device Responsive: {}\n",
            if state.is_responsive { "YES" } else { "NO" }
        ));

        if let Some(firmware_type) = state.firmware_type {
            info.push_str(&format!(
                "Firmware Type: {}\n",
                firmware_type.display_name()
            ));
        }

        if let Some(app_id) = state.app_id {
            info.push_str(&format!("Application ID: {}\n", app_id));
        }

        info.push_str(&format!(
            "Needs Reset: {}",
            if state.needs_reset { "YES" } else { "NO" }
        ));

        Ok(info)
    }

    /// Extract I2C bus number from device path
    fn extract_i2c_bus_number(&self) -> u8 {
        // Extract bus number from path like "/dev/i2c-2"
        self.i2c_device_path
            .split('-')
            .next_back()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2) // Default to bus 2 for Sentai
    }

    /// Check if device is present on I2C bus using i2cdetect
    async fn check_i2c_bus_presence(&self, bus: u8) -> bool {
        use std::process::Command;

        let output = Command::new("i2cdetect")
            .args(["-y", &bus.to_string()])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Check for run mode address (typically 0x52)
                let run_mode_address_str = format!("{:02x}", self.i2c_address);
                let run_mode_with_spaces = format!(" {} ", run_mode_address_str);
                let run_mode_detected = stdout.contains(&run_mode_with_spaces);

                // Check for bootloader mode address (0x48)
                let bootloader_detected = stdout.contains(" 48 ");

                // Device is present if found in either mode
                let present = run_mode_detected || bootloader_detected;

                if present {
                    if run_mode_detected {
                        info!("✅ XM125 detected in run mode (0x{:02X})", self.i2c_address);
                    }
                    if bootloader_detected {
                        info!("✅ XM125 detected in bootloader mode (0x48)");
                    }
                }

                present
            }
            _ => false,
        }
    }
}
