//! Continuous monitoring functions
//!
//! This module handles continuous measurement operations for both distance and presence
//! detection, including CSV export and FIFO output integration.

use crate::cli::Cli;
use crate::display::{
    display_distance_result, display_presence_result, write_distance_to_fifo,
    write_presence_to_fifo,
};
use crate::error::RadarError;
use crate::fifo::FifoWriter;
use crate::radar::XM125Radar;
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::fs::File;
use tokio::time::{sleep, Duration};

/// Monitor distance detection continuously
pub async fn monitor_distance_continuous(
    radar: &mut XM125Radar,
    cli: &Cli,
    count: Option<u32>,
    interval: u64,
    save_to: Option<&str>,
    mut fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    let total_measurements = count.unwrap_or(u32::MAX);
    let mut measurement_count = 0u32;

    // Setup progress bar
    let progress = if !cli.quiet && count.is_some() {
        let pb = ProgressBar::new(total_measurements as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Setup CSV writer if requested
    let mut csv_writer = if let Some(filename) = save_to {
        let file = File::create(filename).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to create CSV file: {e}"),
        })?;
        let mut writer = csv::Writer::from_writer(file);

        // Write CSV header
        writer
            .write_record(&[
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

    info!("🚀 Starting continuous distance monitoring...");
    if let Some(count) = count {
        info!("📊 Taking {} measurements every {}ms", count, interval);
    } else {
        info!(
            "📊 Continuous monitoring every {}ms (Ctrl+C to stop)",
            interval
        );
    }

    while measurement_count < total_measurements {
        let result = radar.measure_distance().await?;
        let timestamp_full = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        // Display result unless quiet mode
        if !cli.quiet {
            display_distance_result(&result, &cli.format);
        }

        // CSV output
        if let Some(ref mut writer) = csv_writer {
            writer
                .write_record(&[
                    &timestamp_full,
                    &format!("{:.3}", result.distance),
                    &format!("{:.1}", result.strength),
                    &format!("{:.1}", result.temperature),
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

        measurement_count += 1;

        // Update progress bar
        if let Some(ref pb) = progress {
            pb.set_position(measurement_count as u64);
        }

        // Break if we've reached the count
        if count.is_some() && measurement_count >= total_measurements {
            break;
        }

        // Wait for next measurement
        sleep(Duration::from_millis(interval)).await;
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("✅ Distance monitoring completed");
    }

    // Print summary
    if let Some(filename) = save_to {
        println!("💾 Results saved to: {filename}");
    }

    Ok(())
}

/// Monitor presence detection continuously
pub async fn monitor_presence_continuous(
    radar: &mut XM125Radar,
    cli: &Cli,
    count: Option<u32>,
    interval: u64,
    save_to: Option<&str>,
    mut fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    let total_measurements = count.unwrap_or(u32::MAX);
    let mut measurement_count = 0u32;

    // Setup progress bar
    let progress = if !cli.quiet && count.is_some() {
        let pb = ProgressBar::new(total_measurements as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Setup CSV writer if requested
    let mut csv_writer = if let Some(filename) = save_to {
        let file = File::create(filename).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to create CSV file: {e}"),
        })?;
        let mut writer = csv::Writer::from_writer(file);

        // Write CSV header
        writer
            .write_record(&[
                "timestamp",
                "measurement_number",
                "presence_detected",
                "presence_distance_m",
                "intra_score",
                "inter_score",
                "signal_quality",
                "confidence",
            ])
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to write CSV header: {e}"),
            })?;
        Some(writer)
    } else {
        None
    };

    info!("🚀 Starting continuous presence monitoring...");
    if let Some(count) = count {
        info!("📊 Taking {} measurements every {}ms", count, interval);
    } else {
        info!(
            "📊 Continuous monitoring every {}ms (Ctrl+C to stop)",
            interval
        );
    }

    while measurement_count < total_measurements {
        let result = radar.measure_presence().await?;
        let timestamp_full = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        measurement_count += 1;

        // Display result unless quiet mode
        if !cli.quiet {
            display_presence_result(&result, &cli.format);
        }

        // CSV output
        if let Some(ref mut writer) = csv_writer {
            let max_score = result.intra_presence_score.max(result.inter_presence_score);
            let signal_quality = if max_score > 2.0 {
                "STRONG"
            } else if max_score > 1.0 {
                "MEDIUM"
            } else if max_score > 0.5 {
                "WEAK"
            } else {
                "NONE"
            };
            let confidence = if result.presence_detected {
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
                .write_record(&[
                    &timestamp_full,
                    &measurement_count.to_string(),
                    &result.presence_detected.to_string(),
                    &format!("{:.3}", result.presence_distance),
                    &format!("{:.2}", result.intra_presence_score),
                    &format!("{:.2}", result.inter_presence_score),
                    signal_quality,
                    confidence,
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

        // Update progress bar
        if let Some(ref pb) = progress {
            pb.set_position(measurement_count as u64);
        }

        // Break if we've reached the count
        if count.is_some() && measurement_count >= total_measurements {
            break;
        }

        // Wait for next measurement
        sleep(Duration::from_millis(interval)).await;
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("✅ Presence monitoring completed");
    }

    // Print summary
    if let Some(filename) = save_to {
        println!("💾 Results saved to: {filename}");
    }

    Ok(())
}
