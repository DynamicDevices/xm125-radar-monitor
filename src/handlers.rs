//! Command handlers
//!
//! This module contains handlers for various CLI commands including firmware management,
//! GPIO control, and bootloader operations.

use crate::cli::{Cli, FirmwareAction, GpioAction};
use crate::error::RadarError;
use crate::firmware::{self, FirmwareType};
use crate::gpio::XM125GpioController;
use crate::radar::XM125Radar;
use log::info;
use std::process::Command;

/// Handle firmware-related commands
pub async fn handle_firmware_action(
    radar: &mut XM125Radar,
    action: &FirmwareAction,
    firmware_path: &str,
) -> Result<(), RadarError> {
    match action {
        FirmwareAction::Check => {
            let info = radar.get_info()?;
            println!("📦 Current Firmware:");
            println!("{info}");
        }

        FirmwareAction::Update { firmware_type, .. } => {
            let manager =
                firmware::FirmwareManager::new(firmware_path, "/usr/bin/xm125-control.sh", 0x52);
            manager.update_firmware(*firmware_type).await?;
        }

        FirmwareAction::Verify { firmware_type } => {
            if let Some(fw_type) = firmware_type {
                info!("Would verify firmware type: {fw_type:?}");
            } else {
                info!("Would verify current firmware");
            }
        }

        // These are handled earlier in the flow
        FirmwareAction::Checksum { .. }
        | FirmwareAction::Erase { .. }
        | FirmwareAction::Bootloader { .. } => {
            unreachable!("These actions should be handled before I2C initialization");
        }
    }
    Ok(())
}

/// Handle firmware erase command
pub async fn handle_firmware_erase_command(confirm: bool) -> Result<(), RadarError> {
    if !confirm {
        eprintln!("❌ Chip erase requires --confirm flag for safety");
        eprintln!("   This will completely erase all firmware from the XM125 module.");
        eprintln!("   Use: xm125-radar-monitor firmware erase --confirm");
        return Err(RadarError::DeviceError {
            message: "Erase operation not confirmed".to_string(),
        });
    }

    println!("⚠️  WARNING: This will completely erase the XM125 firmware!");
    println!("🔄 Starting chip erase...");

    let output = Command::new("stm32flash")
        .args(["-m", "/dev/i2c-2", "-a", "0x48"])
        .output()
        .map_err(|e| RadarError::DeviceError {
            message: format!("Failed to execute stm32flash: {e}"),
        })?;

    if !output.status.success() {
        return Err(RadarError::DeviceError {
            message: format!(
                "Chip erase failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    println!("✅ Chip erase completed successfully");
    println!("   The XM125 module now needs firmware to be programmed before use.");
    Ok(())
}

/// Handle firmware checksum command
pub fn handle_firmware_checksum_command(
    firmware_type: Option<&FirmwareType>,
    verbose: bool,
    firmware_path: &str,
) -> Result<(), RadarError> {
    if let Some(fw_type) = firmware_type {
        let manager =
            firmware::FirmwareManager::new(firmware_path, "/usr/bin/xm125-control.sh", 0x52);
        let checksum = manager.calculate_binary_checksum(*fw_type)?;
        if verbose {
            println!(
                "Firmware: {} ({})",
                fw_type.display_name(),
                fw_type.binary_filename()
            );
            println!("Path: {}/{}", firmware_path, fw_type.binary_filename());
            println!("MD5: {checksum}");
        } else {
            println!("{}: {}", fw_type.display_name(), checksum);
        }
        Ok(())
    } else {
        for fw_type in [
            firmware::FirmwareType::Distance,
            firmware::FirmwareType::Presence,
        ] {
            let manager =
                firmware::FirmwareManager::new(firmware_path, "/usr/bin/xm125-control.sh", 0x52);
            match manager.calculate_binary_checksum(fw_type) {
                Ok(checksum) => {
                    if verbose {
                        println!(
                            "Firmware: {} ({})",
                            fw_type.display_name(),
                            fw_type.binary_filename()
                        );
                        println!("Path: {}/{}", firmware_path, fw_type.binary_filename());
                        println!("MD5: {checksum}");
                        println!();
                    } else {
                        println!("{}: {}", fw_type.display_name(), checksum);
                    }
                }
                Err(e) => {
                    eprintln!("❌ {}: {e}", fw_type.display_name());
                }
            }
        }
        Ok(())
    }
}

/// Handle bootloader command
pub async fn handle_bootloader_command(cli: &Cli, test_mode: bool) -> Result<(), RadarError> {
    let _gpio_pins = cli.get_gpio_pins();
    let mut gpio_controller = XM125GpioController::new();
    gpio_controller.initialize()?;

    if test_mode {
        println!("🧪 Testing bootloader mode (will reset back to run mode)");
        gpio_controller.reset_to_bootloader_mode()?;

        // Wait a moment for the device to enter bootloader mode
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Reset back to run mode
        gpio_controller.reset_to_run_mode()?;
        println!("✅ Bootloader test completed");
    } else {
        println!("🔄 Putting XM125 into bootloader mode...");
        gpio_controller.reset_to_bootloader_mode()?;
        println!("✅ XM125 is now in bootloader mode (I2C address 0x48)");
        println!("   Ready for firmware programming with stm32flash");
        println!("   Use 'xm125-radar-monitor gpio reset-run' to return to normal mode");
    }
    Ok(())
}

/// Handle GPIO commands
pub fn handle_gpio_command(cli: &Cli, action: &GpioAction) -> Result<(), RadarError> {
    let _gpio_pins = cli.get_gpio_pins();
    let mut gpio_controller = XM125GpioController::new();
    gpio_controller.initialize()?;

    match action {
        GpioAction::Init => {
            gpio_controller.initialize()?;
            println!("✅ GPIO pins initialized successfully");
        }
        GpioAction::Status => {
            gpio_controller.show_gpio_status()?;
        }
        GpioAction::ResetRun => {
            gpio_controller.initialize()?;
            gpio_controller.reset_to_run_mode()?;
            println!("✅ XM125 reset to run mode (I2C address 0x52)");
        }
        GpioAction::ResetBootloader => {
            gpio_controller.initialize()?;
            gpio_controller.reset_to_bootloader_mode()?;
            println!("✅ XM125 reset to bootloader mode (I2C address 0x48)");
        }
        GpioAction::Test => {
            gpio_controller.initialize()?;
            gpio_controller.test_bootloader_control()?;
        }
    }
    Ok(())
}
