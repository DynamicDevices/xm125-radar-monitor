#![allow(dead_code)] // Allow dead code during restructure

use clap::Parser;
use log::{error, info, warn};
use std::env;
use std::process;

mod cli;
mod commands;
mod config;
mod display;
mod error;
mod fifo;
mod firmware;
mod gpio;
mod handlers;
mod i2c;
mod monitoring;
mod radar;

use cli::{Cli, Commands, FirmwareAction};
use commands::execute_command;
use error::RadarError;
use fifo::FifoWriter;
use handlers::{
    handle_bootloader_command, handle_firmware_checksum_command, handle_firmware_erase_command,
    handle_gpio_command,
};
use radar::XM125Radar;

/// Application entry point
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.logging.verbose {
        env::set_var("RUST_LOG", "debug");
    } else {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Run the application
    if let Err(e) = run(cli).await {
        error!("Application error: {e}");
        process::exit(1);
    }
}

/// Main application logic
async fn run(cli: Cli) -> Result<(), RadarError> {
    // Handle commands that don't need I2C connection first
    match &cli.command {
        Commands::Firmware { action } => match action {
            FirmwareAction::Checksum {
                firmware_type,
                verbose,
            } => {
                return handle_firmware_checksum_command(
                    firmware_type.as_ref(),
                    *verbose,
                    &cli.firmware_path,
                );
            }
            FirmwareAction::Erase { confirm } => {
                return handle_firmware_erase_command(*confirm).await;
            }
            FirmwareAction::Bootloader { test_mode } => {
                return handle_bootloader_command(&cli, *test_mode).await;
            }
            _ => {} // Other firmware commands need I2C connection
        },
        Commands::Gpio { action } => {
            return handle_gpio_command(&cli, action);
        }
        _ => {} // Other commands need I2C connection
    }

    // Print startup banner unless quiet mode
    if !cli.output.quiet {
        println!("xm125-radar-monitor v{}", env!("CARGO_PKG_VERSION"));
        println!("Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.");
        println!("XM125 Radar Module Monitor");
        println!(
            "I2C: {} @ 0x{:02X} | Auto-reconnect: ON",
            cli.get_i2c_device_path(),
            cli.i2c_address
        );
        println!();
    }

    // Initialize I2C and radar with GPIO pins
    let i2c_device = i2c::I2cDevice::new(&cli.get_i2c_device_path(), cli.i2c_address)?;
    let gpio_pins = cli.get_gpio_pins();
    let mut radar = XM125Radar::new(i2c_device, gpio_pins);

    // Initialize FIFO writer if enabled
    let mut fifo_writer = if cli.output.fifo_output {
        match FifoWriter::new(&cli.output.fifo_path, cli.output.fifo_interval) {
            Ok(writer) => {
                if cli.output.fifo_interval > 0.0 {
                    info!("FIFO output enabled: {} (format: {:?}, interval: {:.1}s - spi-lib compatible)", 
                          cli.output.fifo_path, cli.output.fifo_format, cli.output.fifo_interval);
                } else {
                    info!(
                        "FIFO output enabled: {} (format: {:?}, real-time mode)",
                        cli.output.fifo_path, cli.output.fifo_format
                    );
                }
                // Send startup status (same as spi-lib)
                let _ = writer.write_status("Starting up");
                Some(writer)
            }
            Err(e) => {
                warn!("Failed to initialize FIFO writer: {e}");
                None
            }
        }
    } else {
        None
    };

    // Execute the command
    execute_command(&cli, &mut radar, fifo_writer.as_mut()).await?;

    // Send exit status if FIFO is enabled
    if let Some(ref writer) = fifo_writer {
        let _ = writer.write_status("App exit");
    }

    Ok(())
}
