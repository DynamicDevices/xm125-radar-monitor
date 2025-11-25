//! Display and output formatting functions
//!
//! This module handles all output formatting and display logic for measurements,
//! including console output and FIFO writing for system integration.

use crate::cli::OutputFormat;
use crate::fifo::{FifoFormat, FifoWriter};
use crate::radar::{DistanceMeasurement, PresenceMeasurement};
use chrono::Utc;

/// Display distance measurement result in the specified format
pub fn display_distance_result(result: &DistanceMeasurement, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json_result = serde_json::json!({
                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "distance_m": result.distance,
                "signal_strength": result.strength,
                "temperature_c": result.temperature
            });
            println!("{}", serde_json::to_string_pretty(&json_result).unwrap());
        }
        OutputFormat::Csv => {
            println!("timestamp,distance_m,signal_strength,temperature_c");
            println!(
                "{},{:.3},{:.1},{:.1}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                result.distance,
                result.strength,
                result.temperature
            );
        }
        OutputFormat::Human => {
            println!(
                "📏 Distance: {:.3}m | Signal: {:.1} | Temp: {:.1}°C",
                result.distance, result.strength, result.temperature
            );
        }
    }
}

/// Display presence measurement result in the specified format
pub fn display_presence_result(result: &PresenceMeasurement, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json_result = serde_json::json!({
                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
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
            println!("{}", serde_json::to_string_pretty(&json_result).unwrap());
        }
        OutputFormat::Csv => {
            println!("timestamp,presence_detected,presence_distance_m,intra_score,inter_score,signal_quality,confidence");
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
            println!(
                "{},{},{:.3},{:.2},{:.2},{},{}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                result.presence_detected,
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score,
                signal_quality,
                confidence
            );
        }
        OutputFormat::Human => {
            let status = if result.presence_detected {
                "🟢 DETECTED"
            } else {
                "🔴 NONE"
            };
            let max_score = result.intra_presence_score.max(result.inter_presence_score);
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

            println!(
                "{} | Distance: {:.3}m | Scores: {:.2}/{:.2} | Confidence: {}",
                status,
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score,
                confidence
            );
        }
    }
}

/// Write distance measurement to FIFO with timing control
pub fn write_distance_to_fifo(
    writer: &mut FifoWriter,
    result: &DistanceMeasurement,
    format: &FifoFormat,
) {
    match format {
        FifoFormat::Simple => {
            // Simple format: presence_state (always 1 for distance) and distance
            let _ = writer.write_timed_simple(1, result.distance);
        }
        FifoFormat::Json => {
            let json_data = serde_json::json!({
                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
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
pub fn write_presence_to_fifo(
    writer: &mut FifoWriter,
    result: &PresenceMeasurement,
    format: &FifoFormat,
) {
    match format {
        FifoFormat::Simple => {
            // BGT60TR13C compatible format: presence_state (0/1) and distance
            let presence_state = i32::from(result.presence_detected);
            let _ = writer.write_timed_simple(presence_state, result.presence_distance);
        }
        FifoFormat::Json => {
            let json_data = serde_json::json!({
                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
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
