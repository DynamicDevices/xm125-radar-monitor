//! Configuration and setup functions
//!
//! This module handles device configuration, range setup, and debugging utilities.

use crate::cli::{PresenceRange, ProfileMode};
use crate::error::RadarError;
use crate::radar::{presence::PresenceRange as RadarPresenceRange, XM125Radar};
use log::{info, warn};

/// Configure distance measurement range
pub fn configure_distance_range(radar: &mut XM125Radar, range_str: &str) -> Result<(), RadarError> {
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

    info!("🎯 Configuring distance range: {start:.2}m - {end:.2}m");
    radar.config.start_m = start;
    radar.config.length_m = end - start;
    Ok(())
}

/// Configure presence parameters for the radar
#[allow(unused_assignments)]
pub fn configure_presence_parameters(
    radar: &mut XM125Radar,
    presence_range: Option<&PresenceRange>,
    min_range: Option<f32>,
    max_range: Option<f32>,
    sensitivity: Option<f32>,
    frame_rate: Option<f32>,
    profile: &ProfileMode,
) -> Result<(), RadarError> {
    #[allow(unused_assignments)]
    let mut config_changed = false;

    // Configure range (either preset or custom)
    if let Some(range) = presence_range {
        radar.config.presence_range = match range {
            PresenceRange::Short => RadarPresenceRange::Short,
            PresenceRange::Medium => RadarPresenceRange::Medium,
            PresenceRange::Long => RadarPresenceRange::Long,
        };
        config_changed = true;
        info!("🎯 Set presence range: {range:?}");
    }

    // Custom range overrides preset
    if let (Some(min), Some(max)) = (min_range, max_range) {
        if min >= max {
            return Err(RadarError::DeviceError {
                message: format!("min_range ({min}) must be less than max_range ({max})"),
            });
        }
        radar.config.start_m = min;
        radar.config.length_m = max - min;
        config_changed = true;
        info!("🎯 Set custom presence range: {min:.2}m - {max:.2}m");
    }

    // Configure sensitivity
    if let Some(sens) = sensitivity {
        if !(0.1..=5.0).contains(&sens) {
            return Err(RadarError::DeviceError {
                message: format!("Sensitivity must be between 0.1 and 5.0 (got {sens:.2})"),
            });
        }
        // Convert sensitivity to internal threshold values
        radar.config.intra_detection_threshold = sens * 1000.0;
        radar.config.inter_detection_threshold = sens * 800.0;
        config_changed = true;
        info!("🔧 Set sensitivity: {sens:.2}");
    }

    // Configure frame rate
    if let Some(rate) = frame_rate {
        if !(1.0..=60.0).contains(&rate) {
            return Err(RadarError::DeviceError {
                message: format!("Frame rate must be between 1.0 and 60.0 Hz (got {rate:.1})"),
            });
        }
        radar.config.frame_rate = rate;
        config_changed = true;
        info!("🔧 Set frame rate: {rate:.1} Hz");
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
    config_changed = true;

    if config_changed || (presence_range.is_none() && min_range.is_none() && max_range.is_none()) {
        radar.configure_presence_range()?;
        if config_changed {
            info!("✅ Presence parameters configured successfully");
        } else {
            info!("✅ Applied default presence configuration (long range: 0.5m - 7.0m)");
        }
    }
    Ok(())
}

/// Debug registers if radar is connected, with automatic connection attempt
pub fn debug_registers_if_connected(radar: &mut XM125Radar, mode: &str) {
    if radar.is_connected() {
        match radar.debug_registers(mode) {
            Ok(()) => info!("✅ Register debugging completed successfully"),
            Err(e) => {
                eprintln!("❌ Failed to debug registers: {e}");
                warn!("Register debugging failed, but continuing with measurement");
            }
        }
    } else {
        warn!("⚠️  Radar not connected, skipping register debugging");
        warn!("   Use --verbose for detailed connection diagnostics");
    }
}
