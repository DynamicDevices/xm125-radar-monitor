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
    CalibrationRequired,

    #[error("Radar initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Distance measurement failed: {0}")]
    MeasurementFailed(String),
}

pub type Result<T> = std::result::Result<T, RadarError>;
