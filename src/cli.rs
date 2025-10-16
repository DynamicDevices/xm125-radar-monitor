use clap::{Parser, Subcommand, ValueEnum};

use crate::firmware;

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
        } else {
            format!("/dev/i2c-{}", self.i2c_bus)
        }
    }
}

#[derive(Parser)]
#[command(
    author = "Dynamic Devices Ltd",
    version,
    about = "XM125 Radar Module Monitor - Test and monitor Acconeer XM125 radar sensors",
    long_about = "XM125 Radar Module Monitor

A comprehensive tool for testing and monitoring Acconeer XM125 radar modules via I2C.
Supports both distance detection and presence detection modes with real-time monitoring.

TECHNICIAN QUICK START:
  1. Check device status:        ./xm125-radar-monitor status
  2. Get firmware info:          ./xm125-radar-monitor info  
  3. Test presence detection:    ./xm125-radar-monitor presence
  4. Continuous monitoring:      ./xm125-radar-monitor monitor --count 10
  5. Save data to CSV:           ./xm125-radar-monitor monitor --save-to data.csv

COMMON EXAMPLES:
  # Check device status (shows actual firmware mode)
  xm125-radar-monitor status

  # Test presence detection (default mode)
  xm125-radar-monitor presence

  # Test distance measurement  
  xm125-radar-monitor --mode distance measure

  # Continuous presence monitoring (10 samples, 500ms interval)
  xm125-radar-monitor monitor --count 10 --interval 500

  # Put module into bootloader mode for firmware programming
  xm125-radar-monitor bootloader

  # Use custom I2C bus and address
  xm125-radar-monitor -b 1 -a 0x53 status

  # Enable debug logging for troubleshooting
  xm125-radar-monitor --verbose --mode presence presence

  # Save measurements to CSV file
  xm125-radar-monitor --mode presence monitor --save-to presence_data.csv

TROUBLESHOOTING:
  - If device not found: Check I2C bus/address with 'i2cdetect -y 2'
  - If 'Permission denied': Run with 'sudo' for I2C access
  - If 'Unknown command' errors: Device may need reset via GPIO control
  - Use --verbose flag to see detailed I2C communication

For Sentai targets, use default settings (I2C bus 2, address 0x52)."
)]
#[allow(clippy::struct_excessive_bools)] // CLI flags are naturally boolean
pub struct Cli {
    /// I2C bus number (will be used as /dev/i2c-N if --i2c-device not specified) [default: 2 for Sentai target]
    #[arg(short = 'b', long, default_value_t = 2)]
    pub i2c_bus: u8,

    /// I2C device path (e.g., /dev/i2c-2 for Sentai target)
    #[arg(short = 'd', long)]
    pub i2c_device: Option<String>,

    /// I2C address of XM125 module in hex (e.g., 0x52 for standard XM125)
    #[arg(short = 'a', long, value_parser = parse_i2c_address, default_value = "0x52")]
    pub i2c_address: u16,

    /// Command timeout in seconds (how long to wait for device responses)
    #[arg(short, long, default_value_t = 3)]
    pub timeout: u64,

    /// Output format for measurement data
    #[arg(short, long, default_value = "human")]
    pub format: OutputFormat,

    /// Enable verbose debug logging (shows I2C transactions and internal state)
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress startup banner and configuration info
    #[arg(short, long)]
    pub quiet: bool,

    /// Detector mode: distance, presence, or combined measurements
    #[arg(short = 'm', long, default_value = "presence")]
    pub mode: DetectorMode,

    /// Enable auto-reconnect on connection failures (enabled by default)
    #[arg(
        long,
        default_value_t = true,
        help = "Automatically reconnect if device becomes unresponsive"
    )]
    pub auto_reconnect: bool,

    /// Disable auto-reconnect (use simple connection without retry logic)
    #[arg(
        long,
        conflicts_with = "auto_reconnect",
        help = "Disable automatic reconnection for debugging"
    )]
    pub no_auto_reconnect: bool,

    /// GPIO pin number for XM125 WAKEUP signal (optional hardware control)
    #[arg(
        long,
        help = "GPIO pin for hardware wake control (e.g., 139 for Sentai)"
    )]
    pub wakeup_pin: Option<u32>,

    /// GPIO pin number for XM125 INT signal (optional hardware monitoring)  
    #[arg(
        long,
        help = "GPIO pin for interrupt monitoring (e.g., 125 for Sentai)"
    )]
    pub int_pin: Option<u32>,

    /// Enable automatic firmware updates when detector mode doesn't match
    #[arg(long, help = "Automatically update firmware if wrong type is detected")]
    pub auto_update_firmware: bool,

    /// Verify firmware after auto-updates (may cause timeouts)
    #[arg(long, help = "Verify firmware after automatic updates")]
    pub auto_verify_firmware: bool,

    /// Firmware directory path (contains .bin files)
    #[arg(
        long,
        default_value = "/lib/firmware/acconeer",
        help = "Directory containing firmware binaries"
    )]
    pub firmware_path: String,

    /// XM125 control script path
    #[arg(
        long,
        default_value = "/home/fio/xm125-control.sh",
        help = "Path to XM125 GPIO control script"
    )]
    pub control_script: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Check XM125 radar status and initialization state
    ///
    /// Shows device status flags, initialization progress, and error conditions.
    /// Use this first to verify the device is responding and properly initialized.
    Status,

    /// Connect to XM125 radar with automatic configuration and calibration
    ///
    /// Establishes I2C connection, configures the detector mode, and performs
    /// calibration if needed. The device must be connected before measurements.
    Connect {
        /// Force reconnection even if already connected
        #[arg(short, long, help = "Reset connection state and reconnect")]
        force: bool,
    },

    /// Disconnect from XM125 radar and put device in low power mode
    Disconnect,

    /// Get XM125 device information and firmware details
    ///
    /// Displays sensor ID, firmware version, application type (distance/presence),
    /// and current configuration. Useful for verifying correct firmware is loaded.
    Info,

    /// Perform single distance measurement (requires --mode distance)
    ///
    /// Takes one distance reading and displays distance, signal strength, and temperature.
    /// The device must be in distance detector mode and properly calibrated.
    Measure,

    /// Perform single presence detection (requires --mode presence)  
    ///
    /// Takes one presence reading showing detection status, distance to detected object,
    /// and motion scores (intra=fast motion, inter=slow motion). Device must be in
    /// presence detector mode.
    Presence,

    /// Perform combined distance and presence measurement (requires --mode combined)
    ///
    /// Takes both distance and presence readings in a single operation.
    /// Useful for applications requiring both measurement types.
    Combined,

    /// Perform breathing detection measurement (requires --mode breathing)
    ///
    /// Monitors breathing patterns and estimates breathing rate in BPM.
    /// Shows application state, breathing rate, and presence information.
    Breathing,

    /// Calibrate the XM125 radar sensor
    ///
    /// Forces recalibration of the sensor. Calibration is normally done automatically
    /// during connection, but manual calibration may be needed after environmental changes.
    Calibrate,

    /// Continuously monitor with the configured detector mode
    ///
    /// Takes repeated measurements at specified intervals. Use --count to limit
    /// the number of samples, or omit for continuous monitoring (Ctrl+C to stop).
    /// Measurements can be saved to CSV file with --save-to option.
    Monitor {
        /// Measurement interval in milliseconds (minimum ~100ms recommended)
        #[arg(
            short,
            long,
            default_value_t = 1000,
            help = "Time between measurements in ms"
        )]
        interval: u64,

        /// Number of measurements (0 or omit = infinite)
        #[arg(short, long, help = "Stop after N measurements (omit for continuous)")]
        count: Option<u32>,

        /// Save measurements to file (CSV format)
        #[arg(short, long, help = "Output CSV file path (e.g., measurements.csv)")]
        save_to: Option<String>,
    },

    /// Configure detector parameters (advanced)
    ///
    /// Modify detection parameters like range, sensitivity, and thresholds.
    /// Most users should use the default settings. Changes take effect on next connection.
    Config {
        /// Detection range start in meters (distance mode)
        #[arg(long, help = "Start of detection range (e.g., 0.2 for 20cm minimum)")]
        start: Option<f32>,

        /// Detection range length in meters (distance mode)
        #[arg(long, help = "Length of detection range (e.g., 3.0 for 3m range)")]
        length: Option<f32>,

        /// Presence detection range preset (presence mode)
        #[arg(
            long,
            help = "Predefined range: short (6-70cm), medium (20cm-2m), long (50cm-7m)"
        )]
        presence_range: Option<PresenceRange>,

        /// Threshold sensitivity (0.1 = low, 0.5 = medium, 2.0 = high)
        #[arg(
            long,
            help = "Detection sensitivity: lower = less sensitive, higher = more sensitive"
        )]
        sensitivity: Option<f32>,

        /// Frame rate for presence detection in Hz
        #[arg(
            long,
            help = "Measurement frequency in Hz (e.g., 12.0 for 12 measurements/second)"
        )]
        frame_rate: Option<f32>,
    },

    /// Firmware management commands
    Firmware {
        #[command(subcommand)]
        action: FirmwareAction,
    },

    /// Put XM125 module into bootloader mode for firmware programming
    ///
    /// Uses GPIO control to reset the module into bootloader mode (I2C address 0x48).
    /// This is required for firmware programming with stm32flash. The module will
    /// remain in bootloader mode until reset or power cycled.
    Bootloader {
        /// Reset to run mode after entering bootloader (for testing)
        #[arg(short, long, help = "Reset back to run mode after bootloader entry")]
        reset: bool,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum FirmwareAction {
    /// Check current firmware type and version
    Check,

    /// Update firmware to match the specified detector mode
    ///
    /// Automatically flashes the correct firmware binary for the selected mode.
    /// Uses stm32flash and GPIO control for safe firmware updates.
    Update {
        /// Target firmware type (distance, presence, or breathing)
        #[arg(help = "Firmware type: distance, presence, or breathing")]
        firmware_type: FirmwareType,

        /// Force update even if firmware already matches
        #[arg(short, long, help = "Force firmware update even if already correct")]
        force: bool,

        /// Verify firmware after update (adds delay and may timeout)
        #[arg(long, help = "Verify firmware installation after update")]
        verify: bool,
    },

    /// Verify firmware integrity using checksums
    Verify {
        /// Firmware type to verify against
        firmware_type: Option<FirmwareType>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum FirmwareType {
    /// Distance detector firmware
    Distance,
    /// Presence detector firmware  
    Presence,
    /// Breathing monitor firmware
    Breathing,
}

impl From<FirmwareType> for firmware::FirmwareType {
    fn from(cli_type: FirmwareType) -> Self {
        match cli_type {
            FirmwareType::Distance => firmware::FirmwareType::Distance,
            FirmwareType::Presence => firmware::FirmwareType::Presence,
            FirmwareType::Breathing => firmware::FirmwareType::Breathing,
        }
    }
}

impl From<crate::cli::DetectorMode> for firmware::FirmwareType {
    fn from(mode: crate::cli::DetectorMode) -> Self {
        match mode {
            DetectorMode::Distance => firmware::FirmwareType::Distance,
            DetectorMode::Presence | DetectorMode::Combined => firmware::FirmwareType::Presence, // Default to presence for combined
            DetectorMode::Breathing => firmware::FirmwareType::Breathing,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output with labels and units (default)
    Human,
    /// JSON format for programmatic processing  
    Json,
    /// Comma-separated values for data analysis
    Csv,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DetectorMode {
    /// Distance measurement mode - measures range to objects
    Distance,
    /// Presence detection mode - detects motion and presence
    Presence,
    /// Combined mode - both distance and presence measurements
    Combined,
    /// Breathing detection mode - monitors breathing patterns
    Breathing,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PresenceRange {
    /// Short range: 6cm to 70cm (good for close proximity detection)
    Short,
    /// Medium range: 20cm to 2m (balanced range and sensitivity)  
    Medium,
    /// Long range: 50cm to 7m (maximum detection range)
    Long,
}
