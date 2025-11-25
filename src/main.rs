#![allow(dead_code)] // Allow dead code during restructure

use chrono::Utc;
use clap::Parser;
use log::{error, info, warn};
use std::env;
use std::process;

mod cli;
mod error;
mod fifo;
mod firmware;
mod gpio;
mod i2c;
mod radar;

use cli::{Cli, Commands, FirmwareAction, GpioAction, PresenceRange, ProfileMode};
use error::RadarError;
use fifo::FifoWriter;
use gpio::XM125GpioController;
use radar::XM125Radar;

/// Application entry point
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
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
    if !cli.quiet {
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
    let mut fifo_writer = if cli.fifo_output {
        match FifoWriter::new(&cli.fifo_path, cli.fifo_interval) {
            Ok(writer) => {
                if cli.fifo_interval > 0.0 {
                    info!("FIFO output enabled: {} (format: {:?}, interval: {:.1}s - spi-lib compatible)", 
                          cli.fifo_path, cli.fifo_format, cli.fifo_interval);
                } else {
                    info!(
                        "FIFO output enabled: {} (format: {:?}, real-time mode)",
                        cli.fifo_path, cli.fifo_format
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

/// Execute the main command logic
#[allow(clippy::too_many_lines)]
async fn execute_command(
    cli: &Cli,
    radar: &mut XM125Radar,
    fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    match &cli.command {
        Commands::Status => {
            let status = radar.get_status()?;
            match cli.format {
                cli::OutputFormat::Json => {
                    let status_obj = serde_json::json!({ "status": status });
                    println!("{}", serde_json::to_string_pretty(&status_obj)?);
                }
                cli::OutputFormat::Csv => {
                    println!("status");
                    println!("{status}");
                }
                cli::OutputFormat::Human => {
                    println!("📡 XM125 Status: {status}");
                }
            }
        }

        Commands::Info => {
            let info = radar.get_info()?;
            match cli.format {
                cli::OutputFormat::Json => {
                    let info_obj = serde_json::json!({ "info": info });
                    println!("{}", serde_json::to_string_pretty(&info_obj)?);
                }
                cli::OutputFormat::Csv => {
                    println!("info");
                    println!("{info}");
                }
                cli::OutputFormat::Human => {
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
            radar.set_detector_mode(radar::DetectorMode::Distance);

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
            radar.set_detector_mode(radar::DetectorMode::Presence);

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

/// Configure distance measurement range
fn configure_distance_range(radar: &mut XM125Radar, range_str: &str) -> Result<(), RadarError> {
    let parts: Vec<&str> = range_str.split(':').collect();
    if parts.len() != 2 {
        return Err(RadarError::DeviceError {
            message: format!(
                "Invalid range format '{range_str}'. Use 'start:end' (e.g., '0.1:3.0')"
            ),
        });
    }

    let start: f32 = parts[0].parse().map_err(|_| RadarError::DeviceError {
        message: format!("Invalid start range: {}", parts[0]),
    })?;

    let end: f32 = parts[1].parse().map_err(|_| RadarError::DeviceError {
        message: format!("Invalid end range: {}", parts[1]),
    })?;

    if start >= end {
        return Err(RadarError::DeviceError {
            message: format!("Start range ({start}) must be less than end range ({end})"),
        });
    }

    if start < 0.1 || end > 3.0 {
        return Err(RadarError::DeviceError {
            message: "Distance range must be between 0.1m and 3.0m".to_string(),
        });
    }

    info!("🎯 Configuring distance range: {start:.2}m - {end:.2}m");
    radar.config.start_m = start;
    radar.config.length_m = end - start;
    Ok(())
}

/// Configure presence parameters for the radar
#[allow(unused_assignments)]
fn configure_presence_parameters(
    radar: &mut radar::XM125Radar,
    presence_range: Option<&PresenceRange>,
    min_range: Option<f32>,
    max_range: Option<f32>,
    sensitivity: Option<f32>,
    frame_rate: Option<f32>,
    profile: &ProfileMode,
) -> Result<(), RadarError> {
    let mut config_changed = false;

    // Configure range (either preset or custom)
    if let Some(range) = presence_range {
        info!("🎯 Configuring presence range preset: {range:?}");
        radar.config.presence_range = range.clone().into();
        config_changed = true;
    } else if let (Some(min), Some(max)) = (min_range, max_range) {
        info!("🎯 Configuring custom presence range: {min:.2}m - {max:.2}m");

        // Validate range
        if min >= max {
            return Err(RadarError::DeviceError {
                message: format!(
                    "Minimum range ({min:.2}m) must be less than maximum range ({max:.2}m)"
                ),
            });
        }

        if min < 0.06 || max > 7.0 {
            return Err(RadarError::DeviceError {
                message: "Presence range must be between 0.06m and 7.0m".to_string(),
            });
        }

        // Set custom range (this will be used by configure_presence_range)
        radar.config.start_m = min;
        radar.config.length_m = max - min;
        config_changed = true;
    }

    // Configure sensitivity
    if let Some(sens) = sensitivity {
        info!("🎯 Configuring sensitivity: {sens:.2}");

        if !(0.1..=5.0).contains(&sens) {
            return Err(RadarError::DeviceError {
                message: format!("Sensitivity must be between 0.1 and 5.0 (got {sens:.2})"),
            });
        }

        // Convert sensitivity to internal threshold values
        radar.config.intra_detection_threshold = sens * 1000.0; // Convert to internal units
        radar.config.inter_detection_threshold = sens * 800.0; // Slightly lower for inter
        config_changed = true;
    }

    // Configure frame rate
    if let Some(rate) = frame_rate {
        info!("🎯 Configuring frame rate: {rate:.1} Hz");

        if !(1.0..=60.0).contains(&rate) {
            return Err(RadarError::DeviceError {
                message: format!("Frame rate must be between 1.0 and 60.0 Hz (got {rate:.1})"),
            });
        }

        radar.config.frame_rate = rate;
        config_changed = true;
    }

    // Configure profile mode
    match profile {
        ProfileMode::Auto => {
            radar.config.auto_profile_enabled = true;
            info!("🔧 Using automatic profile selection (recommended)");
        }
        ProfileMode::Manual => {
            radar.config.auto_profile_enabled = false;
            info!("🔧 Using manual profile selection (Profile 5 for 7m range)");
        }
    }
    config_changed = true; // Profile setting always triggers config change

    // Apply configuration to hardware if anything changed OR if no range was specified
    // (to ensure default long range is properly applied)
    if config_changed || (presence_range.is_none() && min_range.is_none() && max_range.is_none()) {
        radar.configure_presence_range()?;
        if config_changed {
            info!("✅ Presence parameters configured successfully");
        } else {
            info!("✅ Applied default presence configuration (long range: 0.5m - 7.0m)");
        }
    Ok(())
    }
}

/// Debug registers if radar is connected, with automatic connection attempt
fn debug_registers_if_connected(radar: &mut XM125Radar, mode: &str) {
    info!(
        "🔍 Debug registers requested, radar connected: {}",
        radar.is_connected()
    );

    // Ensure radar is connected before debugging
    if !radar.is_connected() {
        info!("🔄 Radar not connected, attempting to connect for register debugging...");
        if let Err(e) = radar.connect() {
            eprintln!("❌ Failed to connect radar for register debugging: {e}");
            return;
        }
    }

    if radar.is_connected() {
        match radar.debug_registers(mode) {
            Ok(_) => info!("✅ Register debugging completed successfully"),
            Err(e) => {
                eprintln!("❌ Failed to debug registers: {e}");
                warn!("Failed to debug registers: {e}");
            }
        }
    } else {
        eprintln!("❌ Cannot debug registers: radar not connected after connection attempt");
    }
}

/// Display distance measurement result
fn display_distance_result(result: &radar::DistanceMeasurement, format: &cli::OutputFormat) {
    match format {
        cli::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        cli::OutputFormat::Csv => {
            println!("distance_m,signal_strength,temperature_c");
            println!(
                "{:.3},{:.2},{}",
                result.distance, result.strength, result.temperature
            );
        }
        cli::OutputFormat::Human => {
            println!("📏 Distance Measurement:");
            println!("  Distance: {:.3}m", result.distance);
            println!("  Signal Strength: {:.2}", result.strength);
            println!("  Temperature: {:.1}°C", result.temperature);
        }
    }
}

/// Display presence detection result
fn display_presence_result(result: &radar::PresenceMeasurement, format: &cli::OutputFormat) {
    match format {
        cli::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        cli::OutputFormat::Csv => {
            println!("presence_detected,presence_distance_m,intra_score,inter_score");
            println!(
                "{},{:.2},{:.2},{:.2}",
                result.presence_detected,
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score
            );
        }
        cli::OutputFormat::Human => {
            println!("👁️ Presence Detection:");
            let status = if result.presence_detected {
                "DETECTED"
            } else {
                "NOT DETECTED"
            };
            println!(
                "Presence: {}, Distance: {:.2}m, Intra: {:.2}, Inter: {:.2}",
                status,
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score
            );
        }
    }
}

/// Monitor distance measurements continuously
async fn monitor_distance_continuous(
    radar: &mut radar::XM125Radar,
    cli: &Cli,
    count: Option<u32>,
    interval: u64,
    save_to: Option<&str>,
    mut fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    use tokio::time::{sleep, Duration};

    let mut csv_writer = if let Some(filename) = save_to {
        let file = std::fs::File::create(filename).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to create CSV file '{filename}': {e}"),
        })?;
        let mut writer = csv::Writer::from_writer(file);

        // Write CSV header
        writer
            .write_record([
                "timestamp",
                "distance_m",
                "signal_strength",
                "temperature_c",
            ])
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to write CSV header: {e}"),
            })?;

        Some(writer)
    } else {
        None
    };

    let infinite = count.is_none();
    let total_count = count.unwrap_or(u32::MAX);

    if !cli.quiet {
        if infinite {
            println!("🔄 Starting continuous distance monitoring (Ctrl+C to stop)");
        } else {
            println!("🔄 Starting distance monitoring ({total_count} measurements)");
        }
        println!("📊 Interval: {interval}ms");
        if save_to.is_some() {
            println!("💾 Saving to: {}", save_to.unwrap());
        }
        println!("────────────────────────────────────────────────────────────");
    }

    let mut measurement_count = 0;

    while measurement_count < total_count {
        let start_time = std::time::Instant::now();

        match radar.measure_distance().await {
            Ok(result) => {
                measurement_count += 1;

                // Display result
                if !cli.quiet {
                    let timestamp = Utc::now().format("%H:%M:%S%.3f").to_string();
                    println!(
                        "[{timestamp}] #{measurement_count:3} Distance: {:.3}m, Signal: {:.2}, Temp: {:.1}°C",
                        result.distance, result.strength, result.temperature
                    );
                }

                // Save to CSV if requested
                if let Some(ref mut writer) = csv_writer {
                    let timestamp_full = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
                    writer
                        .write_record([
                            timestamp_full.as_str(),
                            &result.distance.to_string(),
                            &result.strength.to_string(),
                            &result.temperature.to_string(),
                        ])
                        .map_err(|e| RadarError::DeviceError {
                            message: format!("Failed to write CSV record: {e}"),
                        })?;
                    writer.flush().map_err(|e| RadarError::DeviceError {
                        message: format!("Failed to flush CSV writer: {e}"),
                    })?;
                }

                // FIFO output
                if let Some(ref mut writer) = fifo_writer {
                    write_distance_to_fifo(writer, &result, &cli.fifo_format);
                }

                // Check if we've reached the target count
                if !infinite && measurement_count >= total_count {
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Measurement #{} failed: {}", measurement_count + 1, e);
                // Continue with next measurement
            }
        }

        // Wait for the specified interval
        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(interval);
        if elapsed < target_duration {
            sleep(target_duration - elapsed).await;
        }
    }

    if !cli.quiet {
        println!("────────────────────────────────────────────────────────────");
        println!("✅ Completed {measurement_count} distance measurements");
        if let Some(filename) = save_to {
            println!("💾 Results saved to: {filename}");
        }
    }
}

/// Monitor presence detection continuously
#[allow(clippy::too_many_lines)]
async fn monitor_presence_continuous(
    radar: &mut radar::XM125Radar,
    cli: &Cli,
    count: Option<u32>,
    interval: u64,
    save_to: Option<&str>,
    mut fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    use tokio::time::{sleep, Duration};

    let mut csv_writer = if let Some(filename) = save_to {
        let file = std::fs::File::create(filename).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to create CSV file '{filename}': {e}"),
        })?;
        let mut writer = csv::Writer::from_writer(file);

        // Write enhanced CSV header for hardware testing
        writer
            .write_record([
                "timestamp",
                "presence_detected",
                "presence_distance_m",
                "intra_score",
                "inter_score",
                "intra_strength",
                "inter_strength",
                "detection_confidence",
                "measurement_number",
            ])
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to write CSV header: {e}"),
            })?;

        Some(writer)
    } else {
        None
    };

    let infinite = count.is_none();
    let total_count = count.unwrap_or(u32::MAX);

    if !cli.quiet {
        if infinite {
            println!("🔄 Starting continuous presence monitoring (Ctrl+C to stop)");
        } else {
            println!("🔄 Starting presence monitoring ({total_count} measurements)");
        }
        println!("📊 Interval: {interval}ms");
        if save_to.is_some() {
            println!("💾 Saving to: {}", save_to.unwrap());
        }
        println!("────────────────────────────────────────────────────────────");
    }

    let mut measurement_count = 0;

    while measurement_count < total_count {
        let start_time = std::time::Instant::now();

        match radar.measure_presence().await {
            Ok(result) => {
                measurement_count += 1;

                // Display result with enhanced testing information
                if !cli.quiet {
                    let timestamp = Utc::now().format("%H:%M:%S%.3f").to_string();
                    let status = if result.presence_detected {
                        "DETECTED"
                    } else {
                        "NOT DETECTED"
                    };

                    // Calculate signal quality indicators for testing
                    let intra_strength = if result.intra_presence_score > 2.0 {
                        "STRONG"
                    } else if result.intra_presence_score > 1.0 {
                        "MEDIUM"
                    } else if result.intra_presence_score > 0.5 {
                        "WEAK"
                    } else {
                        "NONE"
                    };

                    let inter_strength = if result.inter_presence_score > 2.0 {
                        "STRONG"
                    } else if result.inter_presence_score > 1.0 {
                        "MEDIUM"
                    } else if result.inter_presence_score > 0.5 {
                        "WEAK"
                    } else {
                        "NONE"
                    };

                    // Enhanced output for hardware testing
                    println!(
                        "[{timestamp}] #{measurement_count:3} Presence: {status:>12} | Distance: {:.2}m | Fast: {:.2}({intra_strength:>6}) | Slow: {:.2}({inter_strength:>6})",
                        result.presence_distance, result.intra_presence_score, result.inter_presence_score
                    );
                }

                // Save enhanced data to CSV if requested
                if let Some(ref mut writer) = csv_writer {
                    let timestamp_full = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

                    // Calculate signal strength indicators for CSV
                    let intra_strength = if result.intra_presence_score > 2.0 {
                        "STRONG"
                    } else if result.intra_presence_score > 1.0 {
                        "MEDIUM"
                    } else if result.intra_presence_score > 0.5 {
                        "WEAK"
                    } else {
                        "NONE"
                    };

                    let inter_strength = if result.inter_presence_score > 2.0 {
                        "STRONG"
                    } else if result.inter_presence_score > 1.0 {
                        "MEDIUM"
                    } else if result.inter_presence_score > 0.5 {
                        "WEAK"
                    } else {
                        "NONE"
                    };

                    // Calculate overall detection confidence
                    let confidence = if result.presence_detected {
                        let max_score =
                            result.intra_presence_score.max(result.inter_presence_score);
                        if max_score > 3.0 {
                            "HIGH"
                        } else if max_score > 1.5 {
                            "MEDIUM"
                        } else {
                            "LOW"
                        }
                    } else {
                        "NONE"
                    };

                    writer
                        .write_record([
                            timestamp_full.as_str(),
                            &result.presence_detected.to_string(),
                            &result.presence_distance.to_string(),
                            &result.intra_presence_score.to_string(),
                            &result.inter_presence_score.to_string(),
                            intra_strength,
                            inter_strength,
                            confidence,
                            &measurement_count.to_string(),
                        ])
                        .map_err(|e| RadarError::DeviceError {
                            message: format!("Failed to write CSV record: {e}"),
                        })?;
                    writer.flush().map_err(|e| RadarError::DeviceError {
                        message: format!("Failed to flush CSV writer: {e}"),
                    })?;
                }

                // FIFO output
                if let Some(ref mut writer) = fifo_writer {
                    write_presence_to_fifo(writer, &result, &cli.fifo_format);
                }

                // Check if we've reached the target count
                if !infinite && measurement_count >= total_count {
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Measurement #{} failed: {}", measurement_count + 1, e);
                // Continue with next measurement
            }
        }

        // Wait for the specified interval
        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(interval);
        if elapsed < target_duration {
            sleep(target_duration - elapsed).await;
        }
    }

    if !cli.quiet {
        println!("────────────────────────────────────────────────────────────");
        println!("✅ Completed {measurement_count} presence measurements");
        if let Some(filename) = save_to {
            println!("💾 Results saved to: {filename}");
        }
    }
}

/// Handle firmware-related commands
async fn handle_firmware_action(
    radar: &mut XM125Radar,
    action: &FirmwareAction,
    firmware_path: &str,
) -> Result<(), RadarError> {
    match action {
        FirmwareAction::Check => {
            let info = radar.get_info()?;
            println!("📦 Current Firmware:");
            println!("{info}");
        }
        FirmwareAction::Update {
            firmware_type,
            force: _,
            verify: _,
        } => {
            let manager =
                firmware::FirmwareManager::new(firmware_path, "/usr/bin/xm125-control.sh", 0x52);
            manager.update_firmware(*firmware_type).await?;
        }
        FirmwareAction::Verify { firmware_type } => {
            info!("Firmware verification not yet implemented in v2.0.0");
            if let Some(fw_type) = firmware_type {
                info!("Would verify firmware type: {fw_type:?}");
            } else {
                info!("Would verify current firmware");
            }
        }
        FirmwareAction::Erase { .. }
        | FirmwareAction::Checksum { .. }
        | FirmwareAction::Bootloader { .. } => {
            // These are handled earlier in run() before I2C initialization
            unreachable!("These firmware commands should be handled before I2C initialization");
        }
    }
}

/// Handle firmware erase command
async fn handle_firmware_erase_command(confirm: bool) -> Result<(), RadarError> {
    if !confirm {
        eprintln!("❌ Chip erase requires --confirm flag for safety");
        eprintln!("   This will completely erase all firmware from the XM125 module.");
        eprintln!("   Use: xm125-radar-monitor firmware erase --confirm");
        return Err(RadarError::DeviceError {
            message: "Erase operation not confirmed".to_string(),
        });
    }

    println!("⚠️  WARNING: This will completely erase the XM125 chip!");
    println!("   The module will need to be reprogrammed before it can be used again.");
    println!("   This operation cannot be undone.");
    println!();

    let manager =
        firmware::FirmwareManager::new("/lib/firmware/acconeer", "/usr/bin/xm125-control.sh", 0x52);
    manager.erase_chip().await?;

    println!("✅ Chip erase completed successfully");
    println!("   The XM125 module now needs firmware to be programmed before use.");
}

/// Handle firmware checksum command
fn handle_firmware_checksum_command(
    firmware_type: Option<&firmware::FirmwareType>,
    verbose: bool,
    firmware_path: &str,
) -> Result<(), RadarError> {
    let manager = firmware::FirmwareManager::new(firmware_path, "/usr/bin/xm125-control.sh", 0x52);

    if let Some(fw_type) = firmware_type {
        let checksum = manager.calculate_binary_checksum(*fw_type)?;
        if verbose {
            println!(
                "Firmware: {} ({})",
                fw_type.display_name(),
                fw_type.binary_filename()
            );
            println!("MD5: {checksum}");
        } else {
            println!("{}: {}", fw_type.display_name(), checksum);
        }
    } else {
        // Calculate checksums for all firmware types
        for fw_type in [
            firmware::FirmwareType::Distance,
            firmware::FirmwareType::Presence,
        ] {
            match manager.calculate_binary_checksum(fw_type) {
                Ok(checksum) => {
                    if verbose {
                        println!(
                            "Firmware: {} ({})",
                            fw_type.display_name(),
                            fw_type.binary_filename()
                        );
                        println!("MD5: {checksum}");
                        println!();
                    } else {
                        println!("{}: {}", fw_type.display_name(), checksum);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error calculating checksum for {}: {}",
                        fw_type.display_name(),
                        e
                    );
                }
            }
        }
    }
}

/// Handle bootloader command
async fn handle_bootloader_command(cli: &Cli, test_mode: bool) -> Result<(), RadarError> {
    let gpio_pins = cli.get_gpio_pins();
    let mut gpio_controller = XM125GpioController::with_pins(gpio_pins);

    gpio_controller.initialize()?;

    if test_mode {
        println!("🧪 Testing bootloader mode (will reset back to run mode)");
        gpio_controller.reset_to_bootloader_mode()?;

        // Wait a moment
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        println!("🔄 Resetting back to run mode");
        gpio_controller.reset_to_run_mode()?;

        println!("✅ Bootloader test completed");
    } else {
        println!("🔄 Putting XM125 into bootloader mode...");
        gpio_controller.reset_to_bootloader_mode()?;

        println!("✅ XM125 is now in bootloader mode (I2C address 0x48)");
        println!("   Ready for firmware programming with stm32flash");
        println!("   Use 'xm125-radar-monitor gpio reset-run' to return to normal mode");
    }
}

/// Handle GPIO commands
fn handle_gpio_command(cli: &Cli, action: &GpioAction) -> Result<(), RadarError> {
    let gpio_pins = cli.get_gpio_pins();
    let mut gpio_controller = XM125GpioController::with_pins(gpio_pins);

    match action {
        GpioAction::Init => {
            gpio_controller.initialize()?;
            println!("✅ GPIO pins initialized successfully");
        }
        GpioAction::Status => {
            gpio_controller.show_gpio_status()?;
        }
        GpioAction::ResetRun => {
            gpio_controller.initialize()?;
            gpio_controller.reset_to_run_mode()?;
            println!("✅ XM125 reset to run mode (I2C address 0x52)");
        }
        GpioAction::ResetBootloader => {
            gpio_controller.initialize()?;
            gpio_controller.reset_to_bootloader_mode()?;
            println!("✅ XM125 reset to bootloader mode (I2C address 0x48)");
        }
        GpioAction::Test => {
            gpio_controller.initialize()?;
            gpio_controller.test_bootloader_control()?;
        }
    }
}

/// Write distance measurement to FIFO with timing control
fn write_distance_to_fifo(
    writer: &mut FifoWriter,
    result: &radar::DistanceMeasurement,
    format: &fifo::FifoFormat,
) {
    match format {
        fifo::FifoFormat::Simple => {
            // Simple format: presence_state (always 1 for distance) and distance
            let _ = writer.write_timed_simple(1, result.distance);
        }
        fifo::FifoFormat::Json => {
            let json_data = serde_json::json!({
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "sensor_type": "XM125",
                "detection_mode": "distance",
                "distance_m": result.distance,
                "signal_strength": result.strength,
                "temperature_c": result.temperature
            });
            let _ = writer.write_timed_json(&json_data);
        }
    }
}

/// Write presence measurement to FIFO with timing control
fn write_presence_to_fifo(
    writer: &mut FifoWriter,
    result: &radar::PresenceMeasurement,
    format: &fifo::FifoFormat,
) {
    match format {
        fifo::FifoFormat::Simple => {
            // BGT60TR13C compatible format: presence_state (0/1) and distance
            let presence_state = i32::from(result.presence_detected);
            let _ = writer.write_timed_simple(presence_state, result.presence_distance);
        }
        fifo::FifoFormat::Json => {
            let json_data = serde_json::json!({
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "sensor_type": "XM125",
                "detection_mode": "presence",
                "presence_detected": result.presence_detected,
                "presence_distance_m": result.presence_distance,
                "intra_score": result.intra_presence_score,
                "inter_score": result.inter_presence_score,
                "signal_quality": if result.intra_presence_score.max(result.inter_presence_score) > 2.0 {
                    "STRONG"
                } else if result.intra_presence_score.max(result.inter_presence_score) > 1.0 {
                    "MEDIUM"
                } else if result.intra_presence_score.max(result.inter_presence_score) > 0.5 {
                    "WEAK"
                } else {
                    "NONE"
                },
                "confidence": if result.presence_detected {
                    let max_score = result.intra_presence_score.max(result.inter_presence_score);
                    if max_score > 3.0 { "HIGH" } else if max_score > 1.5 { "MEDIUM" } else { "LOW" }
                } else {
                    "NONE"
                }
            });
            let _ = writer.write_timed_json(&json_data);
        }
    }
}
