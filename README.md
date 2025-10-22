# XM125 Radar Monitor

[![Build Status](https://github.com/DynamicDevices/xm125-radar-monitor/workflows/CI/badge.svg)](https://github.com/DynamicDevices/xm125-radar-monitor/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-ARM64%20Linux-green.svg)](https://github.com/DynamicDevices/xm125-radar-monitor)
[![Codecov](https://codecov.io/gh/DynamicDevices/xm125-radar-monitor/branch/main/graph/badge.svg)](https://codecov.io/gh/DynamicDevices/xm125-radar-monitor)
[![Security Audit](https://github.com/DynamicDevices/xm125-radar-monitor/workflows/Security%20Audit/badge.svg)](https://github.com/DynamicDevices/xm125-radar-monitor/actions)
[![Crates.io](https://img.shields.io/crates/v/xm125-radar-monitor.svg)](https://crates.io/crates/xm125-radar-monitor)
[![Documentation](https://docs.rs/xm125-radar-monitor/badge.svg)](https://docs.rs/xm125-radar-monitor)

Production-ready CLI tool for Acconeer XM125 radar modules with automatic firmware management.

**Maintainer**: Alex J Lennon (ajlennon@dynamicdevices.co.uk)  
**Contact**: info@dynamicdevices.co.uk  
**Copyright**: © 2025 Dynamic Devices Ltd. All rights reserved.

**Keywords**: Acconeer XM125, radar sensor, I2C communication, firmware management, distance detection, presence detection, embedded Linux, ARM64, cross-compilation, stm32flash

## Features

- **Multi-mode Detection**: Distance, presence, and breathing detection
- **Automatic Firmware Management**: Auto-detects and updates firmware via `stm32flash`
- **I2C Communication**: Direct hardware interface with comprehensive error handling
- **GPIO Control**: Hardware reset and bootloader control via GPIO pins
- **Real-time Monitoring**: Continuous measurements with CSV export
- **Cross-compilation**: Native ARM64 builds for embedded targets

## Quick Start

```bash
# Check device status
sudo ./xm125-radar-monitor status

# Test presence detection
sudo ./xm125-radar-monitor --mode presence presence

# Continuous monitoring (10 samples)
sudo ./xm125-radar-monitor --mode presence monitor --count 10

# Auto-update firmware when switching modes
sudo ./xm125-radar-monitor --mode distance --auto-update-firmware measure
```

## Hardware Requirements

- **Target**: ARM64 Linux (tested on Sentai i.MX8MM)
- **I2C Bus**: `/dev/i2c-2` (default)
- **Device Address**: `0x52` (run mode), `0x48` (bootloader)
- **GPIO Pins**: 124 (reset), 125 (interrupt), 139 (wake), 141 (boot)
- **Firmware**: `/lib/firmware/acconeer/*.bin`

## Build & Deploy

```bash
# Cross-compile for ARM64
./build-aarch64.sh

# Deploy to target
scp target/aarch64-unknown-linux-gnu/release/xm125-radar-monitor user@target:/usr/local/bin/
```

## Firmware Management

The tool automatically manages three firmware types:
- **Distance**: `i2c_distance_detector.bin` (App ID: 1)
- **Presence**: `i2c_presence_detector.bin` (App ID: 2)  
- **Breathing**: `i2c_ref_app_breathing.bin` (App ID: 3)

```bash
# Check current firmware
sudo ./xm125-radar-monitor firmware check

# Force firmware update
sudo ./xm125-radar-monitor firmware update presence --force

# Verify firmware integrity
sudo ./xm125-radar-monitor firmware verify
```

## Detection Modes Comparison

| Feature | Distance | Presence | Breathing |
|---------|----------|----------|-----------|
| **Primary Use** | Precise range measurement | Motion/occupancy detection | Breathing pattern analysis |
| **Range** | 0.1-3.0m | 0.5-7.0m (configurable) | 0.3-1.5m |
| **Update Rate** | ~100ms | ~100ms | 5-20s (analysis time) |
| **Output** | Distance, strength, temperature | Detection flag, motion scores | BPM, app state, temperature |
| **Power** | Medium | Low-Medium | Medium-High |
| **Sensitivity** | Object reflectivity | Motion detection | Micro-movements |
| **Applications** | Level sensing, proximity | Smart lighting, security | Health monitoring, sleep |
| **Registers** | 0x10 (packed), 0x11-0x1B (peaks) | 0x10-0x13 (separate) | 0x10 (result), 0x11-0x12 (data) |
| **Calibration** | CFAR thresholds | Motion baselines | Presence + breathing analysis |

## Presence Detection Configuration

### Command Line Options

The XM125 Radar Monitor provides comprehensive configuration options for presence sensing:

#### **1. Detection Range**
```bash
# Configure detection range with predefined presets
--presence-range <RANGE>

# Available options:
--presence-range short    # 6cm to 70cm (close proximity)
--presence-range medium   # 20cm to 2m (balanced)
--presence-range long     # 50cm to 7m (maximum range - default)
```

#### **2. Detection Sensitivity**
```bash
# Adjust detection sensitivity threshold
--sensitivity <VALUE>

# Sensitivity values:
# 0.1 = Low sensitivity (fewer false positives)
# 0.5 = Medium sensitivity  
# 2.0 = High sensitivity (more responsive)
# Default: 1.3 (intra), 1.0 (inter)
```

#### **3. Frame Rate Configuration**
```bash
# Set measurement frequency in Hz
--frame-rate <HZ>

# Examples:
--frame-rate 12.0    # 12 measurements per second (default)
--frame-rate 20.0    # Higher frequency for faster response
--frame-rate 5.0     # Lower frequency for power saving
```

### **Presence Configuration Examples**

#### **Basic Presence Detection**
```bash
# Simple presence test with defaults
sudo xm125-radar-monitor presence

# Presence with verbose logging
sudo xm125-radar-monitor --verbose --mode presence presence
```

#### **Custom Range Configuration**
```bash
# Short range for close proximity detection
sudo xm125-radar-monitor --mode presence presence --presence-range short

# Long range for room occupancy detection  
sudo xm125-radar-monitor --mode presence presence --presence-range long
```

#### **Advanced Presence Configuration**
```bash
# High sensitivity, medium range, fast sampling
sudo xm125-radar-monitor --mode presence presence \
  --presence-range medium \
  --sensitivity 2.0 \
  --frame-rate 20.0

# Low sensitivity, long range, power-efficient
sudo xm125-radar-monitor --mode presence presence \
  --presence-range long \
  --sensitivity 0.5 \
  --frame-rate 5.0
```

#### **Continuous Monitoring with Configuration**
```bash
# Monitor presence with custom settings and save to CSV
sudo xm125-radar-monitor --mode presence monitor \
  --presence-range medium \
  --sensitivity 1.5 \
  --frame-rate 15.0 \
  --count 100 \
  --interval 500 \
  --save-to presence_data.csv
```

### **Range Comparison**

| **Range** | **Distance** | **Best For** | **Power** | **Sensitivity** |
|-----------|--------------|--------------|-----------|-----------------|
| **Short** | 6cm - 70cm | Close proximity, desk sensors | Low | High |
| **Medium** | 20cm - 2m | Personal space, small rooms | Medium | Balanced |
| **Long** | 50cm - 7m | Room occupancy, large spaces | Higher | Lower |

### **Default Presence Settings**

| **Parameter** | **Default Value** | **Description** |
|---------------|-------------------|-----------------|
| **Range** | `long` | 50cm to 7m detection range |
| **Intra Threshold** | `1.3` | Fast motion detection sensitivity |
| **Inter Threshold** | `1.0` | Slow motion detection sensitivity |
| **Frame Rate** | `12.0 Hz` | 12 measurements per second |
| **Sweeps per Frame** | `16` | Signal processing parameter |

## General Configuration

Default settings optimized for Sentai hardware:
- I2C bus: 2
- Device address: 0x52
- Auto-reconnect: enabled
- GPIO pins: 124 (reset), 125 (interrupt), 139 (wake), 141 (boot)

Override via CLI arguments:
```bash
# I2C Configuration
sudo ./xm125-radar-monitor -b 1 -a 0x53 --no-auto-reconnect status

# Custom GPIO pins for different hardware
sudo ./xm125-radar-monitor --gpio-reset 100 --gpio-boot 101 --gpio-wake 102 --gpio-mcu-int 103 status

# GPIO control without external scripts
sudo ./xm125-radar-monitor gpio init

# Debug all register settings after configuration (for evaluation tool comparison)
sudo ./xm125-radar-monitor --debug-registers --mode presence presence
```

### **GPIO Control Commands**

The tool includes internal GPIO control for hardware management without external script dependencies:

```bash
# Initialize GPIO pins and show status
sudo ./xm125-radar-monitor gpio init

# Show current GPIO pin status
sudo ./xm125-radar-monitor gpio status

# Reset XM125 to run mode
sudo ./xm125-radar-monitor gpio reset-run

# Reset XM125 to bootloader mode (for firmware programming)
sudo ./xm125-radar-monitor gpio reset-bootloader

# Test bootloader control functionality
sudo ./xm125-radar-monitor gpio test
```

**GPIO Pin Configuration (Sentai defaults):**
- Reset (GPIO124): `GPIO4_IO28` - Active-low reset control
- MCU Int (GPIO125): `GPIO4_IO29` - Module ready signal input
- Wake Up (GPIO139): `GPIO5_IO11` - Wake up control output
- Boot Pin (GPIO141): `GPIO5_IO13` - Bootloader control (HIGH=bootloader, LOW=run)

**Features:**
- Platform-independent GPIO control via Linux sysfs interface
- Automatic Foundries.io SPI conflict resolution for GPIO141
- Proper STM32 reset timing sequences (10ms assert, 100ms startup)
- Comprehensive error handling and status reporting

### **Register Debugging**

For comparison with other evaluation tools, the `--debug-registers` option logs all module register settings after configuration:

```bash
# Debug presence detector registers
sudo ./xm125-radar-monitor --debug-registers --mode presence presence

# Debug distance detector registers  
sudo ./xm125-radar-monitor --debug-registers --mode distance measure

# Debug breathing detector registers
sudo ./xm125-radar-monitor --debug-registers --mode breathing breathing
```

**Example Register Debug Output:**
```
================================================================================
XM125 Register Dump - Presence Mode
================================================================================

📊 Common Status & Control Registers:
────────────────────────────────────────────────────────────────────────────────
  Addr   (Dec) │ Register Name             │ Value (Hex)  (Decimal) │ Description
────────────────────────────────────────────────────────────────────────────────
  0x0000 (  0) │ Module Version            │ 0x00010203 (     66051) │ Hardware/firmware version info
  0x0001 (  1) │ Protocol Status           │ 0x000000FF (        255) │ Communication protocol status
  0x0002 (  2) │ Measure Counter           │ 0x00000012 (         18) │ Number of measurements performed
  0x0003 (  3) │ Detector Status           │ 0x000000FF (        255) │ Current detector state and flags

👤 Presence Detector Configuration:
────────────────────────────────────────────────────────────────────────────────
  0x0040 ( 64) │ Start Range               │ 0x000001F4 (        500) │ Presence detection start distance (mm)
  0x0041 ( 65) │ End Range                 │ 0x00001B58 (       7000) │ Presence detection end distance (mm)
  0x0042 ( 66) │ Intra Threshold           │ 0x00000514 (       1300) │ Fast motion detection threshold
  0x0043 ( 67) │ Inter Threshold           │ 0x000003E8 (       1000) │ Slow motion detection threshold
  0x0044 ( 68) │ Frame Rate                │ 0x00002EE0 (      12000) │ Measurement frequency (mHz)
================================================================================
```

This output is ideal for:
- **Verification**: Compare settings with Acconeer evaluation tools
- **Debugging**: Identify configuration mismatches
- **Documentation**: Record exact register values for reproducible setups
- **Development**: Validate parameter mapping from CLI to hardware registers

## Output Formats

- **Human**: Readable output with units (default)
- **JSON**: Structured data for APIs
- **CSV**: Data analysis and logging

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Device not found | Check I2C bus with `i2cdetect -y 2` |
| Permission denied | Run with `sudo` for I2C access |
| Unknown command errors | Device needs reset via GPIO control |
| Calibration timeout | Check hardware connections |

Use `--verbose` for detailed I2C transaction logs.

## Dependencies

- `stm32flash`: Firmware programming
- `i2cdetect`, `i2cget`: I2C utilities  
- GPIO sysfs interface (Linux kernel)
- **Internal GPIO Control**: No external scripts required (replaces `xm125-control.sh`)

## Future Applications

The XM125 hardware supports additional detection modes available in Acconeer's reference implementations:

### High-Priority Candidates

| Application | Description | Use Cases |
|-------------|-------------|-----------|
| **Smart Presence** | Multi-zone detection with configurable areas | Smart lighting, HVAC, security systems |
| **Tank Level** | Liquid level measurement with size-specific configs | Industrial monitoring, fuel/water tanks |
| **Parking Detection** | Vehicle presence with obstruction filtering | Smart parking, garage automation |
| **Hand Motion** | Gesture recognition for touchless interfaces | Faucets, UI controls, accessibility |
| **Vibration Monitoring** | Frequency analysis with displacement measurement | Machinery health, structural monitoring |

### Advanced Features

- **Low Power Modes**: Hibernate/sleep for battery applications
- **IQ Data Processing**: Raw signal access for custom algorithms  
- **Multi-Configuration**: Dynamic parameter switching
- **Surface Velocity**: Doppler-based motion measurement
- **Calibration Caching**: Faster startup with stored calibrations

### Implementation Approach

Future modes would follow the established architecture:
- Modular firmware binaries (`*.bin`)
- I2C register protocol extensions
- CLI integration with `--mode <type>`
- Unified monitoring and output formats
- Automatic firmware management via `stm32flash`

*Contributions welcome - see existing distance/presence/breathing implementations as reference.*

## License

Licensed under GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.