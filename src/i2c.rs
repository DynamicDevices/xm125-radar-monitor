use crate::error::{RadarError, Result};
use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
use log::{debug, info, warn};
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

pub struct I2cDevice {
    device: I2cdev,
    address: u16,
    wakeup_pin: Option<u32>,
    int_pin: Option<u32>,
}

impl I2cDevice {
    pub fn new(device_path: &str, address: u16) -> Result<Self> {
        debug!("Opening I2C device {device_path} with address 0x{address:02X}");

        let device = I2cdev::new(device_path).map_err(|e| {
            warn!("Failed to open I2C device {device_path}: {e}");
            RadarError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cannot open I2C device {device_path}: {e}"),
            ))
        })?;

        Ok(Self {
            device,
            address,
            wakeup_pin: None,
            int_pin: None,
        })
    }

    pub fn write_register(&mut self, register: u16, data: &[u8]) -> Result<()> {
        debug!("Writing to register 0x{register:04X}: {data:?}");

        // XM125 register protocol: [reg_high, reg_low, data...]
        let mut buffer = Vec::with_capacity(2 + data.len());
        #[allow(clippy::cast_possible_truncation)] // Register addresses are 16-bit, safe to cast
        {
            buffer.push((register >> 8) as u8); // Register high byte
            buffer.push(register as u8); // Register low byte
        }
        buffer.extend_from_slice(data);

        #[allow(clippy::cast_possible_truncation)] // I2C addresses are 7-bit, safe to cast
        self.device
            .write(self.address as u8, &buffer)
            .map_err(RadarError::I2c)?;

        // Small delay for XM125 processing
        thread::sleep(Duration::from_millis(1));

        Ok(())
    }

    pub fn read_register(&mut self, register: u16, length: usize) -> Result<Vec<u8>> {
        debug!("Reading from register 0x{register:04X}, length: {length}");

        // First, write the register address
        #[allow(clippy::cast_possible_truncation)] // Register addresses are 16-bit, safe to cast
        let reg_bytes = [(register >> 8) as u8, register as u8];
        #[allow(clippy::cast_possible_truncation)] // I2C addresses are 7-bit, safe to cast
        self.device
            .write(self.address as u8, &reg_bytes)
            .map_err(RadarError::I2c)?;

        // Small delay for XM125 processing
        thread::sleep(Duration::from_millis(1));

        // Then read the data
        let mut buffer = vec![0u8; length];
        #[allow(clippy::cast_possible_truncation)] // I2C addresses are 7-bit, safe to cast
        self.device
            .read(self.address as u8, &mut buffer)
            .map_err(RadarError::I2c)?;

        debug!("Read data: {buffer:?}");
        Ok(buffer)
    }

    #[allow(dead_code)] // Reserved for complex command sequences
    pub fn write_read_register(
        &mut self,
        register: u16,
        write_data: &[u8],
        read_length: usize,
    ) -> Result<Vec<u8>> {
        debug!("Write-read register 0x{register:04X}: write {write_data:?}, read length: {read_length}");

        // Write command to register
        self.write_register(register, write_data)?;

        // Wait a bit for processing
        thread::sleep(Duration::from_millis(5));

        // Read response
        self.read_register(register, read_length)
    }

    /// Configure GPIO pins for XM125 hardware control
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    pub fn configure_gpio(&mut self, wakeup_pin: Option<u32>, int_pin: Option<u32>) -> Result<()> {
        if let Some(pin) = wakeup_pin {
            // Check if GPIO is already exported by system control
            if std::path::Path::new(&format!("/sys/class/gpio/gpio{}", pin)).exists() {
                debug!("WAKEUP pin GPIO{} already exported by system", pin);
            } else {
                self.export_gpio(pin)?;
                self.set_gpio_direction(pin, "out")?;
                debug!("Configured WAKEUP pin: GPIO{}", pin);
            }
        }

        if let Some(pin) = int_pin {
            // Check if GPIO is already exported by system control
            if std::path::Path::new(&format!("/sys/class/gpio/gpio{}", pin)).exists() {
                debug!("INT pin GPIO{} already exported by system", pin);
            } else {
                self.export_gpio(pin)?;
                self.set_gpio_direction(pin, "in")?;
                debug!("Configured INT pin: GPIO{}", pin);
            }
        }

        self.wakeup_pin = wakeup_pin;
        self.int_pin = int_pin;
        Ok(())
    }

    /// Wake up the XM125 module using hardware pins
    #[allow(dead_code)] // Reserved for hardware control
    #[allow(clippy::unnecessary_wraps)] // May return errors in future versions
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    pub fn wake_up_module(&self) -> Result<()> {
        let Some(wakeup_pin) = self.wakeup_pin else {
            debug!("No WAKEUP pin configured, assuming hardware is already initialized");
            return Ok(());
        };

        // Check if we can control the GPIO
        if !std::path::Path::new(&format!("/sys/class/gpio/gpio{}", wakeup_pin)).exists() {
            debug!("WAKEUP pin GPIO{} not available for control, assuming hardware is managed externally", wakeup_pin);
            return Ok(());
        }

        info!("Ensuring XM125 module is awake...");

        // Set WAKE UP pin HIGH (if we can write to it)
        if let Err(e) = self.set_gpio_value(wakeup_pin, 1) {
            debug!(
                "Cannot control WAKEUP pin directly: {}, assuming external control",
                e
            );
        } else {
            debug!("Set WAKEUP pin HIGH");
        }

        // Wait for module to be ready (MCU INT pin HIGH)
        if let Some(int_pin) = self.int_pin {
            let timeout = Duration::from_secs(5);
            let start = Instant::now();

            loop {
                match self.read_gpio_value(int_pin) {
                    Ok(1) => {
                        info!("XM125 module is ready (INT pin HIGH)");
                        break;
                    }
                    Ok(0) => {
                        if start.elapsed() > timeout {
                            debug!(
                                "INT pin timeout, but continuing - hardware may be ready anyway"
                            );
                            break;
                        }
                    }
                    Ok(_) => {
                        debug!("Unexpected INT pin value, assuming module is ready");
                        break;
                    }
                    Err(e) => {
                        debug!("Cannot read INT pin: {}, assuming module is ready", e);
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        } else {
            // If no INT pin configured, just wait a reasonable time
            debug!("No INT pin configured, assuming module is ready");
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    /// Put the XM125 module into low power mode
    #[allow(dead_code)] // Reserved for hardware control
    #[allow(clippy::unnecessary_wraps)] // May return errors in future versions
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    pub fn sleep_module(&self) -> Result<()> {
        let Some(wakeup_pin) = self.wakeup_pin else {
            debug!("No WAKEUP pin configured, skipping hardware sleep");
            return Ok(());
        };

        info!("Putting XM125 module to sleep...");

        // Wait for module to be ready first
        if let Some(int_pin) = self.int_pin {
            let timeout = Duration::from_secs(2);
            let start = Instant::now();

            while self.read_gpio_value(int_pin)? == 0 {
                if start.elapsed() > timeout {
                    warn!("Module not ready before sleep, continuing anyway");
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }

        // Set WAKE UP pin LOW
        self.set_gpio_value(wakeup_pin, 0)?;
        debug!("Set WAKEUP pin LOW");

        // Wait for ready signal to go LOW
        if let Some(int_pin) = self.int_pin {
            let timeout = Duration::from_secs(2);
            let start = Instant::now();

            while self.read_gpio_value(int_pin)? == 1 {
                if start.elapsed() > timeout {
                    warn!("INT pin did not go LOW, module may not be sleeping");
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }

            info!("XM125 module is now in low power mode");
        }

        Ok(())
    }

    /// Check if the XM125 module is ready (INT pin HIGH)
    #[allow(dead_code)] // Reserved for hardware status checking
    #[allow(clippy::unnecessary_wraps)] // May return errors in future versions
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    pub fn is_module_ready(&self) -> Result<bool> {
        if let Some(int_pin) = self.int_pin {
            Ok(self.read_gpio_value(int_pin)? == 1)
        } else {
            debug!("No INT pin configured, assuming module is ready");
            Ok(true)
        }
    }

    // GPIO helper functions
    #[allow(clippy::unused_self)] // Self needed for consistency
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    fn export_gpio(&self, pin: u32) -> Result<()> {
        if let Err(e) = std::fs::write("/sys/class/gpio/export", pin.to_string()) {
            // GPIO might already be exported, check if it exists
            if !std::path::Path::new(&format!("/sys/class/gpio/gpio{}", pin)).exists() {
                return Err(RadarError::Io(e));
            }
        }
        Ok(())
    }

    #[allow(clippy::unused_self)] // Self needed for consistency
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    fn set_gpio_direction(&self, pin: u32, direction: &str) -> Result<()> {
        let path = format!("/sys/class/gpio/gpio{}/direction", pin);
        std::fs::write(&path, direction).map_err(RadarError::Io)?;
        Ok(())
    }

    #[allow(dead_code)] // Reserved for hardware control
    #[allow(clippy::unused_self)] // Self needed for consistency
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    fn set_gpio_value(&self, pin: u32, value: u8) -> Result<()> {
        let path = format!("/sys/class/gpio/gpio{}/value", pin);
        std::fs::write(&path, value.to_string()).map_err(RadarError::Io)?;
        Ok(())
    }

    #[allow(clippy::unused_self)] // Self needed for consistency
    #[allow(clippy::uninlined_format_args)] // Allow for GPIO path formatting
    fn read_gpio_value(&self, pin: u32) -> Result<u8> {
        let path = format!("/sys/class/gpio/gpio{}/value", pin);
        let mut content = String::new();
        std::fs::File::open(&path)
            .and_then(|mut f| f.read_to_string(&mut content))
            .map_err(RadarError::Io)?;

        content.trim().parse::<u8>().map_err(|e| {
            RadarError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid GPIO value: {}", e),
            ))
        })
    }
}
