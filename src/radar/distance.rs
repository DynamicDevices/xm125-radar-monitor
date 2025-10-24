// Distance Detection Module
// Implements distance detection functionality with proper datasheet compliance

#![allow(clippy::pedantic)]

use crate::error::{RadarError, Result};
use crate::i2c::I2cDevice;
#[allow(clippy::wildcard_imports)]
use super::registers::*;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMeasurement {
    pub distance: f32,
    pub strength: f32,
    pub temperature: i16,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct DistanceDetector<'a> {
    i2c: &'a mut I2cDevice,
}

impl<'a> DistanceDetector<'a> {
    pub fn new(i2c: &'a mut I2cDevice) -> Self {
        Self { i2c }
    }

    /// Configure distance range
    pub fn configure_range(&mut self, start_m: f32, length_m: f32) -> Result<()> {
        let start_mm = (start_m * 1000.0) as u32;
        let end_mm = ((start_m + length_m) * 1000.0) as u32;
        
        info!("Configuring distance range: {:.3}m to {:.3}m", start_m, start_m + length_m);
        
        // Write range configuration to registers
        self.i2c.write_register(REG_START_CONFIG, &start_mm.to_be_bytes())?;
        self.i2c.write_register(REG_END_CONFIG, &end_mm.to_be_bytes())?;
        
        info!("✅ Distance range configured");
        Ok(())
    }

    /// Configure distance detector with default settings
    pub fn configure_detector(&mut self) -> Result<()> {
        info!("🔧 Configuring distance detector with default settings...");
        
        // Write default configuration values
        self.i2c.write_register(REG_MAX_STEP_LENGTH, &DISTANCE_MAX_STEP_LENGTH_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_CLOSE_RANGE_LEAKAGE_CANCELLATION, &DISTANCE_CLOSE_RANGE_LEAKAGE_CANCELLATION_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_SIGNAL_QUALITY, &DISTANCE_SIGNAL_QUALITY_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_MAX_PROFILE, &DISTANCE_MAX_PROFILE_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_THRESHOLD_METHOD, &DISTANCE_THRESHOLD_METHOD_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_PEAK_SORTING, &DISTANCE_PEAK_SORTING_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_NUM_FRAMES_RECORDED_THRESHOLD, &DISTANCE_NUM_FRAMES_RECORDED_THRESHOLD_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_FIXED_AMPLITUDE_THRESHOLD_VALUE, &DISTANCE_FIXED_AMPLITUDE_THRESHOLD_VALUE_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_THRESHOLD_SENSITIVITY, &DISTANCE_THRESHOLD_SENSITIVITY_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_REFLECTOR_SHAPE, &DISTANCE_REFLECTOR_SHAPE_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(REG_FIXED_STRENGTH_THRESHOLD_VALUE, &DISTANCE_FIXED_STRENGTH_THRESHOLD_VALUE_DEFAULT.to_be_bytes())?;
        
        info!("✅ Distance detector configured with default settings");
        Ok(())
    }

    /// Check if distance detector is busy
    pub fn is_busy(&mut self) -> Result<bool> {
        let status = self.i2c.read_register(REG_DETECTOR_STATUS, 4)?;
        let status_value = u32::from_be_bytes([status[0], status[1], status[2], status[3]]);
        Ok((status_value & STATUS_BUSY_MASK) != 0)
    }

    /// Check if distance detector has errors
    pub fn has_errors(&mut self) -> Result<bool> {
        let status = self.i2c.read_register(REG_DETECTOR_STATUS, 4)?;
        let status_value = u32::from_be_bytes([status[0], status[1], status[2], status[3]]);
        Ok((status_value & STATUS_ERROR_MASK) != 0)
    }

    /// Wait for distance detector to not be busy
    pub async fn wait_for_not_busy(&mut self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if !self.is_busy()? {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Err(RadarError::Timeout { timeout: timeout.as_secs() })
    }

    /// Write command safely with busy/error checking
    pub async fn write_command_safe(&mut self, command: u32) -> Result<()> {
        // Check if detector is busy before writing command
        if self.is_busy()? {
            self.wait_for_not_busy(Duration::from_secs(5)).await?;
        }

        // Check for errors - if present, only RESET MODULE command is allowed
        if self.has_errors()? && command != CMD_RESET_MODULE {
            warn!("Distance detector has errors, resetting module before command");
            self.reset_module().await?;
        }

        // Write the command
        self.i2c.write_register(REG_COMMAND, &command.to_be_bytes())?;
        Ok(())
    }

    /// Reset the distance module
    pub async fn reset_module(&mut self) -> Result<()> {
        info!("🔄 Resetting XM125 distance module...");
        
        // RESET MODULE command can always be sent, even when there are errors
        self.i2c.write_register(REG_COMMAND, &CMD_RESET_MODULE.to_be_bytes())?;
        
        // Wait for reset to complete
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        info!("✅ XM125 distance module reset completed");
        Ok(())
    }

    /// Apply configuration and calibrate
    pub async fn apply_config_and_calibrate(&mut self) -> Result<()> {
        info!("Applying distance detector configuration and calibrating...");
        self.write_command_safe(CMD_APPLY_CONFIG_AND_CALIBRATE).await?;
        
        // Wait for configuration and calibration to complete
        self.wait_for_not_busy(CALIBRATION_TIMEOUT).await?;
        
        // Check for configuration errors
        if self.has_errors()? {
            return Err(RadarError::DeviceError {
                message: "Distance detector configuration/calibration failed".to_string(),
            });
        }
        
        info!("✅ Distance detector configured and calibrated successfully");
        Ok(())
    }

    /// Measure distance
    pub async fn measure(&mut self) -> Result<DistanceMeasurement> {
        // Send measure command
        self.write_command_safe(CMD_MEASURE_DISTANCE).await?;
        
        // Wait for measurement to complete
        self.wait_for_not_busy(MEASUREMENT_TIMEOUT).await?;
        
        // Read measurement results
        let distance_result = self.i2c.read_register(REG_DISTANCE_RESULT, 4)?;
        let strength_result = self.i2c.read_register(REG_PEAK0_STRENGTH, 4)?;
        
        // Parse results
        let distance_value = u32::from_be_bytes([distance_result[0], distance_result[1], distance_result[2], distance_result[3]]);
        let strength_value = u32::from_be_bytes([strength_result[0], strength_result[1], strength_result[2], strength_result[3]]);
        
        // Convert distance from mm to meters
        let distance = (distance_value as f32) / 1000.0;
        
        // Convert strength (scaled appropriately)
        let strength = strength_value as f32;
        
        // Mock temperature for now (would need additional register read)
        let temperature = 25i16;

        Ok(DistanceMeasurement {
            distance,
            strength,
            temperature,
            timestamp: chrono::Utc::now(),
        })
    }
}
