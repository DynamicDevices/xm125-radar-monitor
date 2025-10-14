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
        println!("{} v{}", APP_NAME, VERSION);
        println!("Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.");
        println!("XM125 Radar Module Monitor");
        println!();
    }

    if let Err(e) = run(cli).await {
        error!("Command failed: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), RadarError> {
    debug!("Starting {} v{}", APP_NAME, VERSION);

    // Create I2C connection to XM125
    let i2c_device = i2c::I2cDevice::new(&cli.i2c_device, cli.i2c_address)?;
    let mut radar = radar::XM125Radar::new(i2c_device);

    // Execute command
    match &cli.command {
        Some(cmd) => {
            debug!("Executing command: {:?}", cmd);
            execute_command(cmd.clone(), &mut radar, &cli).await?;
            Ok(())
        }
        None => {
            println!("No command provided. Use --help for usage information.");
            Ok(())
        }
    }
}

async fn execute_command(
    command: cli::Commands,
    radar: &mut radar::XM125Radar,
    cli: &Cli,
) -> Result<(), RadarError> {
    use cli::Commands;

    match command {
        Commands::Status => {
            let status = radar.get_status().await?;
            output_response(cli, "status", &status, "📊", "Radar Status")?;
        }
        Commands::Connect => {
            radar.connect().await?;
            output_response(cli, "connect", "Connected successfully", "🔗", "Connection")?;
        }
        Commands::Disconnect => {
            radar.disconnect().await?;
            output_response(
                cli,
                "disconnect",
                "Disconnected successfully",
                "🔌",
                "Disconnection",
            )?;
        }
        Commands::Info => {
            let info = radar.get_info().await?;
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
        Commands::Monitor { interval, count } => {
            monitor_distances(radar, cli, interval, count).await?;
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
            println!("{} {}:", emoji, title);
            println!("{}", response);
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
                response.replace("\"", "\"\"")
            );
        }
    }

    Ok(())
}
// Test comment
