// Register Debug Module
// Provides comprehensive register debugging functionality

#![allow(clippy::pedantic)]

use super::registers::{
    REG_CLOSE_RANGE_LEAKAGE_CANCELLATION, REG_COMMAND, REG_DETECTOR_STATUS, REG_DISTANCE_RESULT,
    REG_END_CONFIG, REG_FIXED_AMPLITUDE_THRESHOLD_VALUE, REG_FIXED_STRENGTH_THRESHOLD_VALUE,
    REG_MAX_PROFILE, REG_MAX_STEP_LENGTH, REG_MEASURE_COUNTER, REG_NUM_FRAMES_RECORDED_THRESHOLD,
    REG_PEAK0_DISTANCE, REG_PEAK0_STRENGTH, REG_PEAK_SORTING, REG_PROTOCOL_STATUS,
    REG_REFLECTOR_SHAPE, REG_SIGNAL_QUALITY, REG_START_CONFIG, REG_THRESHOLD_METHOD,
    REG_THRESHOLD_SENSITIVITY, REG_VERSION,
};
use crate::error::Result;
use crate::i2c::I2cDevice;

pub struct RegisterDebugger<'a> {
    i2c: &'a mut I2cDevice,
}

impl<'a> RegisterDebugger<'a> {
    pub fn new(i2c: &'a mut I2cDevice) -> Self {
        Self { i2c }
    }

    /// Debug all common registers
    pub fn debug_common_registers(&mut self) -> Result<()> {
        println!("📊 Common Status & Control Registers:");
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        println!(
            "  Addr   (Dec) │ Register Name             │ Value (Hex)  (Decimal) │ Description"
        );
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );

        self.debug_register(
            REG_VERSION,
            "Module Version",
            "Hardware/firmware version info",
        )?;
        self.debug_register(
            REG_PROTOCOL_STATUS,
            "Protocol Status",
            "Communication protocol status",
        )?;
        self.debug_register(
            REG_MEASURE_COUNTER,
            "Measure Counter",
            "Number of measurements performed",
        )?;
        self.debug_register(
            REG_DETECTOR_STATUS,
            "Detector Status",
            "Current detector state and flags",
        )?;
        self.debug_register(
            REG_COMMAND,
            "Command Register",
            "Last executed command code",
        )?;

        Ok(())
    }

    /// Debug presence detector registers
    pub fn debug_presence_registers(&mut self) -> Result<()> {
        println!("\n👤 Presence Detector Configuration:");
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        println!(
            "  Addr   (Dec) │ Register Name             │ Value (Hex)  (Decimal) │ Description"
        );
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );

        // Configuration registers
        self.debug_register(
            64,
            "Sweeps Per Frame",
            "Number of sweeps per measurement frame",
        )?;
        self.debug_register(
            65,
            "Inter Frame Timeout",
            "Presence timeout in seconds (0-30)",
        )?;
        self.debug_register(
            66,
            "Inter Phase Boost",
            "Phase boost for slow motion detection",
        )?;
        self.debug_register(67, "Intra Detection", "Fast motion detection enable (0/1)")?;
        self.debug_register(68, "Inter Detection", "Slow motion detection enable (0/1)")?;
        self.debug_register(69, "Frame Rate", "Frame rate in mHz (value * 1000)")?;
        self.debug_register(
            70,
            "Intra Threshold",
            "Fast motion threshold (value * 1000)",
        )?;
        self.debug_register(
            71,
            "Inter Threshold",
            "Slow motion threshold (value * 1000)",
        )?;
        self.debug_register(72, "Inter Dev Time", "Inter-frame deviation time constant")?;
        self.debug_register(73, "Inter Fast Cutoff", "Fast filter cutoff frequency")?;
        self.debug_register(74, "Inter Slow Cutoff", "Slow filter cutoff frequency")?;
        self.debug_register(75, "Intra Frame Time", "Intra-frame time constant")?;
        self.debug_register(76, "Intra Output Time", "Intra output time constant")?;
        self.debug_register(77, "Inter Output Time", "Inter output time constant")?;
        self.debug_register(78, "Auto Profile", "Auto profile selection enable (0/1)")?;
        self.debug_register(79, "Auto Step Length", "Auto step length enable (0/1)")?;
        self.debug_register(80, "Manual Profile", "Manual profile (1-5)")?;
        self.debug_register(81, "Manual Step Length", "Manual step length")?;
        self.debug_register(82, "Start Point", "Start distance in mm * 1000")?;
        self.debug_register(83, "End Point", "End distance in mm * 1000")?;
        self.debug_register(84, "Reset Filters", "Reset filters on prepare (0/1)")?;
        self.debug_register(85, "HWAAS", "Hardware accelerated average samples")?;
        self.debug_register(86, "Auto Subsweeps", "Automatic subsweeps enable (0/1)")?;
        self.debug_register(87, "Signal Quality", "Signal quality threshold")?;
        self.debug_register(128, "Detection GPIO", "Output detection on GPIO (0/1)")?;

        // Result registers
        println!("\n📊 Presence Detector Results:");
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        println!(
            "  Addr   (Dec) │ Register Name             │ Value (Hex)  (Decimal) │ Description"
        );
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );

        self.debug_register(0, "Version", "RSS version (major.minor.patch)")?;
        self.debug_register(1, "Protocol Status", "Protocol error flags")?;
        self.debug_register(2, "Measure Counter", "Number of measurements since restart")?;
        self.debug_register(3, "Detector Status", "Detector status flags")?;
        self.debug_register(
            16,
            "Presence Result",
            "Presence detection result & temperature",
        )?;
        self.debug_register(
            17,
            "Presence Distance",
            "Distance to detected presence (mm)",
        )?;
        self.debug_register(18, "Intra Score", "Fast motion detection score")?;
        self.debug_register(19, "Inter Score", "Slow motion detection score")?;
        self.debug_register(32, "Actual Frame Rate", "Actual frame rate in mHz")?;

        // Application info
        println!("\n🆔 Application Information:");
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        self.debug_register(65535, "Application ID", "Firmware application identifier")?;

        Ok(())
    }

    /// Debug distance detector registers
    pub fn debug_distance_registers(&mut self) -> Result<()> {
        println!("\n📏 Distance Detector Configuration:");
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        println!(
            "  Addr   (Dec) │ Register Name             │ Value (Hex)  (Decimal) │ Description"
        );
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );

        // Configuration registers
        self.debug_register(
            REG_START_CONFIG,
            "Start Config",
            "Detection start point (mm)",
        )?;
        self.debug_register(REG_END_CONFIG, "End Config", "Detection end point (mm)")?;
        self.debug_register(
            REG_MAX_STEP_LENGTH,
            "Max Step Length",
            "Maximum step length",
        )?;
        self.debug_register(
            REG_CLOSE_RANGE_LEAKAGE_CANCELLATION,
            "Leakage Cancel",
            "Close range leakage cancellation",
        )?;
        self.debug_register(
            REG_SIGNAL_QUALITY,
            "Signal Quality",
            "Signal quality threshold",
        )?;
        self.debug_register(REG_MAX_PROFILE, "Max Profile", "Maximum profile setting")?;
        self.debug_register(
            REG_THRESHOLD_METHOD,
            "Threshold Method",
            "Threshold calculation method",
        )?;
        self.debug_register(REG_PEAK_SORTING, "Peak Sorting", "Peak sorting method")?;
        self.debug_register(
            REG_NUM_FRAMES_RECORDED_THRESHOLD,
            "Frames Threshold",
            "Number of frames for threshold",
        )?;
        self.debug_register(
            REG_FIXED_AMPLITUDE_THRESHOLD_VALUE,
            "Fixed Amplitude",
            "Fixed amplitude threshold value",
        )?;
        self.debug_register(
            REG_THRESHOLD_SENSITIVITY,
            "Sensitivity",
            "Detection sensitivity",
        )?;
        self.debug_register(
            REG_REFLECTOR_SHAPE,
            "Reflector Shape",
            "Expected reflector shape",
        )?;
        self.debug_register(
            REG_FIXED_STRENGTH_THRESHOLD_VALUE,
            "Fixed Strength",
            "Fixed strength threshold value",
        )?;

        // Result registers
        println!("\n📊 Distance Detector Results:");
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        self.debug_register(
            REG_DISTANCE_RESULT,
            "Distance Result",
            "Measured distance (mm)",
        )?;
        self.debug_register(
            REG_PEAK0_DISTANCE,
            "Peak 0 Distance",
            "Peak 0 distance (mm)",
        )?;
        self.debug_register(
            REG_PEAK0_STRENGTH,
            "Peak 0 Strength",
            "Peak 0 signal strength",
        )?;

        Ok(())
    }

    /// Debug a single register
    fn debug_register(&mut self, address: u16, name: &str, description: &str) -> Result<()> {
        match self.i2c.read_register(address, 4) {
            Ok(data) => {
                let value = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                println!(
                    "  0x{:04X} ({:3}) │ {:<25} │ 0x{:08X} ({:10}) │ {}",
                    address, address, name, value, value, description
                );
            }
            Err(e) => {
                println!(
                    "  0x{:04X} ({:3}) │ {:<25} │ ERROR: {:?} │ {}",
                    address, address, name, e, description
                );
            }
        }
        Ok(())
    }

    /// Debug all registers based on detector mode
    pub fn debug_all_registers(&mut self, detector_mode: &str) -> Result<()> {
        println!(
            "================================================================================"
        );
        println!("XM125 Register Dump - {} Mode", detector_mode);
        println!(
            "================================================================================"
        );

        self.debug_common_registers()?;

        match detector_mode.to_lowercase().as_str() {
            "presence" => self.debug_presence_registers()?,
            "distance" => self.debug_distance_registers()?,
            _ => {
                // Debug both for unknown modes
                self.debug_presence_registers()?;
                self.debug_distance_registers()?;
            }
        }

        println!(
            "================================================================================"
        );
        Ok(())
    }
}
