//! Command execution logic
//!
//! This module handles the main command dispatch and execution logic,
//! coordinating between different measurement modes and output formats.

use crate::cli::{Cli, Commands};
use crate::config::{
    configure_distance_range, configure_presence_parameters, debug_registers_if_connected,
};
use crate::display::{
    display_distance_result, display_presence_result, write_distance_to_fifo,
    write_presence_to_fifo,
};
use crate::error::RadarError;
use crate::fifo::FifoWriter;
use crate::handlers::handle_firmware_action;
use crate::monitoring::{monitor_distance_continuous, monitor_presence_continuous};
use crate::radar::{DetectorMode, XM125Radar};

/// Execute the main command logic
#[allow(clippy::too_many_lines)]
pub async fn execute_command(
    cli: &Cli,
    radar: &mut XM125Radar,
    fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    match &cli.command {
        Commands::Status => {
            let status = radar.get_status()?;
            match cli.format {
                crate::cli::OutputFormat::Json => {
                    let status_obj = serde_json::json!({ "status": status });
                    println!("{}", serde_json::to_string_pretty(&status_obj)?);
                }
                crate::cli::OutputFormat::Csv => {
                    println!("status");
                    println!("{status}");
                }
                crate::cli::OutputFormat::Human => {
                    println!("📡 XM125 Status: {status}");
                }
            }
        }

        Commands::Info => {
            let info = radar.get_info()?;
            match cli.format {
                crate::cli::OutputFormat::Json => {
                    let info_obj = serde_json::json!({ "info": info });
                    println!("{}", serde_json::to_string_pretty(&info_obj)?);
                }
                crate::cli::OutputFormat::Csv => {
                    println!("info");
                    println!("{info}");
                }
                crate::cli::OutputFormat::Human => {
                    println!("🔍 XM125 Device Information:");
                    println!("{info}");
                }
            }
        }

        Commands::Distance {
            range,
            continuous,
            count,
            interval,
            save_to,
        } => {
            // Ensure device is in distance mode
            radar.set_detector_mode(DetectorMode::Distance);

            // Configure range if specified
            if let Some(range_str) = range {
                configure_distance_range(radar, range_str)?;
            }

            // Debug registers if requested (global option)
            if cli.debug_registers {
                debug_registers_if_connected(radar, "Distance");
            }

            if *continuous {
                monitor_distance_continuous(
                    radar,
                    cli,
                    *count,
                    *interval,
                    save_to.as_deref(),
                    fifo_writer,
                )
                .await?;
            } else {
                let result = radar.measure_distance().await?;
                display_distance_result(&result, &cli.format);

                // Single measurement FIFO output
                if let Some(writer) = fifo_writer {
                    write_distance_to_fifo(writer, &result, &cli.fifo_format);
                }
            }
        }

        Commands::Presence {
            range,
            min_range,
            max_range,
            sensitivity,
            frame_rate,
            profile,
            continuous,
            count,
            interval,
            save_to,
        } => {
            // Ensure device is in presence mode
            radar.set_detector_mode(DetectorMode::Presence);

            // Configure presence parameters
            configure_presence_parameters(
                radar,
                range.as_ref(),
                *min_range,
                *max_range,
                *sensitivity,
                *frame_rate,
                profile,
            )?;

            // Debug registers if requested (global option)
            if cli.debug_registers {
                debug_registers_if_connected(radar, "Presence");
            }

            if *continuous {
                monitor_presence_continuous(
                    radar,
                    cli,
                    *count,
                    *interval,
                    save_to.as_deref(),
                    fifo_writer,
                )
                .await?;
            } else {
                let result = radar.measure_presence().await?;
                display_presence_result(&result, &cli.format);

                // Single measurement FIFO output
                if let Some(writer) = fifo_writer {
                    write_presence_to_fifo(writer, &result, &cli.fifo_format);
                }
            }
        }

        Commands::Firmware { action } => {
            handle_firmware_action(radar, action, &cli.firmware_path).await?;
        }

        Commands::Gpio { .. } => {
            // GPIO commands are handled earlier, this should not be reached
            unreachable!("GPIO commands should be handled before I2C initialization");
        }
    }
    Ok(())
}
