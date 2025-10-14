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

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Get XM125 radar status
    Status,
    
    /// Connect to XM125 radar
    Connect,
    
    /// Disconnect from XM125 radar
    Disconnect,
    
    /// Get XM125 device information
    Info,

    /// Perform single distance measurement
    Measure,

    /// Calibrate the XM125 radar sensor
    Calibrate,

    /// Continuously monitor distances
    Monitor {
        /// Measurement interval in milliseconds
        #[arg(short, long, default_value_t = 1000)]
        interval: u64,
        
        /// Number of measurements (0 = infinite)
        #[arg(short, long)]
        count: Option<u32>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Csv,
}
