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
use log::{debug, error, info, warn};
use std::process;

mod cli;
mod device_manager;
mod error;
mod firmware;
mod i2c;
mod radar;

use cli::{Cli, FirmwareAction};
use device_manager::DeviceManager;
use error::RadarError;
use firmware::FirmwareManager;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn print_banner(cli: &Cli) {
    println!("{APP_NAME} v{VERSION}");
    println!("Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.");
    println!("XM125 Radar Module Monitor");

    // Show key configuration at startup
    let mode_str = match cli.mode {
        cli::DetectorMode::Distance => "Distance",
        cli::DetectorMode::Presence => "Presence",
        cli::DetectorMode::Combined => "Combined",
        cli::DetectorMode::Breathing => "Breathing",
    };
    println!(
        "Mode: {} | I2C: {} @ 0x{:02X} | Auto-reconnect: {}",
        mode_str,
        cli.get_i2c_device_path(),
        cli.i2c_address,
        if cli.auto_reconnect && !cli.no_auto_reconnect {
            "ON"
        } else {
            "OFF"
        }
    );
    println!();
}

fn print_banner_with_detected_mode(cli: &Cli, detected_mode: Option<&str>) {
    println!("{APP_NAME} v{VERSION}");
    println!("Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.");
    println!("XM125 Radar Module Monitor");

    // Show detected mode if available, otherwise CLI mode
    let mode_str = if let Some(detected) = detected_mode {
        detected
    } else {
        match cli.mode {
            cli::DetectorMode::Distance => "Distance",
            cli::DetectorMode::Presence => "Presence",
            cli::DetectorMode::Combined => "Combined",
            cli::DetectorMode::Breathing => "Breathing",
        }
    };

    println!(
        "Mode: {} | I2C: {} @ 0x{:02X} | Auto-reconnect: {}",
        mode_str,
        cli.get_i2c_device_path(),
        cli.i2c_address,
        if cli.auto_reconnect && !cli.no_auto_reconnect {
            "ON"
        } else {
            "OFF"
        }
    );
    println!();
}

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

    // Don't print banner here for status command - it will print its own with detected mode
    let is_status_command = matches!(cli.command, Some(cli::Commands::Status));
    if !cli.quiet && !is_status_command {
        print_banner(&cli);
    }

    if let Err(e) = run(cli).await {
        error!("Command failed: {e}");
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

#[allow(clippy::too_many_lines)] // Complex main function with comprehensive error handling
async fn run(cli: Cli) -> Result<(), RadarError> {
    debug!("Starting {APP_NAME} v{VERSION}");

    // Create I2C connection to XM125
    let i2c_device_path = cli.get_i2c_device_path();
    debug!(
        "Using I2C device: {} at address 0x{:02X}",
        i2c_device_path, cli.i2c_address
    );
    let mut i2c_device = i2c::I2cDevice::new(&i2c_device_path, cli.i2c_address)?;

    // Configure GPIO pins if provided
    if cli.wakeup_pin.is_some() || cli.int_pin.is_some() {
        debug!(
            "Configuring GPIO pins: WAKEUP={:?}, INT={:?}",
            cli.wakeup_pin, cli.int_pin
        );
        i2c_device.configure_gpio(cli.wakeup_pin, cli.int_pin)?;
    }

    let mut radar = radar::XM125Radar::new(i2c_device);

    // Create firmware manager
    let firmware_manager =
        FirmwareManager::new(&cli.firmware_path, &cli.control_script, cli.i2c_address);

    // Check if firmware update is needed (if auto-update is enabled)
    if cli.auto_update_firmware {
        let desired_firmware_type = firmware::FirmwareType::from(cli.mode.clone());

        // Try to read current firmware info
        match get_current_firmware_info(&mut radar) {
            Ok(current_app_id) => {
                if firmware_manager.firmware_update_needed(current_app_id, desired_firmware_type)? {
                    info!("Firmware update required for {:?} mode", cli.mode);
                    firmware_manager
                        .update_firmware_with_verification(
                            desired_firmware_type,
                            cli.auto_verify_firmware,
                        )
                        .await?;

                    // Recreate radar instance after firmware update
                    let i2c_device_path = cli.get_i2c_device_path();
                    let mut i2c_device = i2c::I2cDevice::new(&i2c_device_path, cli.i2c_address)?;
                    if cli.wakeup_pin.is_some() || cli.int_pin.is_some() {
                        i2c_device.configure_gpio(cli.wakeup_pin, cli.int_pin)?;
                    }
                    radar = radar::XM125Radar::new(i2c_device);
                }
            }
            Err(e) => {
                warn!("Could not check current firmware: {e}");

                // Check if this might be an unprogrammed module before trying firmware update
                if is_likely_unprogrammed_module(&cli) {
                    print_unprogrammed_module_help(&cli);
                    return Ok(());
                }

                // Try firmware update if control script exists
                if std::path::Path::new(&cli.control_script).exists() {
                    info!("Proceeding with firmware update due to communication issues");
                    match firmware_manager
                        .update_firmware_with_verification(
                            desired_firmware_type,
                            cli.auto_verify_firmware,
                        )
                        .await
                    {
                        Ok(()) => {
                            // Recreate radar instance after firmware update
                            let i2c_device_path = cli.get_i2c_device_path();
                            let mut i2c_device =
                                i2c::I2cDevice::new(&i2c_device_path, cli.i2c_address)?;
                            if cli.wakeup_pin.is_some() || cli.int_pin.is_some() {
                                i2c_device.configure_gpio(cli.wakeup_pin, cli.int_pin)?;
                            }
                            radar = radar::XM125Radar::new(i2c_device);
                        }
                        Err(firmware_err) => {
                            warn!("Firmware update failed: {firmware_err}");
                            print_unprogrammed_module_help(&cli);
                            return Ok(());
                        }
                    }
                } else {
                    warn!("Control script not found at: {}", cli.control_script);
                    print_unprogrammed_module_help(&cli);
                    return Ok(());
                }
            }
        }
    }

    // Configure radar based on CLI options
    configure_radar_from_cli(&mut radar, &cli).await?;

    // Execute command
    if let Some(cmd) = &cli.command {
        debug!("Executing command: {cmd:?}");

        // Handle unprogrammed modules gracefully
        match execute_command(cmd.clone(), &mut radar, &cli).await {
            Err(RadarError::I2c(_) | RadarError::NotConnected) => {
                // Check if this might be an unprogrammed module
                if is_likely_unprogrammed_module(&cli) {
                    print_unprogrammed_module_help(&cli);
                    Ok(())
                } else {
                    Err(RadarError::NotConnected)
                }
            }
            result => result,
        }
    } else {
        // No command provided - show help instead of trying to access I2C
        if !cli.quiet {
            println!("No command provided.\n");
        }

        // Print help using clap's built-in help
        let mut cmd = {
            use clap::CommandFactory;
            Cli::command()
        };
        cmd.print_help()?;
        println!(); // Add newline after help
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
            // For status command, detect actual firmware mode and show that instead of CLI default
            let detected_mode = match get_current_firmware_info(radar) {
                Ok(app_id) => {
                    let firmware_type = firmware::FirmwareType::from_app_id(app_id);
                    Some(firmware_type.display_name())
                }
                Err(_) => None, // If we can't detect, fall back to CLI mode
            };

            // Print banner with detected mode for status command
            if !cli.quiet {
                print_banner_with_detected_mode(cli, detected_mode);
            }

            let status = radar.get_status()?;
            output_response(cli, "status", &status, "📊", "Radar Status")?;
        }
        Commands::Connect { force } => {
            let auto_reconnect = cli.auto_reconnect && !cli.no_auto_reconnect;
            if force {
                if auto_reconnect {
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
            monitor_measurements(radar, cli, interval, count).await?;
        }
        Commands::Presence => {
            let result = radar.measure_presence().await?;
            let response = format!(
                "Presence: {}, Distance: {:.2}m, Intra: {:.2}, Inter: {:.2}",
                if result.presence_detected {
                    "DETECTED"
                } else {
                    "NOT DETECTED"
                },
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score
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
        Commands::Breathing => {
            let result = radar.measure_breathing().await?;
            let response = format!(
                "State: {}, Rate: {:.1} BPM, Ready: {}, Temperature: {}°C",
                result.app_state.display_name(),
                result.breathing_rate,
                if result.result_ready { "YES" } else { "NO" },
                result.temperature
            );
            output_response(cli, "breathing", &response, "🫁", "Breathing Detection")?;
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
        Commands::Firmware { action } => {
            handle_firmware_command(action, cli).await?;
        }
        Commands::Bootloader { reset } => {
            handle_bootloader_command(reset, cli)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)] // Complex monitoring function with multiple output formats
async fn monitor_measurements(
    radar: &mut radar::XM125Radar,
    cli: &Cli,
    interval: u64,
    count: Option<u32>,
) -> Result<(), RadarError> {
    use tokio::time::{sleep, Duration};

    let mut measurement_count = 0;

    loop {
        // Choose measurement type based on detector mode
        match cli.mode {
            cli::DetectorMode::Distance => {
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
                            println!(
                                "timestamp,distance_m,strength_db,temperature_c,measurement_count"
                            );
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
            }
            cli::DetectorMode::Presence => {
                let result = radar.measure_presence().await?;
                match cli.format {
                    cli::OutputFormat::Human => {
                        println!(
                            "[{}] Presence: {} | Distance: {:.2}m | Intra: {:.2} | Inter: {:.2}",
                            chrono::Utc::now().format("%H:%M:%S"),
                            if result.presence_detected {
                                "DETECTED"
                            } else {
                                "NOT DETECTED"
                            },
                            result.presence_distance,
                            result.intra_presence_score,
                            result.inter_presence_score
                        );
                    }
                    cli::OutputFormat::Json => {
                        let json_response = serde_json::json!({
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "presence_detected": result.presence_detected,
                            "presence_distance_m": result.presence_distance,
                            "intra_presence_score": result.intra_presence_score,
                            "inter_presence_score": result.inter_presence_score,
                            "measurement_count": measurement_count
                        });
                        println!("{}", serde_json::to_string(&json_response)?);
                    }
                    cli::OutputFormat::Csv => {
                        if measurement_count == 0 {
                            println!("timestamp,presence_detected,presence_distance_m,intra_score,inter_score,measurement_count");
                        }
                        println!(
                            "{},{},{:.2},{:.2},{:.2},{}",
                            chrono::Utc::now().to_rfc3339(),
                            if result.presence_detected { "1" } else { "0" },
                            result.presence_distance,
                            result.intra_presence_score,
                            result.inter_presence_score,
                            measurement_count
                        );
                    }
                }
            }
            cli::DetectorMode::Combined => {
                let result = radar.measure_combined().await?;
                match cli.format {
                    cli::OutputFormat::Human => {
                        let mut parts = Vec::new();
                        if let Some(distance) = &result.distance {
                            parts.push(format!(
                                "Distance: {:.2}m ({:.1}dB)",
                                distance.distance, distance.strength
                            ));
                        }
                        if let Some(presence) = &result.presence {
                            parts.push(format!(
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
                        println!(
                            "[{}] {}",
                            chrono::Utc::now().format("%H:%M:%S"),
                            parts.join(" | ")
                        );
                    }
                    cli::OutputFormat::Json => {
                        println!("{}", serde_json::to_string(&result)?);
                    }
                    cli::OutputFormat::Csv => {
                        if measurement_count == 0 {
                            println!("timestamp,distance_m,strength_db,presence_detected,presence_distance_m,intra_score,inter_score,temperature_c,measurement_count");
                        }
                        let distance_str = if let Some(d) = &result.distance {
                            format!("{:.2},{:.1},{}", d.distance, d.strength, d.temperature)
                        } else {
                            ",,".to_string()
                        };
                        let presence_str = if let Some(p) = &result.presence {
                            format!(
                                "{},{:.2},{:.2},{:.2}",
                                if p.presence_detected { "1" } else { "0" },
                                p.presence_distance,
                                p.intra_presence_score,
                                p.inter_presence_score
                            )
                        } else {
                            ",,,".to_string()
                        };
                        println!(
                            "{},{},{},{}",
                            chrono::Utc::now().to_rfc3339(),
                            distance_str,
                            presence_str,
                            measurement_count
                        );
                    }
                }
            }
            cli::DetectorMode::Breathing => {
                let result = radar.measure_breathing().await?;
                match cli.format {
                    cli::OutputFormat::Human => {
                        println!(
                            "[{}] State: {} | Rate: {:.1} BPM | Ready: {} | Temp: {}°C",
                            chrono::Utc::now().format("%H:%M:%S"),
                            result.app_state.display_name(),
                            result.breathing_rate,
                            if result.result_ready { "YES" } else { "NO" },
                            result.temperature
                        );
                    }
                    cli::OutputFormat::Json => {
                        let json_response = serde_json::json!({
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "app_state": result.app_state.display_name(),
                            "breathing_rate_bpm": result.breathing_rate,
                            "result_ready": result.result_ready,
                            "temperature_c": result.temperature,
                            "measurement_count": measurement_count
                        });
                        println!("{}", serde_json::to_string(&json_response)?);
                    }
                    cli::OutputFormat::Csv => {
                        if measurement_count == 0 {
                            println!("timestamp,app_state,breathing_rate_bpm,result_ready,temperature_c,measurement_count");
                        }
                        println!(
                            "{},{},{:.1},{},{},{}",
                            chrono::Utc::now().to_rfc3339(),
                            result.app_state.display_name(),
                            result.breathing_rate,
                            if result.result_ready { "1" } else { "0" },
                            result.temperature,
                            measurement_count
                        );
                    }
                }
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
        cli::DetectorMode::Breathing => radar::DetectorMode::Breathing,
    };

    radar.set_detector_mode(detector_mode).await?;

    // Configure auto-reconnect (default true, unless --no-auto-reconnect is specified)
    let auto_reconnect = cli.auto_reconnect && !cli.no_auto_reconnect;
    let config = radar::XM125Config {
        detector_mode,
        auto_reconnect,
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

/// Handle firmware management commands
#[allow(clippy::too_many_lines)] // Complex firmware management requires detailed handling
async fn handle_firmware_command(action: FirmwareAction, cli: &Cli) -> Result<(), RadarError> {
    let firmware_manager =
        FirmwareManager::new(&cli.firmware_path, &cli.control_script, cli.i2c_address);

    match action {
        FirmwareAction::Check => {
            // Use the device manager for basic device presence check
            let device_manager = DeviceManager::new(
                cli.get_i2c_device_path(),
                cli.i2c_address,
                cli.firmware_path.clone(),
                cli.control_script.clone(),
            );

            let state = device_manager.check_device_presence().await;

            if !state.is_present {
                let error_msg = "XM125 device not found on I2C bus. Use 'xm125-control.sh --reset-run' to reset device.";
                output_response(cli, "device_not_found", error_msg, "❌", "Device Not Found")?;
                return Ok(());
            }

            // Device is present - now read firmware info directly using firmware manager
            let firmware_manager =
                FirmwareManager::new(&cli.firmware_path, &cli.control_script, cli.i2c_address);

            // Try to read current firmware info
            match get_current_firmware_info_for_check(cli) {
                Ok((app_id, firmware_type)) => {
                    let response = format!(
                        "Current firmware: {} (App ID: {})",
                        firmware_type.display_name(),
                        app_id
                    );
                    output_response(cli, "firmware_check", &response, "✅", "Firmware Check")?;

                    // Try to get checksum
                    match firmware_manager.get_firmware_checksum(firmware_type) {
                        Ok(checksum) => {
                            let checksum_info = format!("Firmware checksum: {checksum}");
                            output_response(
                                cli,
                                "firmware_checksum",
                                &checksum_info,
                                "🔐",
                                "Checksum",
                            )?;
                        }
                        Err(e) => {
                            debug!("Could not read firmware checksum: {e}");
                            // Don't show this as an error to users - checksums are optional
                        }
                    }
                }
                Err(e) => {
                    let warning_msg =
                        format!("XM125 device present but could not read firmware info: {e}");
                    output_response(
                        cli,
                        "device_unresponsive",
                        &warning_msg,
                        "⚠️",
                        "Device Communication Error",
                    )?;
                }
            }
        }

        FirmwareAction::Update {
            firmware_type,
            force,
            verify,
        } => {
            let target_firmware = firmware::FirmwareType::from(firmware_type);

            // Use the new device manager for clean firmware updating
            let device_manager = DeviceManager::new(
                cli.get_i2c_device_path(),
                cli.i2c_address,
                cli.firmware_path.clone(),
                cli.control_script.clone(),
            );

            // Check current state first
            let state = device_manager.check_device_presence().await;

            if !state.is_present {
                let error_msg = "XM125 device not found on I2C bus. Use 'xm125-control.sh --reset-run' to reset device.";
                output_response(cli, "device_not_found", error_msg, "❌", "Device Not Found")?;
                return Ok(());
            }

            // Check if update is needed (unless forced)
            if !force {
                if let Some(current_type) = state.firmware_type {
                    if current_type == target_firmware {
                        let msg = format!(
                            "✅ Firmware already matches {} - no update needed",
                            target_firmware.display_name()
                        );
                        output_response(cli, "firmware_update", &msg, "✅", "Firmware Update")?;
                        return Ok(());
                    }
                }
            }

            // Perform the firmware update
            info!(
                "🚀 Updating firmware to {} (forced: {}, verify: {})",
                target_firmware.display_name(),
                force,
                verify
            );

            match device_manager
                .update_firmware(target_firmware, verify)
                .await
            {
                Ok(()) => {
                    let success_msg = format!(
                        "✅ Successfully updated firmware to {}",
                        target_firmware.display_name()
                    );
                    output_response(
                        cli,
                        "firmware_update",
                        &success_msg,
                        "🚀",
                        "Firmware Update",
                    )?;
                }
                Err(e) => {
                    let error_msg = format!("❌ Firmware update failed: {e}");
                    output_response(cli, "firmware_error", &error_msg, "❌", "Firmware Error")?;
                    return Err(e);
                }
            }
        }

        FirmwareAction::Verify { firmware_type } => {
            #[allow(clippy::single_match_else)]
            // Complex verification logic requires match structure
            match firmware_type {
                Some(fw_type) => {
                    let target_firmware = firmware::FirmwareType::from(fw_type);

                    // Compare device checksum with binary checksum
                    match firmware_manager.get_firmware_checksum(target_firmware) {
                        Ok(device_checksum) => {
                            match firmware_manager.calculate_binary_checksum(target_firmware) {
                                Ok(binary_checksum) => {
                                    if device_checksum == binary_checksum {
                                        let msg = format!("✅ Firmware verification PASSED for {}\nDevice: {}\nBinary: {}", 
                                                         target_firmware.display_name(), device_checksum, binary_checksum);
                                        output_response(
                                            cli,
                                            "firmware_verify",
                                            &msg,
                                            "✅",
                                            "Verification",
                                        )?;
                                    } else {
                                        let msg = format!("❌ Firmware verification FAILED for {}\nDevice: {}\nBinary: {}", 
                                                         target_firmware.display_name(), device_checksum, binary_checksum);
                                        output_response(
                                            cli,
                                            "firmware_verify",
                                            &msg,
                                            "❌",
                                            "Verification",
                                        )?;
                                    }
                                }
                                Err(e) => {
                                    let msg = format!("Could not calculate binary checksum: {e}");
                                    output_response(
                                        cli,
                                        "firmware_verify",
                                        &msg,
                                        "❌",
                                        "Verification Error",
                                    )?;
                                }
                            }
                        }
                        Err(e) => {
                            let msg = format!("Could not read device checksum: {e}");
                            output_response(
                                cli,
                                "firmware_verify",
                                &msg,
                                "❌",
                                "Verification Error",
                            )?;
                        }
                    }
                }
                None => {
                    // Verify all available firmware types
                    let firmware_types = [
                        firmware::FirmwareType::Distance,
                        firmware::FirmwareType::Presence,
                        firmware::FirmwareType::Breathing,
                    ];

                    for fw_type in &firmware_types {
                        match firmware_manager.calculate_binary_checksum(*fw_type) {
                            Ok(checksum) => {
                                let msg = format!("{}: {}", fw_type.display_name(), checksum);
                                output_response(
                                    cli,
                                    "firmware_checksums",
                                    &msg,
                                    "🔐",
                                    "Binary Checksums",
                                )?;
                            }
                            Err(e) => {
                                warn!(
                                    "Could not calculate checksum for {}: {}",
                                    fw_type.display_name(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get current firmware application ID
fn get_current_firmware_info(radar: &mut radar::XM125Radar) -> Result<u32, RadarError> {
    // Try to connect and read application ID
    match radar.connect() {
        Ok(()) => {
            // Read the application ID register directly
            let app_id_data = radar.read_application_id()?;
            Ok(app_id_data)
        }
        Err(e) => {
            warn!("Could not connect to read firmware info: {e}");
            Err(e)
        }
    }
}

/// Check if the module is likely unprogrammed by trying to detect it on I2C
fn is_likely_unprogrammed_module(cli: &Cli) -> bool {
    // Try to detect the module on the I2C bus using i2cdetect
    let i2c_bus = cli.i2c_bus;
    let address = cli.i2c_address;

    // Check if i2cdetect is available
    if std::process::Command::new("i2cdetect")
        .arg("--help")
        .output()
        .map(|output| !output.status.success())
        .unwrap_or(true)
    {
        // i2cdetect not available, assume it might be unprogrammed
        return true;
    }

    // Check both run mode (0x52) and bootloader mode (0x48) addresses
    let run_mode_detected = std::process::Command::new("i2cdetect")
        .args([
            "-y",
            &i2c_bus.to_string(),
            &format!("0x{address:02x}"),
            &format!("0x{address:02x}"),
        ])
        .output()
        .map(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(&format!("{address:02x}"))
        })
        .unwrap_or(false);

    let bootloader_detected = std::process::Command::new("i2cdetect")
        .args(["-y", &i2c_bus.to_string(), "0x48", "0x48"])
        .output()
        .map(|output| {
            output.status.success() && String::from_utf8_lossy(&output.stdout).contains("48")
        })
        .unwrap_or(false);

    // If neither address responds, likely unprogrammed or not present
    !run_mode_detected && !bootloader_detected
}

/// Print helpful instructions for unprogrammed modules
fn print_unprogrammed_module_help(cli: &Cli) {
    println!("❌ XM125 Module Not Found or Not Programmed");
    println!();
    println!(
        "The XM125 radar module is not responding on I2C bus {} at address 0x{:02X}.",
        cli.i2c_bus, cli.i2c_address
    );
    println!();
    println!("🔧 Possible Solutions:");
    println!();
    println!("1. **Check Hardware Connection:**");
    println!("   - Verify I2C connections (SDA, SCL, GND, VCC)");
    println!("   - Ensure module is powered (3.3V)");
    println!("   - Check I2C bus number: {}", cli.i2c_bus);
    println!();
    println!("2. **Reset Module to Run Mode:**");
    println!("   sudo {} --reset-run", cli.control_script);
    println!();
    println!("3. **Program Module Firmware:**");
    println!("   If this is a new/blank module, program it with presence firmware:");
    println!("   sudo {APP_NAME} firmware update presence");
    println!();
    println!("4. **Scan I2C Bus:**");
    println!("   sudo i2cdetect -y {}", cli.i2c_bus);
    println!("   Look for devices at 0x48 (bootloader) or 0x52 (run mode)");
    println!();
    println!("5. **Check Permissions:**");
    println!("   Ensure you're running with sudo for I2C access:");
    println!("   sudo {APP_NAME} status");
    println!();
    println!("💡 For new modules, start with step 3 to program the firmware.");
}

/// Get current firmware info for firmware check command
fn get_current_firmware_info_for_check(
    cli: &Cli,
) -> Result<(u32, firmware::FirmwareType), RadarError> {
    // Create I2C device and radar instance for reading firmware info
    let i2c_device_path = cli.get_i2c_device_path();
    let mut i2c_device = i2c::I2cDevice::new(&i2c_device_path, cli.i2c_address)?;

    // Configure GPIO if pins are specified
    if cli.wakeup_pin.is_some() || cli.int_pin.is_some() {
        i2c_device.configure_gpio(cli.wakeup_pin, cli.int_pin)?;
    }

    let mut radar = radar::XM125Radar::new(i2c_device);

    // Try to connect and read application ID
    match radar.connect() {
        Ok(()) => {
            let app_id = radar.read_application_id()?;
            let firmware_type = firmware::FirmwareType::from_app_id(app_id);
            Ok((app_id, firmware_type))
        }
        Err(e) => {
            warn!("Could not connect to read firmware info: {e}");
            Err(e)
        }
    }
}

/// Handle bootloader command to put XM125 into bootloader mode
fn handle_bootloader_command(reset: bool, cli: &Cli) -> Result<(), RadarError> {
    // Check if control script exists
    if !std::path::Path::new(&cli.control_script).exists() {
        let error_msg = format!(
            "Control script not found: {}\nBootloader mode requires GPIO control via the control script.",
            cli.control_script
        );
        output_response(
            cli,
            "bootloader_error",
            &error_msg,
            "❌",
            "Bootloader Error",
        )?;
        return Err(RadarError::DeviceError {
            message: "Control script not available".to_string(),
        });
    }

    info!("Putting XM125 module into bootloader mode...");

    // Use the control script to put device into bootloader mode
    let output = std::process::Command::new(&cli.control_script)
        .arg("--bootloader")
        .output()
        .map_err(|e| RadarError::DeviceError {
            message: format!("Failed to execute control script: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Failed to enter bootloader mode: {stderr}");
        output_response(
            cli,
            "bootloader_error",
            &error_msg,
            "❌",
            "Bootloader Error",
        )?;
        return Err(RadarError::DeviceError { message: error_msg });
    }

    let success_msg = "✅ XM125 module is now in bootloader mode (I2C address 0x48)\n\
         Use 'stm32flash' or firmware update commands to program the module.\n\
         The module will remain in bootloader mode until reset or power cycled."
        .to_string();
    output_response(cli, "bootloader", &success_msg, "🔧", "Bootloader Mode")?;

    // If reset flag is set, reset back to run mode
    if reset {
        info!("Resetting module back to run mode...");

        let reset_output = std::process::Command::new(&cli.control_script)
            .arg("--reset-run")
            .output()
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to execute reset command: {e}"),
            })?;

        if reset_output.status.success() {
            let reset_msg = "✅ Module reset to run mode (I2C address 0x52)";
            output_response(cli, "reset", reset_msg, "🔄", "Reset Complete")?;
        } else {
            let stderr = String::from_utf8_lossy(&reset_output.stderr);
            let warning_msg = format!("⚠️  Reset to run mode failed: {stderr}");
            output_response(cli, "reset_warning", &warning_msg, "⚠️", "Reset Warning")?;
        }
    }

    Ok(())
}
