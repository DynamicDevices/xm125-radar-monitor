// SPDX-License-Identifier: GPL-3.0-or-later
//
// XM125 GPIO Control Module
// Copyright (c) 2025 Dynamic Devices Ltd
//
// Internal GPIO control implementation to replace external script dependencies.
// Provides robust, cross-platform GPIO operations for XM125 radar module control.

use crate::error::RadarError;
use log::{debug, info, warn};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// XM125 GPIO pin definitions for i.MX8MM platform
#[derive(Debug, Clone, Copy)]
pub struct XM125GpioPins {
    /// Reset pin - `GPIO4_IO28` (96+28=124) - Active-low reset
    pub reset: u32,
    /// MCU interrupt pin - `GPIO4_IO29` (96+29=125) - Module ready signal
    pub mcu_interrupt: u32,
    /// Wake up pin - `GPIO5_IO11` (128+11=139) - Wake up control
    pub wake_up: u32,
    /// Boot pin - `GPIO5_IO13` (128+13=141) - Bootloader control
    pub boot: u32,
}

impl Default for XM125GpioPins {
    fn default() -> Self {
        Self {
            reset: 124,         // GPIO4_IO28 - SAI3_RXFS
            mcu_interrupt: 125, // GPIO4_IO29 - SAI3_RXC
            wake_up: 139,       // GPIO5_IO11 - ECSPI2_MOSI
            boot: 141,          // GPIO5_IO13 - ECSPI2_SS0
        }
    }
}

/// GPIO direction enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioDirection {
    Input,
    Output,
}

impl std::fmt::Display for GpioDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpioDirection::Input => write!(f, "in"),
            GpioDirection::Output => write!(f, "out"),
        }
    }
}

/// GPIO value enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioValue {
    Low = 0,
    High = 1,
}

impl std::fmt::Display for GpioValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// XM125 GPIO Controller
pub struct XM125GpioController {
    pins: XM125GpioPins,
    initialized: bool,
}

impl XM125GpioController {
    /// Create a new GPIO controller with default pin configuration
    pub fn new() -> Self {
        Self {
            pins: XM125GpioPins::default(),
            initialized: false,
        }
    }

    /// Create a new GPIO controller with custom pin configuration
    pub fn with_pins(pins: XM125GpioPins) -> Self {
        Self {
            pins,
            initialized: false,
        }
    }

    /// Initialize all XM125 GPIO pins
    pub fn initialize(&mut self) -> Result<(), RadarError> {
        info!("🔧 Initializing XM125 GPIO pins...");

        // Fix GPIO141 bootloader pin first (Foundries.io workaround)
        self.fix_gpio141_bootloader_pin()?;

        // Export all GPIOs
        self.export_gpio(self.pins.reset, "Reset")?;
        self.export_gpio(self.pins.mcu_interrupt, "MCU Interrupt")?;
        self.export_gpio(self.pins.wake_up, "Wake Up")?;
        self.export_gpio(self.pins.boot, "Bootloader")?;

        // Set directions
        self.set_gpio_direction(self.pins.reset, GpioDirection::Output, "Reset")?;
        self.set_gpio_direction(
            self.pins.mcu_interrupt,
            GpioDirection::Input,
            "MCU Interrupt",
        )?;
        self.set_gpio_direction(self.pins.wake_up, GpioDirection::Output, "Wake Up")?;
        self.set_gpio_direction(self.pins.boot, GpioDirection::Output, "Bootloader")?;

        self.initialized = true;
        info!("✅ GPIO initialization completed successfully");
        Ok(())
    }

    /// Fix GPIO141 bootloader pin (Foundries.io workaround)
    /// This resolves the SPI controller conflict that prevents GPIO141 access
    fn fix_gpio141_bootloader_pin(&self) -> Result<(), RadarError> {
        info!("🔍 Checking GPIO141 bootloader pin availability...");

        let gpio_path = format!("/sys/class/gpio/gpio{}", self.pins.boot);
        if Path::new(&gpio_path).exists() {
            info!("GPIO141 bootloader pin already available");
            return Ok(());
        }

        // Try simple export first
        if self.try_export_gpio(self.pins.boot).is_ok() {
            info!("✅ GPIO141 bootloader pin exported successfully");
            return Ok(());
        }

        warn!("⚠️  GPIO141 claimed by SPI controller - applying Foundries.io workaround...");

        // Step 1: Unbind SPI devices
        info!("Unbinding SPI devices...");
        for spi_dev in &["spi1.0", "spi3.0"] {
            self.unbind_spi_device(spi_dev)?;
        }

        // Step 2: Unbind SPI controller platform driver
        info!("Unbinding SPI controller platform driver...");
        self.unbind_spi_controller("30830000.spi")?;

        // Step 3: Wait for system to stabilize
        thread::sleep(Duration::from_millis(1000));

        // Step 4: Try to export GPIO141 again
        if self.try_export_gpio(self.pins.boot).is_ok() {
            info!("✅ GPIO141 bootloader pin freed and exported successfully");
            Ok(())
        } else {
            Err(RadarError::DeviceError {
                message: "Failed to free GPIO141 bootloader pin after SPI unbind".to_string(),
            })
        }
    }

    /// Unbind SPI device
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn unbind_spi_device(&self, spi_dev: &str) -> Result<(), RadarError> {
        let device_path = format!("/sys/bus/spi/devices/{spi_dev}");
        let driver_path = format!("/sys/bus/spi/devices/{spi_dev}/driver");

        if !Path::new(&device_path).exists() {
            debug!("SPI device {spi_dev} not found");
            return Ok(());
        }

        if !Path::new(&driver_path).exists() {
            debug!("SPI device {spi_dev} already unbound");
            return Ok(());
        }

        debug!("Unbinding SPI device: {spi_dev}");
        match std::fs::write("/sys/bus/spi/drivers/spidev/unbind", spi_dev) {
            Ok(()) => {
                debug!("Successfully unbound {spi_dev}");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to unbind {spi_dev}: {e} (may already be unbound)");
                Ok(()) // Continue anyway
            }
        }
    }

    /// Unbind SPI controller
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn unbind_spi_controller(&self, controller: &str) -> Result<(), RadarError> {
        let controller_path =
            format!("/sys/devices/platform/soc@0/30800000.bus/30800000.spba-bus/{controller}");
        let driver_path = format!("/sys/bus/platform/drivers/spi_imx/{controller}");

        if !Path::new(&controller_path).exists() {
            debug!("SPI controller {controller} not found or already disabled");
            return Ok(());
        }

        if !Path::new(&driver_path).exists() {
            debug!("SPI controller {controller} already unbound");
            return Ok(());
        }

        debug!("Unbinding SPI controller: {controller}");
        match std::fs::write("/sys/bus/platform/drivers/spi_imx/unbind", controller) {
            Ok(()) => {
                debug!("Successfully unbound SPI controller");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to unbind SPI controller: {e} (may already be unbound)");
                Ok(()) // Continue anyway
            }
        }
    }

    /// Export GPIO if not already exported
    fn export_gpio(&self, gpio_num: u32, gpio_name: &str) -> Result<(), RadarError> {
        let gpio_path = format!("/sys/class/gpio/gpio{gpio_num}");
        if Path::new(&gpio_path).exists() {
            debug!("GPIO{gpio_num} ({gpio_name}) already exported");
            return Ok(());
        }

        info!("📤 Exporting GPIO{gpio_num} ({gpio_name})");
        self.try_export_gpio(gpio_num)
            .map_err(|_| RadarError::DeviceError {
                message: format!("Failed to export GPIO{gpio_num} ({gpio_name})"),
            })?;

        // Wait for GPIO to be available
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    /// Try to export a GPIO pin
    #[allow(clippy::unused_self)]
    fn try_export_gpio(&self, gpio_num: u32) -> Result<(), std::io::Error> {
        std::fs::write("/sys/class/gpio/export", gpio_num.to_string())
    }

    /// Set GPIO direction
    #[allow(clippy::unused_self)]
    fn set_gpio_direction(
        &self,
        gpio_num: u32,
        direction: GpioDirection,
        gpio_name: &str,
    ) -> Result<(), RadarError> {
        let direction_path = format!("/sys/class/gpio/gpio{gpio_num}/direction");
        if !Path::new(&direction_path).exists() {
            return Err(RadarError::DeviceError {
                message: format!(
                    "GPIO{gpio_num} ({gpio_name}) not available for direction setting"
                ),
            });
        }

        debug!("🔄 Setting GPIO{gpio_num} ({gpio_name}) direction to {direction}");
        std::fs::write(&direction_path, direction.to_string()).map_err(|e| {
            RadarError::DeviceError {
                message: format!("Failed to set GPIO{gpio_num} direction: {e}"),
            }
        })?;

        Ok(())
    }

    /// Set GPIO value
    pub fn set_gpio_value(
        &self,
        gpio_num: u32,
        value: GpioValue,
        gpio_name: &str,
    ) -> Result<(), RadarError> {
        if !self.initialized {
            return Err(RadarError::DeviceError {
                message: "GPIO controller not initialized".to_string(),
            });
        }

        let value_path = format!("/sys/class/gpio/gpio{gpio_num}/value");
        if !Path::new(&value_path).exists() {
            return Err(RadarError::DeviceError {
                message: format!("GPIO{gpio_num} ({gpio_name}) not available for value setting"),
            });
        }

        debug!("⚡ Setting GPIO{gpio_num} ({gpio_name}) to {value}");
        std::fs::write(&value_path, value.to_string()).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to set GPIO{gpio_num} value: {e}"),
        })?;

        Ok(())
    }

    /// Get GPIO value
    #[allow(clippy::unused_self)]
    pub fn get_gpio_value(&self, gpio_num: u32) -> Result<GpioValue, RadarError> {
        let value_path = format!("/sys/class/gpio/gpio{gpio_num}/value");
        if !Path::new(&value_path).exists() {
            return Err(RadarError::DeviceError {
                message: format!("GPIO{gpio_num} not available for reading"),
            });
        }

        let mut file = File::open(&value_path).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to open GPIO{gpio_num} value file: {e}"),
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to read GPIO{gpio_num} value: {e}"),
            })?;

        match contents.trim() {
            "0" => Ok(GpioValue::Low),
            "1" => Ok(GpioValue::High),
            _ => Err(RadarError::DeviceError {
                message: format!("Invalid GPIO{gpio_num} value: {}", contents.trim()),
            }),
        }
    }

    /// Reset XM125 module to run mode
    pub fn reset_to_run_mode(&self) -> Result<(), RadarError> {
        info!("🔄 Resetting XM125 to RUN mode...");

        // Set bootloader pin LOW for run mode
        self.set_gpio_value(self.pins.boot, GpioValue::Low, "Bootloader (run mode)")?;

        // Ensure wake pin is HIGH
        self.set_gpio_value(self.pins.wake_up, GpioValue::High, "Wake Up (awake)")?;

        // Small delay for pin to stabilize
        thread::sleep(Duration::from_millis(10));

        // Perform reset sequence
        self.perform_reset_sequence()?;

        info!("✅ Reset to RUN mode completed - ready for normal operation");
        Ok(())
    }

    /// Reset XM125 module to bootloader mode
    pub fn reset_to_bootloader_mode(&self) -> Result<(), RadarError> {
        info!("🔄 Resetting XM125 to BOOTLOADER mode...");

        // Set bootloader pin HIGH for bootloader mode
        self.set_gpio_value(
            self.pins.boot,
            GpioValue::High,
            "Bootloader (bootloader mode)",
        )?;

        // Ensure wake pin is HIGH
        self.set_gpio_value(self.pins.wake_up, GpioValue::High, "Wake Up (awake)")?;

        // Small delay for pin to stabilize
        thread::sleep(Duration::from_millis(10));

        // Perform reset sequence
        self.perform_reset_sequence()?;

        info!("✅ Reset to BOOTLOADER mode completed - ready for firmware programming");
        Ok(())
    }

    /// Perform the actual reset sequence (common for both modes)
    fn perform_reset_sequence(&self) -> Result<(), RadarError> {
        // Assert reset (active-low)
        debug!("Asserting reset (LOW)");
        self.set_gpio_value(self.pins.reset, GpioValue::Low, "Reset (asserted)")?;
        thread::sleep(Duration::from_millis(10)); // 10ms reset assertion (minimum for STM32)

        // Deassert reset
        debug!("Deasserting reset (HIGH)");
        self.set_gpio_value(self.pins.reset, GpioValue::High, "Reset (released)")?;
        thread::sleep(Duration::from_millis(100)); // 100ms for application startup

        // Ensure wake pin is HIGH
        self.set_gpio_value(self.pins.wake_up, GpioValue::High, "Wake Up (awake)")?;
        thread::sleep(Duration::from_millis(100)); // Additional time for wake-up

        Ok(())
    }

    /// Set XM125 to run mode (without reset)
    #[allow(dead_code)] // Public API method
    pub fn set_run_mode(&self) -> Result<(), RadarError> {
        info!("🔧 Setting XM125 to RUN mode (without reset)...");

        // Set bootloader pin LOW for run mode
        self.set_gpio_value(self.pins.boot, GpioValue::Low, "Bootloader (run mode)")?;

        // Ensure wake pin is HIGH
        self.set_gpio_value(self.pins.wake_up, GpioValue::High, "Wake Up (awake)")?;

        info!("✅ XM125 set to RUN mode (BOOT0=LOW)");
        Ok(())
    }

    /// Set XM125 to bootloader mode (without reset)
    #[allow(dead_code)] // Public API method
    pub fn set_bootloader_mode(&self) -> Result<(), RadarError> {
        info!("🔧 Setting XM125 to BOOTLOADER mode (without reset)...");

        // Set bootloader pin HIGH for bootloader mode
        self.set_gpio_value(
            self.pins.boot,
            GpioValue::High,
            "Bootloader (bootloader mode)",
        )?;

        // Ensure wake pin is HIGH
        self.set_gpio_value(self.pins.wake_up, GpioValue::High, "Wake Up (awake)")?;

        info!("✅ XM125 set to BOOTLOADER mode (BOOT0=HIGH)");
        Ok(())
    }

    /// Wait for MCU interrupt to go HIGH (module ready)
    #[allow(dead_code)] // Public API method
    pub fn wait_for_module_ready(&self, timeout_seconds: u32) -> Result<(), RadarError> {
        info!("⏳ Waiting for XM125 to become ready (MCU_INT HIGH)...");

        for count in 0..timeout_seconds {
            match self.get_gpio_value(self.pins.mcu_interrupt) {
                Ok(GpioValue::High) => {
                    info!("✅ XM125 module ready (MCU_INT HIGH)");
                    return Ok(());
                }
                Ok(GpioValue::Low) => {
                    if count % 3 == 0 && count > 0 {
                        debug!("Still waiting for module ready... ({count}/{timeout_seconds}s, MCU_INT=LOW)");
                    }
                }
                Err(e) => {
                    warn!("Failed to read MCU interrupt status: {e}");
                }
            }
            thread::sleep(Duration::from_secs(1));
        }

        Err(RadarError::DeviceError {
            message: format!("Timeout waiting for XM125 ready signal after {timeout_seconds}s"),
        })
    }

    /// Show current GPIO status
    #[allow(clippy::unnecessary_wraps)]
    pub fn show_gpio_status(&self) -> Result<(), RadarError> {
        info!("📊 Current XM125 GPIO Status:");
        println!("==========================");

        let reset_val = self
            .get_gpio_value(self.pins.reset)
            .map_or_else(|_| "?".to_string(), |v| format!("{v}"));
        println!(
            "Reset (GPIO{}):     {} (1=released, 0=asserted)",
            self.pins.reset, reset_val
        );

        let mcu_int_val = self
            .get_gpio_value(self.pins.mcu_interrupt)
            .map_or_else(|_| "?".to_string(), |v| format!("{v}"));
        println!(
            "MCU Int (GPIO{}):    {} (1=ready, 0=not ready)",
            self.pins.mcu_interrupt, mcu_int_val
        );

        let wake_val = self
            .get_gpio_value(self.pins.wake_up)
            .map_or_else(|_| "?".to_string(), |v| format!("{v}"));
        println!(
            "Wake Up (GPIO{}):    {} (1=awake, 0=sleep)",
            self.pins.wake_up, wake_val
        );

        let boot_val = self
            .get_gpio_value(self.pins.boot)
            .map_or_else(|_| "?".to_string(), |v| format!("{v}"));
        println!(
            "Boot Pin (GPIO{}):   {} (1=bootloader, 0=run mode)",
            self.pins.boot, boot_val
        );

        println!();
        Ok(())
    }

    /// Test bootloader control functionality
    pub fn test_bootloader_control(&self) -> Result<(), RadarError> {
        info!("🧪 Testing XM125 bootloader control...");

        if !self.initialized {
            return Err(RadarError::DeviceError {
                message: "GPIO controller not initialized".to_string(),
            });
        }

        info!("Setting bootloader mode (HIGH)...");
        self.set_gpio_value(self.pins.boot, GpioValue::High, "Bootloader (test)")?;
        thread::sleep(Duration::from_millis(500));

        info!("Setting run mode (LOW)...");
        self.set_gpio_value(self.pins.boot, GpioValue::Low, "Bootloader (test)")?;
        thread::sleep(Duration::from_millis(500));

        info!("✅ Bootloader control test completed successfully");
        Ok(())
    }

    /// Check if GPIO controller is initialized
    #[allow(dead_code)] // Public API method
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get pin configuration
    #[allow(dead_code)] // Public API method
    pub fn pins(&self) -> &XM125GpioPins {
        &self.pins
    }
}

impl Default for XM125GpioController {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for XM125GpioController {
    fn drop(&mut self) {
        // Optionally unexport GPIOs on drop
        // This is usually not necessary as the kernel handles cleanup
        debug!("XM125GpioController dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_pins_default() {
        let pins = XM125GpioPins::default();
        assert_eq!(pins.reset, 124);
        assert_eq!(pins.mcu_interrupt, 125);
        assert_eq!(pins.wake_up, 139);
        assert_eq!(pins.boot, 141);
    }

    #[test]
    fn test_gpio_direction_display() {
        assert_eq!(GpioDirection::Input.to_string(), "in");
        assert_eq!(GpioDirection::Output.to_string(), "out");
    }

    #[test]
    fn test_gpio_value_display() {
        assert_eq!(GpioValue::Low.to_string(), "0");
        assert_eq!(GpioValue::High.to_string(), "1");
    }

    #[test]
    fn test_gpio_controller_creation() {
        let controller = XM125GpioController::new();
        assert!(!controller.is_initialized());
        assert_eq!(controller.pins().reset, 124);
    }
}
