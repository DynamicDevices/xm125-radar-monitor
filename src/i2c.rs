use crate::error::{RadarError, Result};
use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
use log::{debug, warn};
use std::thread;
use std::time::Duration;

pub struct I2cDevice {
    device: I2cdev,
    address: u16,
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

        Ok(Self { device, address })
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
}
