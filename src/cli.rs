use clap::{Parser, Subcommand, ValueEnum};

/// Parse I2C address from string, supporting both decimal and hex formats
fn parse_i2c_address(s: &str) -> Result<u16, String> {
    if let Some(hex_str) = s.strip_prefix("0x") {
        u16::from_str_radix(hex_str, 16).map_err(|_| format!("Invalid hex I2C address: {s}"))
    } else {
        s.parse::<u16>()
            .map_err(|_| format!("Invalid I2C address: {s}"))
    }
}

impl Cli {
    /// Get the I2C device path, using bus number if device path not specified
    pub fn get_i2c_device_path(&self) -> String {
        if let Some(device) = &self.i2c_device {
            device.clone()
        } else if let Some(bus) = self.i2c_bus {
            format!("/dev/i2c-{bus}")
        } else {
            "/dev/i2c-1".to_string() // Default fallback
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// I2C bus number (will be used as /dev/i2c-N if --i2c-device not specified)
    #[arg(short = 'b', long)]
    pub i2c_bus: Option<u8>,

    /// I2C device path (e.g., /dev/i2c-1)
    #[arg(short = 'd', long)]
    pub i2c_device: Option<String>,

    /// I2C address of the XM125 module (7-bit address, hex format supported)
    #[arg(short = 'a', long, value_parser = parse_i2c_address, default_value = "0x52")]
    pub i2c_address: u16,

    /// Command timeout in seconds
    #[arg(short, long, default_value_t = 3)]
    pub timeout: u64,

    /// Output format
    #[arg(short, long, default_value = "human")]
    pub format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    pub quiet: bool,

    /// Detector mode (distance, presence, or combined)
    #[arg(short = 'm', long, default_value = "distance")]
    pub mode: DetectorMode,

    /// Enable auto-reconnect on connection failures
    #[arg(long)]
    pub auto_reconnect: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Get XM125 radar status
    Status,

    /// Connect to XM125 radar with automatic configuration
    Connect {
        /// Force reconnection even if already connected
        #[arg(short, long)]
        force: bool,
    },

    /// Disconnect from XM125 radar
    Disconnect,

    /// Get XM125 device information
    Info,

    /// Perform single distance measurement
    Measure,

    /// Perform single presence detection
    Presence,

    /// Perform combined distance and presence measurement
    Combined,

    /// Calibrate the XM125 radar sensor
    Calibrate,

    /// Continuously monitor with the configured detector mode
    Monitor {
        /// Measurement interval in milliseconds
        #[arg(short, long, default_value_t = 1000)]
        interval: u64,

        /// Number of measurements (0 = infinite)
        #[arg(short, long)]
        count: Option<u32>,

        /// Save measurements to file (CSV format)
        #[arg(short, long)]
        save_to: Option<String>,
    },

    /// Set detector configuration
    Config {
        /// Detection range start in meters
        #[arg(long)]
        start: Option<f32>,

        /// Detection range length in meters  
        #[arg(long)]
        length: Option<f32>,

        /// Presence detection range preset
        #[arg(long)]
        presence_range: Option<PresenceRange>,

        /// Threshold sensitivity (0.1 - 2.0)
        #[arg(long)]
        sensitivity: Option<f32>,

        /// Frame rate for presence detection
        #[arg(long)]
        frame_rate: Option<f32>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Csv,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DetectorMode {
    Distance,
    Presence,
    Combined,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PresenceRange {
    Short,
    Medium,
    Long,
}
