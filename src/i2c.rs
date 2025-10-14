use crate::error::{RadarError, Result};
use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use log::{debug, warn};
use std::thread;
use std::time::Duration;

pub struct I2cDevice {
    device: I2cdev,
    address: u16,
}

impl I2cDevice {
    pub fn new(device_path: &str, address: u16) -> Result<Self> {
        debug!("Opening I2C device {} with address 0x{:02X}", device_path, address);
        
        let device = I2cdev::new(device_path)
            .map_err(|e| {
                warn!("Failed to open I2C device {}: {}", device_path, e);
                RadarError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Cannot open I2C device {}: {}", device_path, e)
                ))
            })?;

        Ok(Self { device, address })
    }

    pub fn write_register(&mut self, register: u16, data: &[u8]) -> Result<()> {
        debug!("Writing to register 0x{:04X}: {:?}", register, data);
        
        // XM125 register protocol: [reg_high, reg_low, data...]
        let mut buffer = Vec::with_capacity(2 + data.len());
        buffer.push((register >> 8) as u8);  // Register high byte
        buffer.push(register as u8);         // Register low byte
        buffer.extend_from_slice(data);

        self.device.write(self.address as u8, &buffer)
            .map_err(|e| RadarError::I2c(e))?;

        // Small delay for XM125 processing
        thread::sleep(Duration::from_millis(1));
        
        Ok(())
    }

    pub fn read_register(&mut self, register: u16, length: usize) -> Result<Vec<u8>> {
        debug!("Reading from register 0x{:04X}, length: {}", register, length);
        
        // First, write the register address
        let reg_bytes = [(register >> 8) as u8, register as u8];
        self.device.write(self.address as u8, &reg_bytes)
            .map_err(|e| RadarError::I2c(e))?;

        // Small delay for XM125 processing
        thread::sleep(Duration::from_millis(1));

        // Then read the data
        let mut buffer = vec![0u8; length];
        self.device.read(self.address as u8, &mut buffer)
            .map_err(|e| RadarError::I2c(e))?;

        debug!("Read data: {:?}", buffer);
        Ok(buffer)
    }

    pub fn write_read_register(&mut self, register: u16, write_data: &[u8], read_length: usize) -> Result<Vec<u8>> {
        debug!("Write-read register 0x{:04X}: write {:?}, read length: {}", register, write_data, read_length);
        
        // Write command to register
        self.write_register(register, write_data)?;
        
        // Wait a bit for processing
        thread::sleep(Duration::from_millis(5));
        
        // Read response
        self.read_register(register, read_length)
    }
}
