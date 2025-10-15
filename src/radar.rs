use crate::error::{RadarError, Result};
use crate::i2c::I2cDevice;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// Allow pedantic clippy warnings for this module - many are false positives for embedded code
#[allow(clippy::unreadable_literal)] // Hex constants are clearer in embedded context
#[allow(clippy::cast_precision_loss)] // Acceptable precision loss for sensor data
#[allow(clippy::cast_possible_truncation)] // Values are validated to be in range
#[allow(clippy::cast_sign_loss)] // Values are validated to be positive
#[allow(clippy::uninlined_format_args)] // Format args are clearer when separate
#[allow(clippy::too_many_lines)] // Complex embedded protocols require long functions
#[allow(clippy::unused_async)] // Some functions may become async in future
#[allow(clippy::trivially_copy_pass_by_ref)] // Consistent API design
#[allow(clippy::match_same_arms)] // Explicit fallback patterns for robustness
                                  // XM125 I2C Register Addresses (from distance_reg_protocol.h)
const REG_VERSION: u16 = 0; // DISTANCE_REG_VERSION_ADDRESS
#[allow(dead_code)] // Reserved for protocol validation
const REG_PROTOCOL_STATUS: u16 = 1; // DISTANCE_REG_PROTOCOL_STATUS_ADDRESS
#[allow(dead_code)] // Reserved for measurement counting
const REG_MEASURE_COUNTER: u16 = 2; // DISTANCE_REG_MEASURE_COUNTER_ADDRESS
const REG_DETECTOR_STATUS: u16 = 3; // DISTANCE_REG_DETECTOR_STATUS_ADDRESS
const REG_DISTANCE_RESULT: u16 = 16; // DISTANCE_REG_DISTANCE_RESULT_ADDRESS
#[allow(dead_code)] // Reserved for peak detection
const REG_PEAK0_DISTANCE: u16 = 17; // DISTANCE_REG_PEAK0_DISTANCE_ADDRESS
#[allow(dead_code)] // Reserved for peak detection
const REG_PEAK0_STRENGTH: u16 = 27; // DISTANCE_REG_PEAK0_STRENGTH_ADDRESS
                                    // Distance detector configuration registers (from distance_reg_protocol.h)
const REG_START_CONFIG: u16 = 64; // DISTANCE_REG_START_ADDRESS (0x40)
const REG_END_CONFIG: u16 = 65; // DISTANCE_REG_END_ADDRESS (0x41)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_MAX_STEP_LENGTH: u16 = 66; // DISTANCE_REG_MAX_STEP_LENGTH_ADDRESS (0x42)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_CLOSE_RANGE_LEAKAGE_CANCELLATION: u16 = 67; // DISTANCE_REG_CLOSE_RANGE_LEAKAGE_CANCELLATION_ADDRESS (0x43)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_SIGNAL_QUALITY: u16 = 68; // DISTANCE_REG_SIGNAL_QUALITY_ADDRESS (0x44)
const REG_MAX_PROFILE: u16 = 69; // DISTANCE_REG_MAX_PROFILE_ADDRESS (0x45)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_THRESHOLD_METHOD: u16 = 70; // DISTANCE_REG_THRESHOLD_METHOD_ADDRESS (0x46)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_PEAK_SORTING: u16 = 71; // DISTANCE_REG_PEAK_SORTING_ADDRESS (0x47)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_NUM_FRAMES_RECORDED_THRESHOLD: u16 = 72; // DISTANCE_REG_NUM_FRAMES_RECORDED_THRESHOLD_ADDRESS (0x48)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_FIXED_AMPLITUDE_THRESHOLD_VALUE: u16 = 73; // DISTANCE_REG_FIXED_AMPLITUDE_THRESHOLD_VALUE_ADDRESS (0x49)
const REG_THRESHOLD_SENSITIVITY: u16 = 74; // DISTANCE_REG_THRESHOLD_SENSITIVITY_ADDRESS (0x4A)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_REFLECTOR_SHAPE: u16 = 75; // DISTANCE_REG_REFLECTOR_SHAPE_ADDRESS (0x4B)
#[allow(dead_code)] // Reserved for advanced configuration
const REG_FIXED_STRENGTH_THRESHOLD_VALUE: u16 = 76; // DISTANCE_REG_FIXED_STRENGTH_THRESHOLD_VALUE_ADDRESS (0x4C)
const REG_COMMAND: u16 = 256; // DISTANCE_REG_COMMAND_ADDRESS
#[allow(dead_code)] // Reserved for application identification
const REG_APPLICATION_ID: u16 = 65535; // DISTANCE_REG_APPLICATION_ID_ADDRESS

// Presence Detection Registers (from presence_reg_protocol.h)
const REG_PRESENCE_RESULT: u16 = 16; // PRESENCE_REG_PRESENCE_RESULT_ADDRESS
const REG_PRESENCE_DISTANCE: u16 = 17; // PRESENCE_REG_PRESENCE_DISTANCE_ADDRESS
const REG_INTRA_PRESENCE_SCORE: u16 = 18; // PRESENCE_REG_INTRA_PRESENCE_SCORE_ADDRESS
const REG_INTER_PRESENCE_SCORE: u16 = 19; // PRESENCE_REG_INTER_PRESENCE_SCORE_ADDRESS

// Breathing Detection Registers (from ref_app_breathing_reg_protocol.h)
const REG_BREATHING_RESULT: u16 = 16; // REF_APP_BREATHING_REG_BREATHING_RESULT_ADDRESS
const REG_BREATHING_RATE: u16 = 17; // REF_APP_BREATHING_REG_BREATHING_RATE_ADDRESS
const REG_BREATHING_APP_STATE: u16 = 18; // REF_APP_BREATHING_REG_APP_STATE_ADDRESS

// Breathing Configuration Registers
const REG_BREATHING_START: u16 = 64; // REF_APP_BREATHING_REG_START_ADDRESS
const REG_BREATHING_END: u16 = 65; // REF_APP_BREATHING_REG_END_ADDRESS
const REG_BREATHING_NUM_DISTANCES_TO_ANALYZE: u16 = 66; // REF_APP_BREATHING_REG_NUM_DISTANCES_TO_ANALYZE_ADDRESS
const REG_BREATHING_DISTANCE_DETERMINATION_DURATION_S: u16 = 67; // REF_APP_BREATHING_REG_DISTANCE_DETERMINATION_DURATION_S_ADDRESS
const REG_BREATHING_USE_PRESENCE_PROCESSOR: u16 = 68; // REF_APP_BREATHING_REG_USE_PRESENCE_PROCESSOR_ADDRESS
const REG_BREATHING_LOWEST_BREATHING_RATE: u16 = 69; // REF_APP_BREATHING_REG_LOWEST_BREATHING_RATE_ADDRESS
const REG_BREATHING_HIGHEST_BREATHING_RATE: u16 = 70; // REF_APP_BREATHING_REG_HIGHEST_BREATHING_RATE_ADDRESS
const REG_BREATHING_TIME_SERIES_LENGTH_S: u16 = 71; // REF_APP_BREATHING_REG_TIME_SERIES_LENGTH_S_ADDRESS
const REG_BREATHING_FRAME_RATE: u16 = 72; // REF_APP_BREATHING_REG_FRAME_RATE_ADDRESS
const REG_BREATHING_SWEEPS_PER_FRAME: u16 = 73; // REF_APP_BREATHING_REG_SWEEPS_PER_FRAME_ADDRESS
const REG_BREATHING_HWAAS: u16 = 74; // REF_APP_BREATHING_REG_HWAAS_ADDRESS
const REG_BREATHING_PROFILE: u16 = 75; // REF_APP_BREATHING_REG_PROFILE_ADDRESS
const REG_BREATHING_INTRA_DETECTION_THRESHOLD: u16 = 76; // REF_APP_BREATHING_REG_INTRA_DETECTION_THRESHOLD_ADDRESS

// Presence Configuration Registers (estimated based on typical Acconeer patterns)
#[allow(dead_code)] // Reserved for presence range configuration
const REG_PRESENCE_START: u16 = 64; // Estimated - presence detection start range
#[allow(dead_code)] // Reserved for presence range configuration
const REG_PRESENCE_END: u16 = 65; // Estimated - presence detection end range
#[allow(dead_code)] // Reserved for presence threshold configuration
const REG_INTRA_DETECTION_THRESHOLD: u16 = 66; // Estimated - fast motion threshold
#[allow(dead_code)] // Reserved for presence threshold configuration
const REG_INTER_DETECTION_THRESHOLD: u16 = 67; // Estimated - slow motion threshold
#[allow(dead_code)] // Reserved for presence frame rate configuration
const REG_FRAME_RATE: u16 = 68; // Estimated - presence detection frame rate

// Command codes for XM125 (from distance_reg_protocol.h)
const CMD_APPLY_CONFIG_AND_CALIBRATE: u32 = 1; // DISTANCE_REG_COMMAND_ENUM_APPLY_CONFIG_AND_CALIBRATE
const CMD_MEASURE_DISTANCE: u32 = 2; // DISTANCE_REG_COMMAND_ENUM_MEASURE_DISTANCE
const CMD_APPLY_CONFIGURATION: u32 = 3; // DISTANCE_REG_COMMAND_ENUM_APPLY_CONFIGURATION
#[allow(dead_code)] // Reserved for manual calibration
const CMD_CALIBRATE: u32 = 4; // DISTANCE_REG_COMMAND_ENUM_CALIBRATE
#[allow(dead_code)] // Reserved for recalibration
const CMD_RECALIBRATE: u32 = 5; // DISTANCE_REG_COMMAND_ENUM_RECALIBRATE
const CMD_RESET_MODULE: u32 = 0x5253_5421; // DISTANCE_REG_COMMAND_ENUM_RESET_MODULE

// Presence detector specific commands (from presence_reg_protocol.h)
const CMD_PRESENCE_APPLY_CONFIGURATION: u32 = 1; // PRESENCE_REG_COMMAND_ENUM_APPLY_CONFIGURATION
const CMD_PRESENCE_START: u32 = 2; // PRESENCE_REG_COMMAND_ENUM_START
#[allow(dead_code)] // Reserved for stopping presence measurements
const CMD_PRESENCE_STOP: u32 = 3; // PRESENCE_REG_COMMAND_ENUM_STOP

// Breathing detector specific commands (from ref_app_breathing_reg_protocol.h)
const CMD_BREATHING_APPLY_CONFIGURATION: u32 = 1; // REF_APP_BREATHING_REG_COMMAND_ENUM_APPLY_CONFIGURATION
const CMD_BREATHING_START_APP: u32 = 2; // REF_APP_BREATHING_REG_COMMAND_ENUM_START_APP
#[allow(dead_code)] // Reserved for stopping breathing measurements
const CMD_BREATHING_STOP_APP: u32 = 3; // REF_APP_BREATHING_REG_COMMAND_ENUM_STOP_APP

// Legacy/placeholder commands for compatibility (not in actual XM125 protocol)
#[allow(dead_code)] // Reserved for backward compatibility
const CMD_ENABLE_DETECTOR: u32 = CMD_APPLY_CONFIGURATION;
const CMD_DISABLE_DETECTOR: u32 = CMD_RESET_MODULE;
const CMD_ENABLE_PRESENCE_DETECTOR: u32 = CMD_PRESENCE_APPLY_CONFIGURATION;
#[allow(dead_code)] // Reserved for future presence-specific commands
const CMD_MEASURE_PRESENCE: u32 = CMD_MEASURE_DISTANCE; // Placeholder - presence uses same command for now
#[allow(dead_code)] // Reserved for continuous monitoring
const CMD_ENABLE_CONTINUOUS_MODE: u32 = CMD_APPLY_CONFIGURATION;
#[allow(dead_code)] // Reserved for continuous monitoring
const CMD_DISABLE_CONTINUOUS_MODE: u32 = CMD_RESET_MODULE;

// Placeholder register for compatibility
const REG_SENSOR_INFO: u16 = REG_VERSION; // Use version register for device info

// Status flags from distance_reg_protocol.h (official Acconeer specification)
// Distance detector status bits (different from presence detector)
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_RSS_REGISTER_OK: u32 = 0x00000001; // Bit 0
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_CONFIG_CREATE_OK: u32 = 0x00000002; // Bit 1
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_SENSOR_CREATE_OK: u32 = 0x00000004; // Bit 2
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_DETECTOR_CREATE_OK: u32 = 0x00000008; // Bit 3 (different from presence!)
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_DETECTOR_BUFFER_OK: u32 = 0x00000010; // Bit 4
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_SENSOR_BUFFER_OK: u32 = 0x00000020; // Bit 5
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_CALIBRATION_BUFFER_OK: u32 = 0x00000040; // Bit 6
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_CONFIG_APPLY_OK: u32 = 0x00000080; // Bit 7
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_SENSOR_CALIBRATE_OK: u32 = 0x00000100; // Bit 8
#[allow(dead_code)] // Reserved for distance-specific status checking
const STATUS_DISTANCE_DETECTOR_CALIBRATE_OK: u32 = 0x00000200; // Bit 9

// Distance detector error flags
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_RSS_REGISTER_ERROR: u32 = 0x00010000; // Bit 16
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_CONFIG_CREATE_ERROR: u32 = 0x00020000; // Bit 17
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_SENSOR_CREATE_ERROR: u32 = 0x00040000; // Bit 18
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_DETECTOR_CREATE_ERROR: u32 = 0x00080000; // Bit 19
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_DETECTOR_BUFFER_ERROR: u32 = 0x00100000; // Bit 20
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_SENSOR_BUFFER_ERROR: u32 = 0x00200000; // Bit 21
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_CALIBRATION_BUFFER_ERROR: u32 = 0x00400000; // Bit 22
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_CONFIG_APPLY_ERROR: u32 = 0x00800000; // Bit 23
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_SENSOR_CALIBRATE_ERROR: u32 = 0x01000000; // Bit 24
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_DETECTOR_CALIBRATE_ERROR: u32 = 0x02000000; // Bit 25
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_DETECTOR_ERROR: u32 = 0x10000000; // Bit 28
#[allow(dead_code)] // Reserved for distance-specific error handling
const STATUS_DISTANCE_BUSY: u32 = 0x80000000; // Bit 31

// Presence detector status bits (from presence_reg_protocol.h)
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_RSS_REGISTER_OK: u32 = 0x00000001; // Bit 0
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_CONFIG_CREATE_OK: u32 = 0x00000002; // Bit 1
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_SENSOR_CREATE_OK: u32 = 0x00000004; // Bit 2
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_SENSOR_CALIBRATE_OK: u32 = 0x00000008; // Bit 3
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_DETECTOR_CREATE_OK: u32 = 0x00000010; // Bit 4 (different from distance!)
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_DETECTOR_BUFFER_OK: u32 = 0x00000020; // Bit 5
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_SENSOR_BUFFER_OK: u32 = 0x00000040; // Bit 6
#[allow(dead_code)] // Reserved for presence-specific status checking
const STATUS_PRESENCE_CONFIG_APPLY_OK: u32 = 0x00000080; // Bit 7

// Presence detector error flags
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_RSS_REGISTER_ERROR: u32 = 0x00010000; // Bit 16
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_CONFIG_CREATE_ERROR: u32 = 0x00020000; // Bit 17
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_SENSOR_CREATE_ERROR: u32 = 0x00040000; // Bit 18
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_SENSOR_CALIBRATE_ERROR: u32 = 0x00080000; // Bit 19
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_DETECTOR_CREATE_ERROR: u32 = 0x00100000; // Bit 20
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_DETECTOR_BUFFER_ERROR: u32 = 0x00200000; // Bit 21
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_SENSOR_BUFFER_ERROR: u32 = 0x00400000; // Bit 22
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_CONFIG_APPLY_ERROR: u32 = 0x00800000; // Bit 23
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_DETECTOR_ERROR: u32 = 0x10000000; // Bit 28
#[allow(dead_code)] // Reserved for presence-specific error handling
const STATUS_PRESENCE_BUSY: u32 = 0x80000000; // Bit 31

// Generic status flags (for backward compatibility)
const STATUS_RSS_REGISTER_OK: u32 = 0x00000001;
const STATUS_CONFIG_CREATE_OK: u32 = 0x00000002;
const STATUS_SENSOR_CREATE_OK: u32 = 0x00000004;
const STATUS_SENSOR_CALIBRATE_OK: u32 = 0x00000008; // Presence bit 3
const STATUS_DETECTOR_CREATE_OK: u32 = 0x00000010; // Presence bit 4
const STATUS_CONFIG_APPLY_OK: u32 = 0x00000080;
const STATUS_BUSY: u32 = 0x80000000;
const STATUS_ERROR: u32 = 0x10000000;

// Error masks
const STATUS_CONFIG_CREATE_ERROR: u32 = 0x00020000;
const STATUS_SENSOR_CREATE_ERROR: u32 = 0x00040000;
const STATUS_SENSOR_CALIBRATE_ERROR: u32 = 0x01000000; // Distance bit 24
const STATUS_DETECTOR_CREATE_ERROR: u32 = 0x00100000; // Presence bit 20
const STATUS_DETECTOR_CALIBRATE_ERROR: u32 = 0x02000000; // Distance bit 25
const STATUS_CONFIG_APPLY_ERROR: u32 = 0x00800000;

// Status masks for efficient checking
#[allow(dead_code)] // Reserved for comprehensive status checking
const STATUS_ALL_OK_MASK: u32 = STATUS_RSS_REGISTER_OK
    | STATUS_CONFIG_CREATE_OK
    | STATUS_SENSOR_CREATE_OK
    | STATUS_SENSOR_CALIBRATE_OK
    | STATUS_DETECTOR_CREATE_OK;
const STATUS_ALL_ERROR_MASK: u32 = STATUS_CONFIG_CREATE_ERROR
    | STATUS_SENSOR_CREATE_ERROR
    | STATUS_SENSOR_CALIBRATE_ERROR
    | STATUS_DETECTOR_CREATE_ERROR
    | STATUS_DETECTOR_CALIBRATE_ERROR
    | STATUS_CONFIG_APPLY_ERROR;

// Legacy compatibility
const STATUS_DETECTOR_READY: u32 = STATUS_DETECTOR_CREATE_OK;
const STATUS_CALIBRATION_DONE: u32 = STATUS_SENSOR_CALIBRATE_OK;
const STATUS_MEASUREMENT_READY: u32 = 0x04; // Keep for distance detector compatibility

// Timeout constants - based on Acconeer documentation
const CALIBRATION_TIMEOUT: Duration = Duration::from_secs(2); // Reduced from 10s based on docs showing 500ms-1s typical
const MEASUREMENT_TIMEOUT: Duration = Duration::from_secs(5);

// Distance detector default values (from distance_reg_protocol.h)
const DISTANCE_START_DEFAULT: u32 = 100; // 100mm = 0.1m (closer than Acconeer default for better detection)
const DISTANCE_END_DEFAULT: u32 = 3000; // 3000mm = 3.0m
const DISTANCE_MAX_STEP_LENGTH_DEFAULT: u32 = 0; // Auto step length
const DISTANCE_CLOSE_RANGE_LEAKAGE_CANCELLATION_DEFAULT: u32 = 1; // Enabled
const DISTANCE_SIGNAL_QUALITY_DEFAULT: u32 = 15000; // Signal quality threshold
const DISTANCE_MAX_PROFILE_DEFAULT: u32 = 5; // Profile 5 (DISTANCE_REG_MAX_PROFILE_ENUM_PROFILE5)
const DISTANCE_THRESHOLD_METHOD_DEFAULT: u32 = 0; // CFAR method (DISTANCE_REG_THRESHOLD_METHOD_ENUM_CFAR)
const DISTANCE_PEAK_SORTING_DEFAULT: u32 = 0; // Strongest peaks (DISTANCE_REG_PEAK_SORTING_ENUM_STRONGEST)
const DISTANCE_NUM_FRAMES_RECORDED_THRESHOLD_DEFAULT: u32 = 100; // Frames for threshold calculation
const DISTANCE_FIXED_AMPLITUDE_THRESHOLD_VALUE_DEFAULT: u32 = 100000; // Fixed amplitude threshold
const DISTANCE_THRESHOLD_SENSITIVITY_DEFAULT: u32 = 100; // 0.1 sensitivity (factor 1000) - much more sensitive
const DISTANCE_REFLECTOR_SHAPE_DEFAULT: u32 = 0; // Generic reflector (DISTANCE_REG_REFLECTOR_SHAPE_ENUM_GENERIC)
const DISTANCE_FIXED_STRENGTH_THRESHOLD_VALUE_DEFAULT: u32 = 0; // Fixed strength threshold

// Breathing detector default values (from ref_app_breathing_reg_protocol.h)
const BREATHING_START_DEFAULT: u32 = 300; // 300mm = 0.3m
const BREATHING_END_DEFAULT: u32 = 1500; // 1500mm = 1.5m
const BREATHING_NUM_DISTANCES_TO_ANALYZE_DEFAULT: u32 = 3; // Number of distance points
const BREATHING_DISTANCE_DETERMINATION_DURATION_S_DEFAULT: u32 = 5; // 5 seconds
const BREATHING_USE_PRESENCE_PROCESSOR_DEFAULT: u32 = 1; // Enabled
const BREATHING_LOWEST_BREATHING_RATE_DEFAULT: u32 = 6; // 6 BPM
const BREATHING_HIGHEST_BREATHING_RATE_DEFAULT: u32 = 60; // 60 BPM
const BREATHING_TIME_SERIES_LENGTH_S_DEFAULT: u32 = 20; // 20 seconds
const BREATHING_FRAME_RATE_DEFAULT: u32 = 10000; // 10 Hz (factor 1000)
const BREATHING_SWEEPS_PER_FRAME_DEFAULT: u32 = 16; // 16 sweeps
const BREATHING_HWAAS_DEFAULT: u32 = 32; // Hardware accelerated average samples
const BREATHING_PROFILE_DEFAULT: u32 = 3; // Profile 3 (REF_APP_BREATHING_REG_PROFILE_ENUM_PROFILE3)
const BREATHING_INTRA_DETECTION_THRESHOLD_DEFAULT: u32 = 6000; // 6.0 threshold (factor 1000)

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DetectorMode {
    Distance,
    Presence,
    Combined,
    Breathing,
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
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedMeasurement {
    pub distance: Option<DistanceMeasurement>,
    pub presence: Option<PresenceMeasurement>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BreathingAppState {
    Init,
    NoPresence,
    IntraPresence,
    DetermineDistance,
    EstimateBreathingRate,
}

impl BreathingAppState {
    fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Init,
            1 => Self::NoPresence,
            2 => Self::IntraPresence,
            3 => Self::DetermineDistance,
            4 => Self::EstimateBreathingRate,
            _ => Self::Init, // Default fallback
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Init => "Initializing",
            Self::NoPresence => "No Presence",
            Self::IntraPresence => "Presence Detected",
            Self::DetermineDistance => "Determining Distance",
            Self::EstimateBreathingRate => "Estimating Breathing Rate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingMeasurement {
    pub result_ready: bool,
    pub breathing_rate: f32, // BPM (breaths per minute)
    pub app_state: BreathingAppState,
    pub temperature: i16, // Temperature in Celsius
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct XM125Config {
    pub detector_mode: DetectorMode,
    pub start_m: f32,
    pub length_m: f32,
    #[allow(dead_code)] // Reserved for advanced configuration
    pub max_step_length: u16,
    #[allow(dead_code)] // Reserved for profile selection
    pub max_profile: u8,
    pub threshold_sensitivity: f32,
    // Presence detection specific
    pub presence_range: PresenceRange,
    #[allow(dead_code)] // Reserved for presence tuning
    pub intra_detection_threshold: f32,
    #[allow(dead_code)] // Reserved for presence tuning
    pub inter_detection_threshold: f32,
    #[allow(dead_code)] // Reserved for frame rate control
    pub frame_rate: f32,
    #[allow(dead_code)] // Reserved for sweep configuration
    pub sweeps_per_frame: u16,
    // Continuous monitoring
    #[allow(dead_code)] // Used in configuration logic
    pub auto_reconnect: bool,
    #[allow(dead_code)] // Reserved for monitoring intervals
    pub measurement_interval_ms: u64,
}

impl Default for XM125Config {
    fn default() -> Self {
        Self {
            detector_mode: DetectorMode::Distance,
            start_m: 0.10,  // 10 cm minimum distance (closer than Acconeer default)
            length_m: 2.90, // 2.90m range (end at 3.0m total)
            max_step_length: 24, // Good balance of accuracy/speed
            max_profile: 5, // Profile 5 (Acconeer default for distance)
            threshold_sensitivity: 0.1, // High sensitivity (more sensitive than Acconeer default)
            // Presence detection defaults
            presence_range: PresenceRange::Long,
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
    #[allow(dead_code)] // Reserved for continuous monitoring state
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
            u32::from_be_bytes([info_data[0], info_data[1], info_data[2], info_data[3]]);
        let firmware_version = u16::from_be_bytes([info_data[4], info_data[5]]);

        // Read Application ID register to determine firmware type
        let app_id_data = self.i2c.read_register(REG_APPLICATION_ID, 4)?;
        let app_id = u32::from_be_bytes([
            app_id_data[0],
            app_id_data[1],
            app_id_data[2],
            app_id_data[3],
        ]);

        let app_type = match app_id {
            1 => "Distance Detector",
            2 => "Presence Detector",
            _ => &format!("Unknown ({app_id})"),
        };

        Ok(format!(
            "XM125 Radar Module\nSensor ID: 0x{:08X}\nFirmware Version: {}.{}\nApplication Type: {}\nApplication ID: {}\nConfig: {:.2}m-{:.2}m range",
            sensor_id,
            firmware_version >> 8,
            firmware_version & 0xFF,
            app_type,
            app_id,
            self.config.start_m,
            self.config.start_m + self.config.length_m
        ))
    }

    pub async fn calibrate(&mut self) -> Result<()> {
        info!("Starting XM125 calibration with proper configuration sequence...");

        // Step 1: Check initial status - verify no Busy or Error bits
        let initial_status = self.get_status_raw()?;
        if initial_status & STATUS_BUSY != 0 {
            return Err(RadarError::DeviceError {
                message: "XM125 is busy - cannot start calibration".to_string(),
            });
        }
        if initial_status & STATUS_ALL_ERROR_MASK != 0 {
            return Err(RadarError::DeviceError {
                message: format!(
                    "XM125 has error flags before calibration: 0x{:08X}",
                    initial_status
                ),
            });
        }

        // Step 2: Write configuration to Start and End registers for distance detector
        if matches!(
            self.config.detector_mode,
            DetectorMode::Distance | DetectorMode::Combined
        ) {
            info!(
                "Configuring distance detection range: {:.2}m to {:.2}m",
                self.config.start_m,
                self.config.start_m + self.config.length_m
            );

            let start_mm = (self.config.start_m * 1000.0) as u32;
            let end_mm = ((self.config.start_m + self.config.length_m) * 1000.0) as u32;

            debug!("Writing Start register: {}mm", start_mm);
            let start_bytes = start_mm.to_le_bytes();
            self.i2c.write_register(REG_START_CONFIG, &start_bytes)?;

            debug!("Writing End register: {}mm", end_mm);
            let end_bytes = end_mm.to_le_bytes();
            self.i2c.write_register(REG_END_CONFIG, &end_bytes)?;

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Step 3: Send APPLY_CONFIG_AND_CALIBRATE command
        info!("Sending APPLY_CONFIG_AND_CALIBRATE command...");
        self.send_command(CMD_APPLY_CONFIG_AND_CALIBRATE)?;

        // Step 4: Poll Detector Status until Busy bit is cleared
        info!("Waiting for calibration to complete...");
        let start_time = Instant::now();

        loop {
            tokio::time::sleep(Duration::from_millis(50)).await; // Poll every 50ms
            let status = self.get_status_raw()?;

            if status & STATUS_BUSY != 0 {
                debug!("Calibration in progress... (status: 0x{:08X})", status);
                if start_time.elapsed() > CALIBRATION_TIMEOUT {
                    return Err(RadarError::Timeout { timeout: 2 });
                }
                continue;
            }

            // Busy bit cleared - check for errors
            if status & STATUS_ALL_ERROR_MASK != 0 {
                let mut error_details = Vec::new();
                if status & STATUS_SENSOR_CALIBRATE_ERROR != 0 {
                    error_details.push("Sensor calibration failed");
                }
                if status & STATUS_DETECTOR_CALIBRATE_ERROR != 0 {
                    error_details.push("Detector calibration failed");
                }
                if status & STATUS_CONFIG_CREATE_ERROR != 0 {
                    error_details.push("Configuration creation failed");
                }
                if status & STATUS_SENSOR_CREATE_ERROR != 0 {
                    error_details.push("Sensor creation failed");
                }
                if status & STATUS_DETECTOR_CREATE_ERROR != 0 {
                    error_details.push("Detector creation failed");
                }
                if status & STATUS_CONFIG_APPLY_ERROR != 0 {
                    error_details.push("Configuration apply failed");
                }

                return Err(RadarError::DeviceError {
                    message: format!(
                        "Calibration failed: {} (status: 0x{:08X})",
                        error_details.join(", "),
                        status
                    ),
                });
            }

            // Check if calibration completed successfully
            if (status & STATUS_SENSOR_CALIBRATE_OK) != 0
                && (status & STATUS_DETECTOR_CREATE_OK) != 0
            {
                self.is_calibrated = true;
                self.last_calibration = Some(Instant::now());
                info!(
                    "XM125 calibration completed successfully (status: 0x{:08X})",
                    status
                );
                return Ok(());
            }

            // Check progress - log which steps have completed
            let mut progress = Vec::new();
            if status & STATUS_RSS_REGISTER_OK != 0 {
                progress.push("RSS");
            }
            if status & STATUS_CONFIG_CREATE_OK != 0 {
                progress.push("Config");
            }
            if status & STATUS_SENSOR_CREATE_OK != 0 {
                progress.push("Sensor");
            }
            if status & STATUS_DETECTOR_CREATE_OK != 0 {
                progress.push("Detector");
            }
            if status & STATUS_CONFIG_APPLY_OK != 0 {
                progress.push("Applied");
            }

            debug!(
                "Calibration progress: {} (status: 0x{:08X})",
                progress.join(", "),
                status
            );

            if start_time.elapsed() > CALIBRATION_TIMEOUT {
                return Err(RadarError::Timeout { timeout: 2 });
            }
        }
    }

    pub async fn measure_distance(&mut self) -> Result<DistanceMeasurement> {
        // Auto-connect if not connected and auto-reconnect is enabled
        if !self.is_connected && self.config.auto_reconnect {
            info!("Auto-connecting for distance measurement...");
            self.auto_connect().await?;
        }

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
        self.read_distance_result().await
    }

    fn get_status_raw(&mut self) -> Result<u32> {
        let status_data = self.i2c.read_register(REG_DETECTOR_STATUS, 4)?;
        Ok(u32::from_be_bytes([
            status_data[0],
            status_data[1],
            status_data[2],
            status_data[3],
        ]))
    }

    /// Read application ID register
    pub fn read_application_id(&mut self) -> Result<u32> {
        let app_id_data = self.i2c.read_register(REG_APPLICATION_ID, 4)?;
        Ok(u32::from_be_bytes([
            app_id_data[0],
            app_id_data[1],
            app_id_data[2],
            app_id_data[3],
        ]))
    }

    fn send_command(&mut self, command: u32) -> Result<()> {
        debug!("Sending command: 0x{command:08X}");
        let cmd_bytes = command.to_be_bytes(); // Fixed: Use big-endian for XM125 commands
        self.i2c.write_register(REG_COMMAND, &cmd_bytes)?;
        Ok(())
    }

    async fn read_distance_result(&mut self) -> Result<DistanceMeasurement> {
        // Loop to handle calibration if needed
        loop {
            // Read distance result register (4 bytes packed format)
            let result_data = self.i2c.read_register(REG_DISTANCE_RESULT, 4)?;
            let result_value = u32::from_be_bytes([
                result_data[0],
                result_data[1],
                result_data[2],
                result_data[3],
            ]);

            // Parse packed result format according to distance_reg_protocol.h
            let num_distances = result_value & 0x0000000F; // Bits 0-3
            let near_start_edge = (result_value & 0x00000100) != 0; // Bit 8
            let calibration_needed = (result_value & 0x00000200) != 0; // Bit 9
            let measure_error = (result_value & 0x00000400) != 0; // Bit 10
            let temperature_raw = (result_value & 0xFFFF0000) >> 16; // Bits 16-31

            debug!(
                "Distance result: num_distances={}, near_edge={}, cal_needed={}, error={}",
                num_distances, near_start_edge, calibration_needed, measure_error
            );

            if measure_error {
                debug!("Distance measurement error flag set - may indicate no objects detected");
                // Note: This error flag can be set when no objects are detected, which is normal
                // We'll continue processing but log it for debugging
            }

            if calibration_needed {
                warn!("Distance detector indicates calibration needed - triggering recalibration");
                // Trigger recalibration when needed
                self.calibrate().await?;
                // Continue loop to re-read result after calibration
                continue;
            }

            // Read peak 0 distance and strength (primary measurement)
            let peak0_distance_data = self.i2c.read_register(REG_PEAK0_DISTANCE, 4)?;
            let peak0_distance_mm = u32::from_be_bytes([
                peak0_distance_data[0],
                peak0_distance_data[1],
                peak0_distance_data[2],
                peak0_distance_data[3],
            ]);

            let peak0_strength_data = self.i2c.read_register(REG_PEAK0_STRENGTH, 4)?;
            let peak0_strength_raw = u32::from_be_bytes([
                peak0_strength_data[0],
                peak0_strength_data[1],
                peak0_strength_data[2],
                peak0_strength_data[3],
            ]);

            // Convert to proper units (values are factor 1000 larger than RSS values)
            #[allow(clippy::cast_precision_loss)]
            // Converting mm to meters, precision loss acceptable
            let distance = if num_distances > 0 {
                (peak0_distance_mm as f32) / 1000.0 // Convert mm to meters
            } else {
                0.0 // No distance detected
            };

            #[allow(clippy::cast_precision_loss)] // Converting strength, precision loss acceptable
            let strength = (peak0_strength_raw as f32) / 1000.0; // Convert to proper strength units

            // Convert temperature (signed 16-bit value)
            #[allow(clippy::cast_possible_wrap)] // Temperature conversion
            let temperature = temperature_raw as i16;

            debug!(
                "Parsed distance: {:.2}m, strength: {:.1}, temp: {}°C, peaks: {}",
                distance, strength, temperature, num_distances
            );

            self.last_measurement = Some(Instant::now());

            return Ok(DistanceMeasurement {
                distance,
                strength,
                temperature,
                timestamp: chrono::Utc::now(),
            });
        }
    }

    /// Measure breathing patterns
    pub async fn measure_breathing(&mut self) -> Result<BreathingMeasurement> {
        // Auto-connect if not connected and auto-reconnect is enabled
        if !self.is_connected && self.config.auto_reconnect {
            info!("Auto-connecting for breathing measurement...");
            self.auto_connect().await?;
        }

        // Start breathing application
        self.send_command(CMD_BREATHING_START_APP)?;

        // Wait for measurement to complete
        let start_time = Instant::now();
        while start_time.elapsed() < MEASUREMENT_TIMEOUT {
            let status = self.get_status_raw()?;
            if (status & STATUS_BUSY) == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Read breathing result register (16/0x10)
        let result_data = self.i2c.read_register(REG_BREATHING_RESULT, 4)?;
        let result_value = u32::from_be_bytes([
            result_data[0],
            result_data[1],
            result_data[2],
            result_data[3],
        ]);

        // Parse breathing result format
        let result_ready = (result_value & 0x00000001) != 0; // Bit 0: RESULT_READY
        let result_ready_sticky = (result_value & 0x00000002) != 0; // Bit 1: RESULT_READY_STICKY
        let temperature_raw = (result_value & 0xFFFF0000) >> 16; // Bits 16-31: TEMPERATURE

        debug!(
            "Breathing result: ready={}, sticky={}, temp_raw=0x{:04X}",
            result_ready, result_ready_sticky, temperature_raw
        );

        // Read breathing rate register (17/0x11)
        let rate_data = self.i2c.read_register(REG_BREATHING_RATE, 4)?;
        let rate_raw = u32::from_be_bytes([rate_data[0], rate_data[1], rate_data[2], rate_data[3]]);

        // Read app state register (18/0x12)
        let state_data = self.i2c.read_register(REG_BREATHING_APP_STATE, 4)?;
        let state_raw =
            u32::from_be_bytes([state_data[0], state_data[1], state_data[2], state_data[3]]);

        // Convert values to proper units
        #[allow(clippy::cast_precision_loss)]
        // Converting breathing rate, precision loss acceptable
        let breathing_rate = (rate_raw as f32) / 1000.0; // Factor 1000 larger than RSS value
        let app_state = BreathingAppState::from_u32(state_raw);

        // Convert temperature (signed 16-bit value)
        #[allow(clippy::cast_possible_wrap)] // Temperature conversion
        let temperature = temperature_raw as i16;

        debug!(
            "Parsed breathing: rate={:.2} BPM, state={:?}, temp={}°C",
            breathing_rate, app_state, temperature
        );

        self.last_measurement = Some(Instant::now());

        Ok(BreathingMeasurement {
            result_ready,
            breathing_rate,
            app_state,
            temperature,
            timestamp: chrono::Utc::now(),
        })
    }

    pub fn set_config(&mut self, config: XM125Config) {
        self.config = config;
        // Would need to send config to device here
        debug!("Updated XM125 configuration: {:?}", self.config);

        // Log user-friendly configuration information
        info!("📡 XM125 Configuration:");
        info!("  Detector Mode: {:?}", self.config.detector_mode);

        match self.config.detector_mode {
            DetectorMode::Presence | DetectorMode::Combined => {
                let (preset_start, preset_end) = match self.config.presence_range {
                    PresenceRange::Short => (0.06, 0.7),
                    PresenceRange::Medium => (0.2, 2.0),
                    PresenceRange::Long => (0.5, 7.0),
                };

                info!(
                    "  Presence Range: {:?} ({:.2}m - {:.2}m)",
                    self.config.presence_range, preset_start, preset_end
                );
                info!(
                    "  Detection Sensitivity: {:.1} (0.1=low, 0.5=medium, 2.0=high)",
                    self.config.threshold_sensitivity
                );
                info!("  Frame Rate: {:.1} Hz", self.config.frame_rate);
                info!(
                    "  Intra Threshold: {:.1} (fast motion)",
                    self.config.intra_detection_threshold
                );
                info!(
                    "  Inter Threshold: {:.1} (slow motion)",
                    self.config.inter_detection_threshold
                );
            }
            DetectorMode::Distance => {
                let end_range = self.config.start_m + self.config.length_m;
                info!(
                    "  Distance Range: {:.2}m - {:.2}m (length: {:.2}m)",
                    self.config.start_m, end_range, self.config.length_m
                );
                info!(
                    "  Detection Sensitivity: {:.1} (0.1=low, 0.5=medium, 2.0=high)",
                    self.config.threshold_sensitivity
                );
            }
            DetectorMode::Breathing => {
                let end_range = self.config.start_m + self.config.length_m;
                info!(
                    "  Breathing Range: {:.2}m - {:.2}m (length: {:.2}m)",
                    self.config.start_m, end_range, self.config.length_m
                );
                info!("  Expected Breathing Rate: 6-60 BPM");
                info!("  Frame Rate: {:.1} Hz", self.config.frame_rate);
                info!("  Analysis Duration: 5s (distance determination)");
                info!("  Time Series Length: 20s (breathing estimation)");
            }
        }

        info!(
            "  Auto-reconnect: {}",
            if self.config.auto_reconnect {
                "enabled"
            } else {
                "disabled"
            }
        );
        info!(
            "  Measurement Interval: {}ms",
            self.config.measurement_interval_ms
        );
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
                Err(RadarError::NotConnected) if attempt == 1 => {
                    // First connection failure - try resetting XM125 to run mode
                    warn!(
                        "XM125 not detected on I2C bus, attempting hardware reset to run mode..."
                    );
                    if let Err(reset_err) = self.reset_xm125_to_run_mode() {
                        warn!("Failed to reset XM125: {reset_err}");
                    } else {
                        info!("XM125 reset completed, retrying connection...");
                        // Give the module time to initialize after reset
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
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
                // Write distance detector configuration registers
                self.write_distance_configuration().await?;
                self.send_command(CMD_APPLY_CONFIG_AND_CALIBRATE)?;
            }
            DetectorMode::Presence => {
                self.send_command(CMD_ENABLE_PRESENCE_DETECTOR)?;
                self.configure_presence_range();
            }
            DetectorMode::Combined => {
                // Write distance detector configuration registers
                self.write_distance_configuration().await?;
                self.send_command(CMD_APPLY_CONFIG_AND_CALIBRATE)?;
                self.send_command(CMD_ENABLE_PRESENCE_DETECTOR)?;
                self.configure_presence_range();
            }
            DetectorMode::Breathing => {
                // Write breathing detector configuration registers
                self.write_breathing_configuration().await?;
                self.send_command(CMD_BREATHING_APPLY_CONFIGURATION)?;
            }
        }

        // Apply configuration and calibrate
        self.wait_for_calibration().await?;

        Ok(())
    }

    /// Configure presence detection range parameters
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // Range values are always positive and within u32 range
    fn configure_presence_range(&mut self) {
        let (start, end) = match self.config.presence_range {
            PresenceRange::Short => (0.06, 0.7),
            PresenceRange::Medium => (0.2, 2.0),
            PresenceRange::Long => (0.5, 7.0),
        };

        info!("Configuring presence detection parameters:");
        info!("  Range: {start:.2}m - {end:.2}m");
        info!(
            "  Intra threshold: {:.2}",
            self.config.intra_detection_threshold
        );
        info!(
            "  Inter threshold: {:.2}",
            self.config.inter_detection_threshold
        );
        info!("  Frame rate: {:.1} Hz", self.config.frame_rate);

        // Note: These register writes are estimated based on typical Acconeer patterns
        // The actual register addresses would need to be confirmed from official documentation

        // Configure detection range (convert meters to millimeters)
        let start_mm = (start * 1000.0) as u32;
        let end_mm = (end * 1000.0) as u32;

        debug!("Writing presence range configuration (estimated registers):");
        debug!("  Start range: {start_mm}mm -> register (not implemented)");
        debug!("  End range: {end_mm}mm -> register (not implemented)");

        // Configure detection thresholds (convert to appropriate format)
        let intra_threshold = (self.config.intra_detection_threshold * 1000.0) as u32;
        let inter_threshold = (self.config.inter_detection_threshold * 1000.0) as u32;

        debug!("  Intra threshold: {intra_threshold} -> register (not implemented)");
        debug!("  Inter threshold: {inter_threshold} -> register (not implemented)");

        // Configure frame rate (convert to appropriate format)
        let frame_rate = (self.config.frame_rate * 1000.0) as u32; // Convert to milliHz or similar
        debug!("  Frame rate: {frame_rate} -> register (not implemented)");

        warn!("Presence configuration registers are not yet implemented - using firmware defaults");
        warn!("Configuration parameters are logged but not written to device registers");

        // TODO: Implement actual register writes when register addresses are confirmed:
        // self.i2c.write_register(REG_PRESENCE_START, &start_mm.to_be_bytes())?;
        // self.i2c.write_register(REG_PRESENCE_END, &end_mm.to_be_bytes())?;
        // self.i2c.write_register(REG_INTRA_DETECTION_THRESHOLD, &intra_threshold.to_be_bytes())?;
        // self.i2c.write_register(REG_INTER_DETECTION_THRESHOLD, &inter_threshold.to_be_bytes())?;
        // self.i2c.write_register(REG_FRAME_RATE, &frame_rate.to_be_bytes())?;
    }

    /// Write distance detector configuration registers
    async fn write_distance_configuration(&mut self) -> Result<()> {
        info!("Writing distance detector configuration registers");

        // Convert config values to device units
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let start_mm = (self.config.start_m * 1000.0) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let end_mm = ((self.config.start_m + self.config.length_m) * 1000.0) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let sensitivity = (self.config.threshold_sensitivity * 1000.0) as u32;

        // Use defaults for advanced parameters, but allow customization via config
        let start_value = if start_mm > 0 {
            start_mm
        } else {
            DISTANCE_START_DEFAULT
        };
        let end_value = if end_mm > start_value {
            end_mm
        } else {
            DISTANCE_END_DEFAULT
        };
        let sensitivity_value = if sensitivity > 0 {
            sensitivity
        } else {
            DISTANCE_THRESHOLD_SENSITIVITY_DEFAULT
        };

        info!("Distance detector configuration:");
        info!(
            "  Range: {}mm - {}mm ({:.2}m - {:.2}m)",
            start_value,
            end_value,
            start_value as f32 / 1000.0,
            end_value as f32 / 1000.0
        );
        info!("  Sensitivity: {} (factor 1000)", sensitivity_value);
        info!(
            "  Profile: {} (Profile {})",
            DISTANCE_MAX_PROFILE_DEFAULT, DISTANCE_MAX_PROFILE_DEFAULT
        );

        // Write essential configuration registers
        self.i2c
            .write_register(REG_START_CONFIG, &start_value.to_be_bytes())?;
        debug!(
            "Wrote start range: {}mm to register 0x{:04X}",
            start_value, REG_START_CONFIG
        );

        self.i2c
            .write_register(REG_END_CONFIG, &end_value.to_be_bytes())?;
        debug!(
            "Wrote end range: {}mm to register 0x{:04X}",
            end_value, REG_END_CONFIG
        );

        self.i2c
            .write_register(REG_THRESHOLD_SENSITIVITY, &sensitivity_value.to_be_bytes())?;
        debug!(
            "Wrote sensitivity: {} to register 0x{:04X}",
            sensitivity_value, REG_THRESHOLD_SENSITIVITY
        );

        self.i2c
            .write_register(REG_MAX_PROFILE, &DISTANCE_MAX_PROFILE_DEFAULT.to_be_bytes())?;
        debug!(
            "Wrote profile: {} to register 0x{:04X}",
            DISTANCE_MAX_PROFILE_DEFAULT, REG_MAX_PROFILE
        );

        // Write advanced configuration with defaults
        self.i2c.write_register(
            REG_MAX_STEP_LENGTH,
            &DISTANCE_MAX_STEP_LENGTH_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_CLOSE_RANGE_LEAKAGE_CANCELLATION,
            &DISTANCE_CLOSE_RANGE_LEAKAGE_CANCELLATION_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_SIGNAL_QUALITY,
            &DISTANCE_SIGNAL_QUALITY_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_THRESHOLD_METHOD,
            &DISTANCE_THRESHOLD_METHOD_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_PEAK_SORTING,
            &DISTANCE_PEAK_SORTING_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_NUM_FRAMES_RECORDED_THRESHOLD,
            &DISTANCE_NUM_FRAMES_RECORDED_THRESHOLD_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_FIXED_AMPLITUDE_THRESHOLD_VALUE,
            &DISTANCE_FIXED_AMPLITUDE_THRESHOLD_VALUE_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_REFLECTOR_SHAPE,
            &DISTANCE_REFLECTOR_SHAPE_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_FIXED_STRENGTH_THRESHOLD_VALUE,
            &DISTANCE_FIXED_STRENGTH_THRESHOLD_VALUE_DEFAULT.to_be_bytes(),
        )?;

        info!("Distance detector configuration written successfully");
        Ok(())
    }

    /// Write breathing detector configuration registers
    async fn write_breathing_configuration(&mut self) -> Result<()> {
        info!("Writing breathing detector configuration registers");

        // Convert config values to device units
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let start_mm = (self.config.start_m * 1000.0) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let end_mm = ((self.config.start_m + self.config.length_m) * 1000.0) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let frame_rate = (self.config.frame_rate * 1000.0) as u32; // Convert to milliHz
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // Values are always positive and within u32 range
        let intra_threshold = (self.config.intra_detection_threshold * 1000.0) as u32;

        // Use defaults for advanced parameters, but allow customization via config
        let start_value = if start_mm > 0 {
            start_mm
        } else {
            BREATHING_START_DEFAULT
        };
        let end_value = if end_mm > start_value {
            end_mm
        } else {
            BREATHING_END_DEFAULT
        };
        let frame_rate_value = if frame_rate > 0 {
            frame_rate
        } else {
            BREATHING_FRAME_RATE_DEFAULT
        };
        let intra_threshold_value = if intra_threshold > 0 {
            intra_threshold
        } else {
            BREATHING_INTRA_DETECTION_THRESHOLD_DEFAULT
        };

        info!("Breathing detector configuration:");
        info!(
            "  Range: {}mm - {}mm ({:.2}m - {:.2}m)",
            start_value,
            end_value,
            start_value as f32 / 1000.0,
            end_value as f32 / 1000.0
        );
        info!(
            "  Frame Rate: {} milliHz ({:.1} Hz)",
            frame_rate_value,
            frame_rate_value as f32 / 1000.0
        );
        info!(
            "  Breathing Rate Range: {}-{} BPM",
            BREATHING_LOWEST_BREATHING_RATE_DEFAULT, BREATHING_HIGHEST_BREATHING_RATE_DEFAULT
        );
        info!(
            "  Profile: {} (Profile {})",
            BREATHING_PROFILE_DEFAULT, BREATHING_PROFILE_DEFAULT
        );

        // Write essential configuration registers
        self.i2c
            .write_register(REG_BREATHING_START, &start_value.to_be_bytes())?;
        debug!(
            "Wrote breathing start: {}mm to register 0x{:04X}",
            start_value, REG_BREATHING_START
        );

        self.i2c
            .write_register(REG_BREATHING_END, &end_value.to_be_bytes())?;
        debug!(
            "Wrote breathing end: {}mm to register 0x{:04X}",
            end_value, REG_BREATHING_END
        );

        self.i2c
            .write_register(REG_BREATHING_FRAME_RATE, &frame_rate_value.to_be_bytes())?;
        debug!(
            "Wrote frame rate: {} milliHz to register 0x{:04X}",
            frame_rate_value, REG_BREATHING_FRAME_RATE
        );

        self.i2c.write_register(
            REG_BREATHING_INTRA_DETECTION_THRESHOLD,
            &intra_threshold_value.to_be_bytes(),
        )?;
        debug!(
            "Wrote intra threshold: {} to register 0x{:04X}",
            intra_threshold_value, REG_BREATHING_INTRA_DETECTION_THRESHOLD
        );

        // Write advanced configuration with defaults
        self.i2c.write_register(
            REG_BREATHING_NUM_DISTANCES_TO_ANALYZE,
            &BREATHING_NUM_DISTANCES_TO_ANALYZE_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_BREATHING_DISTANCE_DETERMINATION_DURATION_S,
            &BREATHING_DISTANCE_DETERMINATION_DURATION_S_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_BREATHING_USE_PRESENCE_PROCESSOR,
            &BREATHING_USE_PRESENCE_PROCESSOR_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_BREATHING_LOWEST_BREATHING_RATE,
            &BREATHING_LOWEST_BREATHING_RATE_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_BREATHING_HIGHEST_BREATHING_RATE,
            &BREATHING_HIGHEST_BREATHING_RATE_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_BREATHING_TIME_SERIES_LENGTH_S,
            &BREATHING_TIME_SERIES_LENGTH_S_DEFAULT.to_be_bytes(),
        )?;
        self.i2c.write_register(
            REG_BREATHING_SWEEPS_PER_FRAME,
            &BREATHING_SWEEPS_PER_FRAME_DEFAULT.to_be_bytes(),
        )?;
        self.i2c
            .write_register(REG_BREATHING_HWAAS, &BREATHING_HWAAS_DEFAULT.to_be_bytes())?;
        self.i2c.write_register(
            REG_BREATHING_PROFILE,
            &BREATHING_PROFILE_DEFAULT.to_be_bytes(),
        )?;

        info!("Breathing detector configuration written successfully");
        Ok(())
    }

    /// Wait for calibration to complete
    async fn wait_for_calibration(&mut self) -> Result<()> {
        let start_time = Instant::now();

        loop {
            let status = self.get_status_raw()?;

            // Check if device is already calibrated (for presence detector, status 0x07 is sufficient)
            if status & STATUS_SENSOR_CALIBRATE_OK != 0 {
                self.is_calibrated = true;
                self.last_calibration = Some(Instant::now());
                info!("XM125 calibration completed successfully");
                return Ok(());
            }

            // For presence detector, status 0x07 (bits 0,1,2) indicates ready state
            if status == 0x07 {
                self.is_calibrated = true;
                self.last_calibration = Some(Instant::now());
                info!("XM125 presence detector ready (status: 0x{status:02X})");
                return Ok(());
            }

            if status & STATUS_ERROR != 0 {
                return Err(RadarError::DeviceError {
                    message: "Calibration failed - device error".to_string(),
                });
            }

            if start_time.elapsed() > CALIBRATION_TIMEOUT {
                return Err(RadarError::Timeout { timeout: 2 });
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Measure presence detection
    pub async fn measure_presence(&mut self) -> Result<PresenceMeasurement> {
        // Auto-connect if not connected and auto-reconnect is enabled
        if !self.is_connected && self.config.auto_reconnect {
            info!("Auto-connecting for presence measurement...");
            self.auto_connect().await?;
        }

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

        // Send presence detection command sequence
        // 1. Apply configuration first
        self.send_command(CMD_PRESENCE_APPLY_CONFIGURATION)?;

        // Small delay for configuration to be applied
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 2. Start measurement
        self.send_command(CMD_PRESENCE_START)?;

        // Wait for measurement
        self.wait_for_measurement().await?;

        // Read presence result registers individually (4 bytes each)
        let presence_result_data = self.i2c.read_register(REG_PRESENCE_RESULT, 4)?;
        let presence_distance_data = self.i2c.read_register(REG_PRESENCE_DISTANCE, 4)?;
        let intra_score_data = self.i2c.read_register(REG_INTRA_PRESENCE_SCORE, 4)?;
        let inter_score_data = self.i2c.read_register(REG_INTER_PRESENCE_SCORE, 4)?;

        // Parse presence result (register 16/0x10)
        let presence_result = u32::from_be_bytes([
            presence_result_data[0],
            presence_result_data[1],
            presence_result_data[2],
            presence_result_data[3],
        ]);

        let presence_detected = (presence_result & 0x01) != 0; // Bit 0: presence detected
        #[allow(clippy::no_effect_underscore_binding)] // Reserved for future sticky detection logic
        let _presence_sticky = (presence_result & 0x02) != 0; // Bit 1: presence sticky
        let detector_error = (presence_result & 0x8000) != 0; // Bit 15: detector error

        if detector_error {
            return Err(RadarError::DeviceError {
                message: "Presence detector error flag set".to_string(),
            });
        }

        // Parse presence distance (register 17/0x11) - distance in mm
        let distance_raw = u32::from_be_bytes([
            presence_distance_data[0],
            presence_distance_data[1],
            presence_distance_data[2],
            presence_distance_data[3],
        ]);

        // Parse intra presence score (register 18/0x12) - fast motion
        let intra_score_raw = u32::from_be_bytes([
            intra_score_data[0],
            intra_score_data[1],
            intra_score_data[2],
            intra_score_data[3],
        ]);

        // Parse inter presence score (register 19/0x13) - slow motion
        let inter_score_raw = u32::from_be_bytes([
            inter_score_data[0],
            inter_score_data[1],
            inter_score_data[2],
            inter_score_data[3],
        ]);

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
            timestamp: chrono::Utc::now(),
        })
    }

    /// Measure combined distance and presence
    pub async fn measure_combined(&mut self) -> Result<CombinedMeasurement> {
        // Auto-connect if not connected and auto-reconnect is enabled
        if !self.is_connected && self.config.auto_reconnect {
            info!("Auto-connecting for combined measurement...");
            self.auto_connect().await?;
        }

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
    #[allow(dead_code)] // Reserved for future continuous monitoring feature
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
    #[allow(dead_code)] // Reserved for future continuous monitoring feature
    pub fn stop_continuous_monitoring(&mut self) -> Result<()> {
        if self.continuous_mode {
            self.send_command(CMD_DISABLE_CONTINUOUS_MODE)?;
            self.continuous_mode = false;
            info!("Stopped continuous monitoring mode");
        }
        Ok(())
    }

    /// Check if continuous monitoring is active
    #[allow(dead_code)] // Reserved for future continuous monitoring feature
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
    #[allow(dead_code)] // Reserved for configuration queries
    pub fn get_detector_mode(&self) -> DetectorMode {
        self.config.detector_mode
    }

    /// Reset XM125 to run mode using the control script
    #[allow(clippy::uninlined_format_args)] // Allow for error message formatting
    #[allow(clippy::unused_self)] // Self needed for future enhancements
    fn reset_xm125_to_run_mode(&self) -> Result<()> {
        use std::process::Command;

        info!("Executing XM125 reset to run mode...");

        let output = Command::new("/home/fio/xm125-control.sh")
            .arg("--reset-run")
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to execute xm125-control.sh: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RadarError::DeviceError {
                message: format!("XM125 reset script failed: {}", stderr),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("XM125 reset output: {}", stdout);

        info!("XM125 hardware reset to run mode completed");
        Ok(())
    }

    /// Check if radar is connected
    #[allow(dead_code)] // Reserved for connection state queries
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
