// XM125 Radar Module

#![allow(clippy::pedantic)]
// Main interface for XM125 radar functionality with modular design

pub mod debug;
pub mod distance;
pub mod presence;
pub mod registers;

use crate::error::{RadarError, Result};
use crate::gpio::{XM125GpioController, XM125GpioPins};
use crate::i2c::I2cDevice;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::time::Instant;

// Re-export public types
pub use distance::DistanceMeasurement;
pub use presence::{PresenceMeasurement, PresenceRange};
pub use registers::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DetectorMode {
    Distance,
    Presence,
    Combined,
    Breathing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XM125Config {
    pub detector_mode: DetectorMode,
    pub start_m: f32,
    pub length_m: f32,
    pub max_step_length: u32,
    pub max_profile: u32,
    pub threshold_sensitivity: f32,
    // Presence detection configuration
    pub presence_range: PresenceRange,
    pub intra_detection_threshold: f32,
    pub inter_detection_threshold: f32,
    pub frame_rate: f32,
    pub sweeps_per_frame: u32,
    // Connection settings
    pub auto_reconnect: bool,
    pub measurement_interval_ms: u64,
}

impl Default for XM125Config {
    fn default() -> Self {
        Self {
            detector_mode: DetectorMode::Distance,
            start_m: 0.10,  // 10 cm minimum distance
            length_m: 2.90, // 2.90m range (end at 3.0m total)
            max_step_length: 24,
            max_profile: 5,
            threshold_sensitivity: 0.1,
            // Presence detection defaults
            presence_range: PresenceRange::Long,
            intra_detection_threshold: 1.3,
            inter_detection_threshold: 1.0,
            frame_rate: 12.0,
            sweeps_per_frame: 16,
            // Connection settings
            auto_reconnect: true,
            measurement_interval_ms: 1000,
        }
    }
}

pub struct XM125Radar {
    i2c: I2cDevice,
    pub config: XM125Config,
    is_connected: bool,
    is_calibrated: bool,
    last_calibration: Option<Instant>,
    continuous_mode: bool,
    last_measurement: Option<Instant>,
}

impl XM125Radar {
    pub fn new(i2c: I2cDevice) -> Self {
        Self {
            i2c,
            config: XM125Config::default(),
            is_connected: false,
            is_calibrated: false,
            last_calibration: None,
            continuous_mode: false,
            last_measurement: None,
        }
    }

    /// Connect to XM125 radar module with automatic reset if needed
    pub fn connect(&mut self) -> Result<()> {
        info!("Connecting to XM125 radar module...");

        // First, try to connect without any warnings
        match self.get_status_raw() {
            Ok(_) => {
                self.is_connected = true;
                info!("Successfully connected to XM125");
                return Ok(());
            }
            Err(_) => {
                // Device not responding - try to initialize it properly before warning
                debug!("Initial connection failed, attempting hardware initialization...");
            }
        }

        // Try hardware reset to ensure module is in run mode
        if let Err(reset_err) = self.reset_xm125_to_run_mode() {
            debug!("Hardware reset failed: {reset_err}");
        } else {
            // Give module time to initialize after reset
            std::thread::sleep(std::time::Duration::from_millis(1000));

            // Try connection again after reset
            if self.get_status_raw().is_ok() {
                self.is_connected = true;
                info!("Successfully connected to XM125 after hardware initialization");
                return Ok(());
            }
        }

        // Only issue warning after we've tried proper initialization
        warn!("Failed to connect to XM125: I2C communication error after hardware initialization");
        warn!("XM125 not detected on I2C bus - check hardware connections and power");
        Err(RadarError::NotConnected)
    }

    /// Reset XM125 to run mode using internal GPIO control
    fn reset_xm125_to_run_mode(&self) -> Result<()> {
        info!("Executing XM125 reset to run mode using internal GPIO control...");

        // Create GPIO controller with default pins (can be overridden via CLI)
        let pins = XM125GpioPins::default();
        let mut gpio_controller = XM125GpioController::with_pins(pins);

        // Initialize GPIO pins
        gpio_controller
            .initialize()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to initialize GPIO for reset: {}", e),
            })?;

        // Reset to run mode
        gpio_controller
            .reset_to_run_mode()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to reset XM125 to run mode: {}", e),
            })?;

        info!("XM125 hardware reset to run mode completed using internal GPIO");
        Ok(())
    }

    /// Get raw status from device
    fn get_status_raw(&mut self) -> Result<u32> {
        let status_data = self.i2c.read_register(REG_DETECTOR_STATUS, 4)?;
        Ok(u32::from_be_bytes([
            status_data[0],
            status_data[1],
            status_data[2],
            status_data[3],
        ]))
    }

    /// Get formatted status string
    pub fn get_status(&mut self) -> Result<String> {
        let status = self.get_status_raw()?;

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

        Ok(format!(
            "Status: {} (0x{:08X})",
            status_parts.join(", "),
            status
        ))
    }

    /// Get device information
    pub fn get_info(&mut self) -> Result<String> {
        let version_data = self.i2c.read_register(REG_VERSION, 4)?;
        let version = u32::from_be_bytes([
            version_data[0],
            version_data[1],
            version_data[2],
            version_data[3],
        ]);

        let app_id_data = self.i2c.read_register(REG_APPLICATION_ID, 4)?;
        let app_id = u32::from_be_bytes([
            app_id_data[0],
            app_id_data[1],
            app_id_data[2],
            app_id_data[3],
        ]);

        Ok(format!(
            "XM125 Radar Module\nVersion: 0x{:08X}\nApplication ID: 0x{:08X}",
            version, app_id
        ))
    }

    /// Read application ID (for firmware compatibility)
    pub fn read_application_id(&mut self) -> Result<u32> {
        let app_id_data = self.i2c.read_register(REG_APPLICATION_ID, 4)?;
        Ok(u32::from_be_bytes([
            app_id_data[0],
            app_id_data[1],
            app_id_data[2],
            app_id_data[3],
        ]))
    }

    /// Set detector mode
    pub fn set_detector_mode(&mut self, mode: DetectorMode) {
        self.config.detector_mode = mode;
    }

    /// Get detector mode
    pub fn get_detector_mode(&self) -> DetectorMode {
        self.config.detector_mode
    }

    /// Check if radar is connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Configure presence detector
    pub async fn configure_presence_detector(&mut self) -> Result<()> {
        info!("🔧 Configuring presence detector...");

        // Set detector mode to presence
        self.config.detector_mode = DetectorMode::Presence;

        // Create presence detector and configure it
        let mut presence_detector = presence::PresenceDetector::new(&mut self.i2c);

        // Configure range (check for custom range override)
        let custom_start = if self.config.start_m > 0.0 {
            Some(self.config.start_m)
        } else {
            None
        };
        let custom_length = if self.config.length_m > 0.0 {
            Some(self.config.length_m)
        } else {
            None
        };

        let (profile, step_length) = presence_detector.configure_range(
            self.config.presence_range,
            custom_start,
            custom_length,
        )?;
        presence_detector.configure_thresholds(
            self.config.intra_detection_threshold,
            self.config.inter_detection_threshold,
            self.config.frame_rate,
            profile,
            step_length,
        )?;

        // Apply configuration
        presence_detector.apply_configuration().await?;

        info!("✅ Presence detector configured successfully");
        Ok(())
    }

    /// Start presence detector
    pub async fn start_presence_detector(&mut self) -> Result<()> {
        let mut presence_detector = presence::PresenceDetector::new(&mut self.i2c);
        presence_detector.start_detector().await
    }

    /// Stop presence detector
    pub async fn stop_presence_detector(&mut self) -> Result<()> {
        let mut presence_detector = presence::PresenceDetector::new(&mut self.i2c);
        presence_detector.stop_detector().await
    }

    /// Measure presence
    pub async fn measure_presence(&mut self) -> Result<PresenceMeasurement> {
        // Ensure the detector is configured and started
        if self.config.detector_mode != DetectorMode::Presence {
            self.configure_presence_detector().await?;
            self.start_presence_detector().await?;
        }

        let mut presence_detector = presence::PresenceDetector::new(&mut self.i2c);
        presence_detector.measure().await
    }

    /// Configure distance detector
    pub async fn configure_distance_detector(&mut self) -> Result<()> {
        info!("🔧 Configuring distance detector...");

        // Set detector mode to distance
        self.config.detector_mode = DetectorMode::Distance;

        // Create distance detector and configure it
        let mut distance_detector = distance::DistanceDetector::new(&mut self.i2c);

        distance_detector.configure_range(self.config.start_m, self.config.length_m)?;
        distance_detector.configure_detector()?;
        distance_detector.apply_config_and_calibrate().await?;

        self.is_calibrated = true;
        self.last_calibration = Some(Instant::now());

        info!("✅ Distance detector configured successfully");
        Ok(())
    }

    /// Measure distance
    pub async fn measure_distance(&mut self) -> Result<DistanceMeasurement> {
        // Ensure the detector is configured
        if self.config.detector_mode != DetectorMode::Distance || !self.is_calibrated {
            self.configure_distance_detector().await?;
        }

        let mut distance_detector = distance::DistanceDetector::new(&mut self.i2c);
        distance_detector.measure().await
    }

    /// Debug registers
    pub fn debug_registers(&mut self, mode: &str) -> Result<()> {
        let mut debugger = debug::RegisterDebugger::new(&mut self.i2c);
        debugger.debug_all_registers(mode)
    }

    /// Configure presence range (for backward compatibility with main.rs)
    pub fn configure_presence_range(&mut self) -> Result<()> {
        // Create presence detector and configure range
        let mut presence_detector = presence::PresenceDetector::new(&mut self.i2c);

        // Check if custom range was explicitly set (not default values)
        // Default values are start_m=0.10, length_m=2.90 for distance mode
        // For presence mode, we should use the presence_range preset unless explicitly overridden
        let is_custom_range = self.config.start_m != 0.10 || self.config.length_m != 2.90;

        let (custom_start, custom_length) = if is_custom_range {
            (Some(self.config.start_m), Some(self.config.length_m))
        } else {
            (None, None)
        };

        let (profile, step_length) = presence_detector.configure_range(
            self.config.presence_range,
            custom_start,
            custom_length,
        )?;
        presence_detector.configure_thresholds(
            self.config.intra_detection_threshold,
            self.config.inter_detection_threshold,
            self.config.frame_rate,
            profile,
            step_length,
        )?;

        // Calculate final range values for apply_complete_configuration
        let (final_start_mm, final_end_mm) =
            if let (Some(start_m), Some(length_m)) = (custom_start, custom_length) {
                let start_mm = (start_m * 1000.0) as u32;
                let end_mm = ((start_m + length_m) * 1000.0) as u32;
                (start_mm, end_mm)
            } else {
                // Use preset range values
                match self.config.presence_range {
                    presence::PresenceRange::Short => (60u32, 700u32),
                    presence::PresenceRange::Medium => (200u32, 2000u32),
                    presence::PresenceRange::Long => (300u32, 5500u32),
                }
            };

        // Apply complete configuration including range settings
        presence_detector.apply_complete_configuration(final_start_mm, final_end_mm)?;

        Ok(())
    }

    /// Configure distance range from string (e.g., "0.1:3.0")
    pub fn configure_distance_range(&mut self, range_str: &str) -> Result<()> {
        let parts: Vec<&str> = range_str.split(':').collect();
        if parts.len() != 2 {
            return Err(RadarError::DeviceError {
                message: format!(
                    "Invalid range format '{}'. Expected 'start:end' (e.g., '0.1:3.0')",
                    range_str
                ),
            });
        }

        let start_m: f32 = parts[0].parse().map_err(|_| RadarError::DeviceError {
            message: format!("Invalid start distance '{}'", parts[0]),
        })?;

        let end_m: f32 = parts[1].parse().map_err(|_| RadarError::DeviceError {
            message: format!("Invalid end distance '{}'", parts[1]),
        })?;

        if start_m >= end_m {
            return Err(RadarError::DeviceError {
                message: "Start distance must be less than end distance".to_string(),
            });
        }

        self.config.start_m = start_m;
        self.config.length_m = end_m - start_m;

        info!(
            "Distance range configured: {:.3}m to {:.3}m",
            start_m, end_m
        );
        Ok(())
    }
}
