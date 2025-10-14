use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// I2C device path (e.g., /dev/i2c-1)
    #[arg(short = 'd', long, default_value = "/dev/i2c-1")]
    pub i2c_device: String,

    /// I2C address of the XM125 module (7-bit address)
    #[arg(short = 'a', long, default_value_t = 0x52)]
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
