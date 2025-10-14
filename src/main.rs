/*
 * XM125 Radar Monitor - Embedded Rust application for XM125 radar module
 * Copyright (C) 2025 Dynamic Devices Ltd
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::Parser;
use log::{debug, error};
use std::process;

mod cli;
mod error;
mod i2c;
mod radar;

use cli::Cli;
use error::RadarError;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    if !cli.quiet {
        println!("{APP_NAME} v{VERSION}");
        println!("Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.");
        println!("XM125 Radar Module Monitor");
        println!();
    }

    if let Err(e) = run(cli).await {
        error!("Command failed: {e}");
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), RadarError> {
    debug!("Starting {APP_NAME} v{VERSION}");

    // Create I2C connection to XM125
    let i2c_device_path = cli.get_i2c_device_path();
    debug!(
        "Using I2C device: {} at address 0x{:02X}",
        i2c_device_path, cli.i2c_address
    );
    let i2c_device = i2c::I2cDevice::new(&i2c_device_path, cli.i2c_address)?;
    let mut radar = radar::XM125Radar::new(i2c_device);

    // Configure radar based on CLI options
    configure_radar_from_cli(&mut radar, &cli).await?;

    // Execute command
    if let Some(cmd) = &cli.command {
        debug!("Executing command: {cmd:?}");
        execute_command(cmd.clone(), &mut radar, &cli).await?;
        Ok(())
    } else {
        println!("No command provided. Use --help for usage information.");
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
async fn execute_command(
    command: cli::Commands,
    radar: &mut radar::XM125Radar,
    cli: &Cli,
) -> Result<(), RadarError> {
    use cli::Commands;

    match command {
        Commands::Status => {
            let status = radar.get_status()?;
            output_response(cli, "status", &status, "📊", "Radar Status")?;
        }
        Commands::Connect { force } => {
            if force {
                if cli.auto_reconnect {
                    radar.auto_connect().await?;
                } else {
                    radar.connect_async().await?;
                }
            } else {
                radar.connect()?;
            }
            output_response(cli, "connect", "Connected successfully", "🔗", "Connection")?;
        }
        Commands::Disconnect => {
            radar.disconnect();
            output_response(
                cli,
                "disconnect",
                "Disconnected successfully",
                "🔌",
                "Disconnection",
            )?;
        }
        Commands::Info => {
            let info = radar.get_info()?;
            output_response(cli, "info", &info, "ℹ️", "Device Information")?;
        }
        Commands::Measure => {
            let result = radar.measure_distance().await?;
            let response = format!(
                "Distance: {:.2}m, Strength: {:.1}dB, Temperature: {}°C",
                result.distance, result.strength, result.temperature
            );
            output_response(cli, "measure", &response, "📏", "Distance Measurement")?;
        }
        Commands::Calibrate => {
            radar.calibrate().await?;
            output_response(
                cli,
                "calibrate",
                "Calibration completed successfully",
                "🎯",
                "Calibration",
            )?;
        }
        Commands::Monitor {
            interval,
            count,
            save_to: _,
        } => {
            monitor_distances(radar, cli, interval, count).await?;
        }
        Commands::Presence => {
            let result = radar.measure_presence().await?;
            let response = format!(
                "Presence: {}, Distance: {:.2}m, Intra: {:.2}, Inter: {:.2}, Temperature: {}°C",
                if result.presence_detected {
                    "DETECTED"
                } else {
                    "NOT DETECTED"
                },
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score,
                result.temperature
            );
            output_response(cli, "presence", &response, "👁️", "Presence Detection")?;
        }
        Commands::Combined => {
            let result = radar.measure_combined().await?;
            let mut response_parts = Vec::new();

            if let Some(distance) = &result.distance {
                response_parts.push(format!(
                    "Distance: {:.2}m ({:.1}dB)",
                    distance.distance, distance.strength
                ));
            }

            if let Some(presence) = &result.presence {
                response_parts.push(format!(
                    "Presence: {} at {:.2}m (intra: {:.2}, inter: {:.2})",
                    if presence.presence_detected {
                        "DETECTED"
                    } else {
                        "NOT DETECTED"
                    },
                    presence.presence_distance,
                    presence.intra_presence_score,
                    presence.inter_presence_score
                ));
            }

            let response = response_parts.join(" | ");
            output_response(cli, "combined", &response, "🎯", "Combined Detection")?;
        }
        Commands::Config {
            start,
            length,
            presence_range,
            sensitivity,
            frame_rate,
        } => {
            configure_detector(
                radar,
                start,
                length,
                presence_range,
                sensitivity,
                frame_rate,
            );
            output_response(
                cli,
                "config",
                "Configuration updated successfully",
                "⚙️",
                "Configuration",
            )?;
        }
    }

    Ok(())
}

async fn monitor_distances(
    radar: &mut radar::XM125Radar,
    cli: &Cli,
    interval: u64,
    count: Option<u32>,
) -> Result<(), RadarError> {
    use tokio::time::{sleep, Duration};

    let mut measurement_count = 0;

    loop {
        let result = radar.measure_distance().await?;

        match cli.format {
            cli::OutputFormat::Human => {
                println!(
                    "[{}] Distance: {:.2}m | Strength: {:.1}dB | Temp: {}°C",
                    chrono::Utc::now().format("%H:%M:%S"),
                    result.distance,
                    result.strength,
                    result.temperature
                );
            }
            cli::OutputFormat::Json => {
                let json_response = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "distance_m": result.distance,
                    "strength_db": result.strength,
                    "temperature_c": result.temperature,
                    "measurement_count": measurement_count
                });
                println!("{}", serde_json::to_string(&json_response)?);
            }
            cli::OutputFormat::Csv => {
                if measurement_count == 0 {
                    println!("timestamp,distance_m,strength_db,temperature_c,measurement_count");
                }
                println!(
                    "{},{},{},{},{}",
                    chrono::Utc::now().to_rfc3339(),
                    result.distance,
                    result.strength,
                    result.temperature,
                    measurement_count
                );
            }
        }

        measurement_count += 1;

        if let Some(max_count) = count {
            if measurement_count >= max_count {
                break;
            }
        }

        sleep(Duration::from_millis(interval)).await;
    }

    Ok(())
}

fn output_response(
    cli: &Cli,
    command: &str,
    response: &str,
    emoji: &str,
    title: &str,
) -> Result<(), RadarError> {
    if cli.quiet {
        return Ok(());
    }

    match cli.format {
        cli::OutputFormat::Human => {
            println!("{emoji} {title}:");
            println!("{response}");
        }
        cli::OutputFormat::Json => {
            let json_response = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "command": command,
                "status": "success",
                "data": response
            });
            println!("{}", serde_json::to_string_pretty(&json_response)?);
        }
        cli::OutputFormat::Csv => {
            println!("timestamp,command,status,response");
            println!(
                "{},{},success,\"{}\"",
                chrono::Utc::now().to_rfc3339(),
                command,
                response.replace('"', "\"\"")
            );
        }
    }

    Ok(())
}

/// Configure radar based on CLI options
async fn configure_radar_from_cli(
    radar: &mut radar::XM125Radar,
    cli: &Cli,
) -> Result<(), RadarError> {
    // Convert CLI detector mode to radar detector mode
    let detector_mode = match cli.mode {
        cli::DetectorMode::Distance => radar::DetectorMode::Distance,
        cli::DetectorMode::Presence => radar::DetectorMode::Presence,
        cli::DetectorMode::Combined => radar::DetectorMode::Combined,
    };

    radar.set_detector_mode(detector_mode).await?;

    // Configure auto-reconnect
    let config = radar::XM125Config {
        detector_mode,
        auto_reconnect: cli.auto_reconnect,
        ..Default::default()
    };

    radar.set_config(config);
    Ok(())
}

/// Configure detector with new settings
fn configure_detector(
    radar: &mut radar::XM125Radar,
    start: Option<f32>,
    length: Option<f32>,
    presence_range: Option<cli::PresenceRange>,
    sensitivity: Option<f32>,
    frame_rate: Option<f32>,
) {
    let mut config = radar::XM125Config::default();

    if let Some(start) = start {
        config.start_m = start;
    }

    if let Some(length) = length {
        config.length_m = length;
    }

    if let Some(range) = presence_range {
        config.presence_range = match range {
            cli::PresenceRange::Short => radar::PresenceRange::Short,
            cli::PresenceRange::Medium => radar::PresenceRange::Medium,
            cli::PresenceRange::Long => radar::PresenceRange::Long,
        };
    }

    if let Some(sensitivity) = sensitivity {
        config.threshold_sensitivity = sensitivity;
    }

    if let Some(rate) = frame_rate {
        config.frame_rate = rate;
    }

    radar.set_config(config);
}
// Test comment
