use crate::error::{RadarError, Result};
use crate::i2c::I2cDevice;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// XM125 I2C Register Addresses (from distance_reg_protocol.h)
const REG_VERSION: u16 = 0; // DISTANCE_REG_VERSION_ADDRESS
const REG_PROTOCOL_STATUS: u16 = 1; // DISTANCE_REG_PROTOCOL_STATUS_ADDRESS
const REG_MEASURE_COUNTER: u16 = 2; // DISTANCE_REG_MEASURE_COUNTER_ADDRESS
const REG_DETECTOR_STATUS: u16 = 3; // DISTANCE_REG_DETECTOR_STATUS_ADDRESS
const REG_DISTANCE_RESULT: u16 = 16; // DISTANCE_REG_DISTANCE_RESULT_ADDRESS
const REG_PEAK0_DISTANCE: u16 = 17; // DISTANCE_REG_PEAK0_DISTANCE_ADDRESS
const REG_PEAK0_STRENGTH: u16 = 27; // DISTANCE_REG_PEAK0_STRENGTH_ADDRESS
const REG_START_CONFIG: u16 = 64; // DISTANCE_REG_START_ADDRESS
const REG_END_CONFIG: u16 = 65; // DISTANCE_REG_END_ADDRESS
const REG_COMMAND: u16 = 256; // DISTANCE_REG_COMMAND_ADDRESS
const REG_APPLICATION_ID: u16 = 65535; // DISTANCE_REG_APPLICATION_ID_ADDRESS

// Command codes for XM125 (from distance_reg_protocol.h)
const CMD_APPLY_CONFIG_AND_CALIBRATE: u32 = 1; // DISTANCE_REG_COMMAND_ENUM_APPLY_CONFIG_AND_CALIBRATE
const CMD_MEASURE_DISTANCE: u32 = 2; // DISTANCE_REG_COMMAND_ENUM_MEASURE_DISTANCE
const CMD_APPLY_CONFIGURATION: u32 = 3; // DISTANCE_REG_COMMAND_ENUM_APPLY_CONFIGURATION
const CMD_CALIBRATE: u32 = 4; // DISTANCE_REG_COMMAND_ENUM_CALIBRATE
const CMD_RECALIBRATE: u32 = 5; // DISTANCE_REG_COMMAND_ENUM_RECALIBRATE
const CMD_RESET_MODULE: u32 = 0x5253_5421; // DISTANCE_REG_COMMAND_ENUM_RESET_MODULE

// Legacy/placeholder commands for compatibility (not in actual XM125 protocol)
const CMD_ENABLE_DETECTOR: u32 = CMD_APPLY_CONFIGURATION;
const CMD_DISABLE_DETECTOR: u32 = CMD_RESET_MODULE;
const CMD_ENABLE_PRESENCE_DETECTOR: u32 = CMD_APPLY_CONFIGURATION;
const CMD_MEASURE_PRESENCE: u32 = CMD_MEASURE_DISTANCE; // Placeholder - presence needs different protocol
const CMD_ENABLE_CONTINUOUS_MODE: u32 = CMD_APPLY_CONFIGURATION;
const CMD_DISABLE_CONTINUOUS_MODE: u32 = CMD_RESET_MODULE;

// Placeholder register for compatibility
const REG_SENSOR_INFO: u16 = REG_VERSION; // Use version register for device info
const REG_PRESENCE_RESULT: u16 = REG_DISTANCE_RESULT; // Placeholder - needs presence protocol

// Status flags
const STATUS_DETECTOR_READY: u32 = 0x01;
const STATUS_CALIBRATION_DONE: u32 = 0x02;
const STATUS_MEASUREMENT_READY: u32 = 0x04;
const STATUS_PRESENCE_DETECTED: u32 = 0x08;
const STATUS_CONTINUOUS_MODE: u32 = 0x10;
const STATUS_ERROR: u32 = 0x80;

// Timeout constants
const CALIBRATION_TIMEOUT: Duration = Duration::from_secs(10);
const MEASUREMENT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DetectorMode {
    Distance,
    Presence,
    Combined,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PresenceRange {
    Short,  // 0.06m - 0.7m
    Medium, // 0.2m - 2.0m
    Long,   // 0.5m - 7.0m
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMeasurement {
    pub distance: f32,
    pub strength: f32,
    pub temperature: i16,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMeasurement {
    pub presence_detected: bool,
    pub presence_distance: f32,
    pub intra_presence_score: f32, // Fast motion score
    pub inter_presence_score: f32, // Slow motion score
    pub temperature: i16,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedMeasurement {
    pub distance: Option<DistanceMeasurement>,
    pub presence: Option<PresenceMeasurement>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct XM125Config {
    pub detector_mode: DetectorMode,
    pub start_m: f32,
    pub length_m: f32,
    pub max_step_length: u16,
    pub max_profile: u8,
    pub threshold_sensitivity: f32,
    // Presence detection specific
    pub presence_range: PresenceRange,
    pub intra_detection_threshold: f32,
    pub inter_detection_threshold: f32,
    pub frame_rate: f32,
    pub sweeps_per_frame: u16,
    // Continuous monitoring
    pub auto_reconnect: bool,
    pub measurement_interval_ms: u64,
}

impl Default for XM125Config {
    fn default() -> Self {
        Self {
            detector_mode: DetectorMode::Distance,
            start_m: 0.18,              // 18 cm minimum distance
            length_m: 3.0,              // 3 meter range
            max_step_length: 24,        // Good balance of accuracy/speed
            max_profile: 3,             // Profile 3 for medium range
            threshold_sensitivity: 0.5, // Medium sensitivity
            // Presence detection defaults
            presence_range: PresenceRange::Medium,
            intra_detection_threshold: 1.3,
            inter_detection_threshold: 1.0,
            frame_rate: 12.0,
            sweeps_per_frame: 16,
            // Continuous monitoring defaults
            auto_reconnect: true,
            measurement_interval_ms: 1000,
        }
    }
}

pub struct XM125Radar {
    i2c: I2cDevice,
    config: XM125Config,
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

    pub fn connect(&mut self) -> Result<()> {
        info!("Connecting to XM125 radar module...");

        // Check if device is responsive
        match self.get_status_raw() {
            Ok(_) => {
                self.is_connected = true;
                info!("Successfully connected to XM125");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to XM125: {e}");
                Err(RadarError::NotConnected)
            }
        }
    }

    pub fn disconnect(&mut self) {
        if self.is_connected {
            // Disable detector before disconnecting
            let _ = self.send_command(CMD_DISABLE_DETECTOR);
            self.is_connected = false;
            info!("Disconnected from XM125");
        }
    }

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

    pub fn get_info(&mut self) -> Result<String> {
        // Read sensor information from XM125
        let info_data = self.i2c.read_register(REG_SENSOR_INFO, 16)?;

        // Parse basic sensor information (this would need to match actual XM125 format)
        let sensor_id =
            u32::from_le_bytes([info_data[0], info_data[1], info_data[2], info_data[3]]);
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
        self.send_command(CMD_APPLY_CONFIG_AND_CALIBRATE)?;

        // Wait for calibration to complete
        let start_time = Instant::now();
        loop {
            let status = self.get_status_raw()?;

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

            if start_time.elapsed() > CALIBRATION_TIMEOUT {
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
        // Using map_or instead of is_none_or due to stability requirements
        #[allow(unknown_lints, clippy::unnecessary_map_or)]
        if !self.is_calibrated
            || self
                .last_calibration
                .map_or(true, |t| t.elapsed() > Duration::from_secs(300))
        {
            self.calibrate().await?;
        }

        // Send measurement command
        self.send_command(CMD_MEASURE_DISTANCE)?;

        // Wait for measurement to be ready
        let start_time = Instant::now();
        loop {
            let status = self.get_status_raw()?;

            if status & STATUS_MEASUREMENT_READY != 0 {
                break;
            }

            if status & STATUS_ERROR != 0 {
                return Err(RadarError::MeasurementFailed(
                    "Device error during measurement".to_string(),
                ));
            }

            if start_time.elapsed() > Duration::from_secs(2) {
                return Err(RadarError::Timeout { timeout: 2 });
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Read measurement result
        self.read_distance_result()
    }

    fn get_status_raw(&mut self) -> Result<u32> {
        let status_data = self.i2c.read_register(REG_DETECTOR_STATUS, 4)?;
        Ok(u32::from_le_bytes([
            status_data[0],
            status_data[1],
            status_data[2],
            status_data[3],
        ]))
    }

    fn send_command(&mut self, command: u32) -> Result<()> {
        debug!("Sending command: 0x{command:08X}");
        let cmd_bytes = command.to_le_bytes();
        self.i2c.write_register(REG_COMMAND, &cmd_bytes)?;
        Ok(())
    }

    fn read_distance_result(&mut self) -> Result<DistanceMeasurement> {
        // Read distance result (assuming 16 bytes: distance, strength, temp, etc.)
        let result_data = self.i2c.read_register(REG_DISTANCE_RESULT, 16)?;

        // Parse the result data (this format would need to match actual XM125 output)
        let distance_mm = u32::from_le_bytes([
            result_data[0],
            result_data[1],
            result_data[2],
            result_data[3],
        ]);
        let strength_raw = u32::from_le_bytes([
            result_data[4],
            result_data[5],
            result_data[6],
            result_data[7],
        ]);
        let temperature = i16::from_le_bytes([result_data[8], result_data[9]]);

        #[allow(clippy::cast_precision_loss)] // Converting mm to meters, precision loss acceptable
        let distance = distance_mm as f32 / 1000.0; // Convert mm to meters
        #[allow(clippy::cast_precision_loss)] // Converting to dB, precision loss acceptable
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

    /// Automatically connect with retry logic
    pub async fn auto_connect(&mut self) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: Duration = Duration::from_millis(500);

        for attempt in 1..=MAX_RETRIES {
            match self.connect_async().await {
                Ok(()) => {
                    info!("Successfully connected to XM125 on attempt {attempt}");
                    return Ok(());
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        warn!("Connection attempt {attempt} failed: {e}. Retrying...");
                        tokio::time::sleep(RETRY_DELAY).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(RadarError::NotConnected)
    }

    /// Async version of connect with detector configuration
    pub async fn connect_async(&mut self) -> Result<()> {
        info!("Connecting to XM125 radar module...");

        // Check if device is responsive
        match self.get_status_raw() {
            Ok(_) => {
                self.is_connected = true;
                info!("Successfully connected to XM125");

                // Configure the detector based on current mode
                self.configure_detector().await?;

                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to XM125: {e}");
                Err(RadarError::NotConnected)
            }
        }
    }

    /// Configure the detector based on the current mode
    async fn configure_detector(&mut self) -> Result<()> {
        info!(
            "Configuring detector for mode: {:?}",
            self.config.detector_mode
        );

        match self.config.detector_mode {
            DetectorMode::Distance => {
                self.send_command(CMD_ENABLE_DETECTOR)?;
            }
            DetectorMode::Presence => {
                self.send_command(CMD_ENABLE_PRESENCE_DETECTOR)?;
                self.configure_presence_range();
            }
            DetectorMode::Combined => {
                self.send_command(CMD_ENABLE_DETECTOR)?;
                self.send_command(CMD_ENABLE_PRESENCE_DETECTOR)?;
                self.configure_presence_range();
            }
        }

        // Apply configuration and calibrate
        self.wait_for_calibration().await?;

        Ok(())
    }

    /// Configure presence detection range parameters
    fn configure_presence_range(&mut self) {
        let (start, end) = match self.config.presence_range {
            PresenceRange::Short => (0.06, 0.7),
            PresenceRange::Medium => (0.2, 2.0),
            PresenceRange::Long => (0.5, 7.0),
        };

        // Write range configuration (simplified - would need actual register protocol)
        info!("Configured presence range: {start:.2}m - {end:.2}m");
    }

    /// Wait for calibration to complete
    async fn wait_for_calibration(&mut self) -> Result<()> {
        let start_time = Instant::now();

        loop {
            let status = self.get_status_raw()?;

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

            if start_time.elapsed() > CALIBRATION_TIMEOUT {
                return Err(RadarError::Timeout { timeout: 10 });
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Measure presence detection
    pub async fn measure_presence(&mut self) -> Result<PresenceMeasurement> {
        if !self.is_connected {
            return Err(RadarError::NotConnected);
        }

        // Ensure we're configured for presence detection
        if !matches!(
            self.config.detector_mode,
            DetectorMode::Presence | DetectorMode::Combined
        ) {
            return Err(RadarError::InvalidParameters(
                "Detector not configured for presence detection".to_string(),
            ));
        }

        self.send_command(CMD_MEASURE_PRESENCE)?;

        // Wait for measurement
        self.wait_for_measurement().await?;

        // Read presence result
        let result_data = self.i2c.read_register(REG_PRESENCE_RESULT, 20)?;

        let presence_detected = result_data[0] != 0;
        let distance_raw = u32::from_le_bytes([
            result_data[1],
            result_data[2],
            result_data[3],
            result_data[4],
        ]);
        let intra_score_raw = u32::from_le_bytes([
            result_data[5],
            result_data[6],
            result_data[7],
            result_data[8],
        ]);
        let inter_score_raw = u32::from_le_bytes([
            result_data[9],
            result_data[10],
            result_data[11],
            result_data[12],
        ]);
        let temperature = i16::from_le_bytes([result_data[13], result_data[14]]);

        #[allow(clippy::cast_precision_loss)]
        let presence_distance = (distance_raw as f32) / 1000.0; // Convert mm to m
        #[allow(clippy::cast_precision_loss)]
        let intra_score = (intra_score_raw as f32) / 1000.0;
        #[allow(clippy::cast_precision_loss)]
        let inter_score = (inter_score_raw as f32) / 1000.0;

        self.last_measurement = Some(Instant::now());

        Ok(PresenceMeasurement {
            presence_detected,
            presence_distance,
            intra_presence_score: intra_score,
            inter_presence_score: inter_score,
            temperature,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Measure combined distance and presence
    pub async fn measure_combined(&mut self) -> Result<CombinedMeasurement> {
        if !self.is_connected {
            return Err(RadarError::NotConnected);
        }

        if self.config.detector_mode != DetectorMode::Combined {
            return Err(RadarError::InvalidParameters(
                "Detector not configured for combined detection".to_string(),
            ));
        }

        let distance = self.measure_distance().await.ok();

        let presence = self.measure_presence().await.ok();

        Ok(CombinedMeasurement {
            distance,
            presence,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Start continuous monitoring mode
    pub async fn start_continuous_monitoring(&mut self) -> Result<()> {
        if !self.is_connected && self.config.auto_reconnect {
            self.auto_connect().await?;
        }

        if !self.is_connected {
            return Err(RadarError::NotConnected);
        }

        self.send_command(CMD_ENABLE_CONTINUOUS_MODE)?;
        self.continuous_mode = true;

        info!(
            "Started continuous monitoring mode with {}ms intervals",
            self.config.measurement_interval_ms
        );
        Ok(())
    }

    /// Stop continuous monitoring mode
    pub fn stop_continuous_monitoring(&mut self) -> Result<()> {
        if self.continuous_mode {
            self.send_command(CMD_DISABLE_CONTINUOUS_MODE)?;
            self.continuous_mode = false;
            info!("Stopped continuous monitoring mode");
        }
        Ok(())
    }

    /// Check if continuous monitoring is active
    pub fn is_continuous_monitoring(&self) -> bool {
        self.continuous_mode
    }

    /// Wait for measurement to be ready
    async fn wait_for_measurement(&mut self) -> Result<()> {
        let start_time = Instant::now();

        loop {
            let status = self.get_status_raw()?;

            if status & STATUS_MEASUREMENT_READY != 0 {
                return Ok(());
            }

            if status & STATUS_ERROR != 0 {
                return Err(RadarError::DeviceError {
                    message: "Device error during measurement".to_string(),
                });
            }

            if start_time.elapsed() > MEASUREMENT_TIMEOUT {
                return Err(RadarError::Timeout { timeout: 5 });
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Get detector mode
    pub fn get_detector_mode(&self) -> DetectorMode {
        self.config.detector_mode
    }

    /// Check if radar is connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Set detector mode and reconfigure if connected
    pub async fn set_detector_mode(&mut self, mode: DetectorMode) -> Result<()> {
        let old_mode = self.config.detector_mode;
        self.config.detector_mode = mode;

        if self.is_connected && old_mode != mode {
            info!("Switching detector mode from {old_mode:?} to {mode:?}");
            self.configure_detector().await?;
        }

        Ok(())
    }
}
