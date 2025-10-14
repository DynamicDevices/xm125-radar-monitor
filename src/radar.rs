use crate::error::{RadarError, Result};
use crate::i2c::I2cDevice;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// XM125 I2C Register Addresses (based on Acconeer documentation)
const REG_MAIN_COMMAND: u16 = 0x0000;
const REG_MAIN_STATUS: u16 = 0x0002;
const REG_DISTANCE_RESULT: u16 = 0x0100;
const REG_DISTANCE_CONFIG: u16 = 0x0200;
const REG_SENSOR_INFO: u16 = 0x0300;

// Command codes for XM125
const CMD_ENABLE_DETECTOR: u32 = 0x01;
const CMD_DISABLE_DETECTOR: u32 = 0x02;
const CMD_CALIBRATE_DETECTOR: u32 = 0x03;
const CMD_MEASURE_DISTANCE: u32 = 0x04;
const CMD_GET_DETECTOR_STATUS: u32 = 0x05;

// Status flags
const STATUS_DETECTOR_READY: u32 = 0x01;
const STATUS_CALIBRATION_DONE: u32 = 0x02;
const STATUS_MEASUREMENT_READY: u32 = 0x04;
const STATUS_ERROR: u32 = 0x80;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMeasurement {
    pub distance: f32,
    pub strength: f32,
    pub temperature: i16,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct XM125Config {
    pub start_m: f32,
    pub length_m: f32,
    pub max_step_length: u16,
    pub max_profile: u8,
    pub threshold_sensitivity: f32,
}

impl Default for XM125Config {
    fn default() -> Self {
        Self {
            start_m: 0.18,         // 18 cm minimum distance
            length_m: 3.0,         // 3 meter range
            max_step_length: 24,   // Good balance of accuracy/speed
            max_profile: 3,        // Profile 3 for medium range
            threshold_sensitivity: 0.5, // Medium sensitivity
        }
    }
}

pub struct XM125Radar {
    i2c: I2cDevice,
    config: XM125Config,
    is_connected: bool,
    is_calibrated: bool,
    last_calibration: Option<Instant>,
}

impl XM125Radar {
    pub fn new(i2c: I2cDevice) -> Self {
        Self {
            i2c,
            config: XM125Config::default(),
            is_connected: false,
            is_calibrated: false,
            last_calibration: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to XM125 radar module...");

        // Check if device is responsive
        match self.get_status_raw().await {
            Ok(_) => {
                self.is_connected = true;
                info!("Successfully connected to XM125");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to XM125: {}", e);
                Err(RadarError::NotConnected)
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if self.is_connected {
            // Disable detector before disconnecting
            let _ = self.send_command(CMD_DISABLE_DETECTOR).await;
            self.is_connected = false;
            info!("Disconnected from XM125");
        }
        Ok(())
    }

    pub async fn get_status(&mut self) -> Result<String> {
        let status = self.get_status_raw().await?;
        
        let mut status_parts = Vec::new();
        
        if status & STATUS_DETECTOR_READY != 0 {
            status_parts.push("Detector Ready");
        }
        if status & STATUS_CALIBRATION_DONE != 0 {
            status_parts.push("Calibrated");
        }
        if status & STATUS_MEASUREMENT_READY != 0 {
            status_parts.push("Measurement Ready");
        }
        if status & STATUS_ERROR != 0 {
            status_parts.push("ERROR");
        }

        if status_parts.is_empty() {
            status_parts.push("Initializing");
        }

        Ok(format!("Status: {} (0x{:08X})", status_parts.join(", "), status))
    }

    pub async fn get_info(&mut self) -> Result<String> {
        // Read sensor information from XM125
        let info_data = self.i2c.read_register(REG_SENSOR_INFO, 16)?;
        
        // Parse basic sensor information (this would need to match actual XM125 format)
        let sensor_id = u32::from_le_bytes([info_data[0], info_data[1], info_data[2], info_data[3]]);
        let firmware_version = u16::from_le_bytes([info_data[4], info_data[5]]);
        
        Ok(format!(
            "XM125 Radar Module\nSensor ID: 0x{:08X}\nFirmware Version: {}.{}\nConfig: {:.2}m-{:.2}m range",
            sensor_id,
            firmware_version >> 8,
            firmware_version & 0xFF,
            self.config.start_m,
            self.config.start_m + self.config.length_m
        ))
    }

    pub async fn calibrate(&mut self) -> Result<()> {
        info!("Starting XM125 calibration...");

        // Send calibration command
        self.send_command(CMD_CALIBRATE_DETECTOR).await?;

        // Wait for calibration to complete
        let start_time = Instant::now();
        loop {
            let status = self.get_status_raw().await?;
            
            if status & STATUS_CALIBRATION_DONE != 0 {
                self.is_calibrated = true;
                self.last_calibration = Some(Instant::now());
                info!("XM125 calibration completed successfully");
                return Ok(());
            }
            
            if status & STATUS_ERROR != 0 {
                return Err(RadarError::DeviceError {
                    message: "Calibration failed - device error".to_string(),
                });
            }

            if start_time.elapsed() > Duration::from_secs(10) {
                return Err(RadarError::Timeout { timeout: 10 });
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn measure_distance(&mut self) -> Result<DistanceMeasurement> {
        if !self.is_connected {
            return Err(RadarError::NotConnected);
        }

        // Check if calibration is needed (every 5 minutes or if not calibrated)
        if !self.is_calibrated || 
           self.last_calibration.map_or(true, |t| t.elapsed() > Duration::from_secs(300)) {
            self.calibrate().await?;
        }

        // Send measurement command
        self.send_command(CMD_MEASURE_DISTANCE).await?;

        // Wait for measurement to be ready
        let start_time = Instant::now();
        loop {
            let status = self.get_status_raw().await?;
            
            if status & STATUS_MEASUREMENT_READY != 0 {
                break;
            }
            
            if status & STATUS_ERROR != 0 {
                return Err(RadarError::MeasurementFailed(
                    "Device error during measurement".to_string()
                ));
            }

            if start_time.elapsed() > Duration::from_secs(2) {
                return Err(RadarError::Timeout { timeout: 2 });
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Read measurement result
        self.read_distance_result().await
    }

    async fn get_status_raw(&mut self) -> Result<u32> {
        let status_data = self.i2c.read_register(REG_MAIN_STATUS, 4)?;
        Ok(u32::from_le_bytes([
            status_data[0], status_data[1], status_data[2], status_data[3]
        ]))
    }

    async fn send_command(&mut self, command: u32) -> Result<()> {
        debug!("Sending command: 0x{:08X}", command);
        let cmd_bytes = command.to_le_bytes();
        self.i2c.write_register(REG_MAIN_COMMAND, &cmd_bytes)?;
        Ok(())
    }

    async fn read_distance_result(&mut self) -> Result<DistanceMeasurement> {
        // Read distance result (assuming 16 bytes: distance, strength, temp, etc.)
        let result_data = self.i2c.read_register(REG_DISTANCE_RESULT, 16)?;
        
        // Parse the result data (this format would need to match actual XM125 output)
        let distance_mm = u32::from_le_bytes([
            result_data[0], result_data[1], result_data[2], result_data[3]
        ]);
        let strength_raw = u32::from_le_bytes([
            result_data[4], result_data[5], result_data[6], result_data[7]
        ]);
        let temperature = i16::from_le_bytes([result_data[8], result_data[9]]);

        let distance = distance_mm as f32 / 1000.0; // Convert mm to meters
        let strength = (strength_raw as f32) / 100.0; // Convert to dB (assuming 0.01 dB resolution)

        Ok(DistanceMeasurement {
            distance,
            strength,
            temperature,
            timestamp: chrono::Utc::now(),
        })
    }

    pub fn set_config(&mut self, config: XM125Config) {
        self.config = config;
        // Would need to send config to device here
        debug!("Updated XM125 configuration: {:?}", self.config);
    }
}
