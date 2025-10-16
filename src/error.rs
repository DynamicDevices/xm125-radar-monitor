use thiserror::Error;

#[derive(Error, Debug)]
pub enum RadarError {
    #[error("I2C communication error: {0}")]
    I2c(#[from] linux_embedded_hal::I2CError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command timeout after {timeout}s")]
    Timeout { timeout: u64 },

    #[error("Invalid response from XM125: {response}")]
    #[allow(dead_code)] // Reserved for future protocol validation
    InvalidResponse { response: String },

    #[error("XM125 returned error: {message}")]
    DeviceError { message: String },

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("XM125 not connected or not responding")]
    NotConnected,

    #[error("Invalid command parameters: {0}")]
    InvalidParameters(String),

    #[error("Calibration required - temperature change detected")]
    #[allow(dead_code)] // Reserved for automatic calibration logic
    CalibrationRequired,

    #[error("Radar initialization failed: {0}")]
    #[allow(dead_code)] // Reserved for initialization error handling
    InitializationFailed(String),

    #[error("Distance measurement failed: {0}")]
    MeasurementFailed(String),

    #[error("Firmware error: {message}")]
    #[allow(dead_code)] // Reserved for firmware management error handling
    FirmwareError { message: String },

    #[error("XM125 module not programmed or not responding")]
    #[allow(dead_code)] // Reserved for unprogrammed module detection
    ModuleNotProgrammed,
}

pub type Result<T> = std::result::Result<T, RadarError>;
