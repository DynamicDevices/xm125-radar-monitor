# XM125 Radar Monitor

[![Build Status](https://github.com/DynamicDevices/xm125-radar-monitor/workflows/CI/badge.svg)](https://github.com/DynamicDevices/xm125-radar-monitor/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-ARM64%20Linux-green.svg)](https://github.com/DynamicDevices/xm125-radar-monitor)

Production-ready CLI tool for Acconeer XM125 radar modules with automatic firmware management and comprehensive configuration options.

**Version**: 1.7.1  
**Maintainer**: Alex J Lennon (ajlennon@dynamicdevices.co.uk)  
**Copyright**: © 2025 Dynamic Devices Ltd. All rights reserved.

## Features

- **Multi-mode Detection**: Distance, presence, and breathing detection
- **Automatic Firmware Management**: Auto-detects and updates firmware via `stm32flash`
- **Comprehensive Configuration**: Direct parameter control for presence detection
- **Continuous Monitoring**: Unified single/continuous operation with CSV export
- **Internal GPIO Control**: Hardware reset and bootloader control without external scripts
- **I2C Communication**: Direct hardware interface with robust error handling
- **Register Debugging**: Complete register dumps for evaluation tool comparison
- **Cross-compilation**: Native ARM64 builds for embedded targets

## Quick Start

```bash
# Check device status
sudo xm125-radar-monitor status

# Basic presence detection (single measurement, default long range)
sudo xm125-radar-monitor presence

# Continuous monitoring with custom range and register debug
sudo xm125-radar-monitor --debug-registers presence --min-range 0.3 --max-range 5.0 --continuous --count 100 --interval 500

# Infinite monitoring with CSV export
sudo xm125-radar-monitor presence --presence-range long --continuous --save-to occupancy.csv
```

## Hardware Requirements

- **Target**: ARM64 Linux (tested on Sentai i.MX8MM)
- **I2C**: `/dev/i2c-2` at address `0x52` (run mode) / `0x48` (bootloader)
- **GPIO**: 124 (reset), 125 (interrupt), 139 (wake), 141 (boot)
- **Firmware**: `/lib/firmware/acconeer/*.bin`

## Presence Detection Configuration

### Range Options

```bash
# Preset ranges (default: long)
--presence-range short    # 6cm - 70cm (close proximity)
--presence-range medium   # 20cm - 2m (balanced)
--presence-range long     # 50cm - 7m (room occupancy, default)

# Custom ranges (both required, conflicts with presets)
--min-range 0.3 --max-range 5.0   # Custom 30cm - 5m range
```

### Detection Parameters

```bash
# Sensitivity control (0.1 = low, 5.0 = high)
--sensitivity 1.5

# Frame rate control (1.0 - 60.0 Hz)
--frame-rate 20.0
```

### Continuous Monitoring

```bash
# Enable continuous mode
--continuous

# Number of measurements (omit for infinite)
--count 100

# Measurement interval in milliseconds
--interval 500

# Save to CSV file
--save-to presence_data.csv
```

## Complete Usage Examples

### Single Measurements

```bash
# Basic presence detection (default long range: 0.5m - 7.0m)
sudo xm125-radar-monitor presence

# High-sensitivity close proximity detection
sudo xm125-radar-monitor presence --presence-range short --sensitivity 2.5

# Custom range with balanced settings
sudo xm125-radar-monitor presence --min-range 0.5 --max-range 3.0 --sensitivity 1.2
```

### Continuous Monitoring

```bash
# Continuous monitoring with register debugging
sudo xm125-radar-monitor --debug-registers presence --min-range 0.3 --max-range 5.0 --continuous --count 100 --interval 500

# Long range room occupancy monitoring with CSV output
sudo xm125-radar-monitor presence --presence-range long --continuous --save-to occupancy.csv

# Power-efficient infinite monitoring (2 second intervals)
sudo xm125-radar-monitor presence --presence-range long --frame-rate 5.0 --continuous --interval 2000

# High-frequency monitoring for 50 measurements
sudo xm125-radar-monitor presence --presence-range short --sensitivity 2.0 --continuous --count 50 --interval 200
```

## Firmware Management

```bash
# Check current firmware
sudo xm125-radar-monitor firmware check

# Update to presence detector firmware
sudo xm125-radar-monitor firmware update presence

# Verify firmware integrity
sudo xm125-radar-monitor firmware verify

# Erase chip (requires confirmation)
sudo xm125-radar-monitor firmware erase --confirm

# Calculate firmware checksums
sudo xm125-radar-monitor firmware checksum --verbose
```

## GPIO Control

Internal GPIO management without external script dependencies:

```bash
# Initialize GPIO pins
sudo xm125-radar-monitor gpio init

# Show GPIO status
sudo xm125-radar-monitor gpio status

# Reset to run mode
sudo xm125-radar-monitor gpio reset-run

# Reset to bootloader mode
sudo xm125-radar-monitor gpio reset-bootloader
```

## Register Debugging

Compare configuration with Acconeer evaluation tools:

```bash
# Debug presence registers with default configuration
sudo xm125-radar-monitor --debug-registers presence

# Debug with custom configuration
sudo xm125-radar-monitor --debug-registers presence --min-range 0.3 --max-range 5.0 --sensitivity 1.8

# Debug during continuous monitoring
sudo xm125-radar-monitor --debug-registers presence --presence-range medium --continuous --count 10
```

**Example Register Output:**
```
================================================================================
XM125 Register Dump - Presence Mode
================================================================================

👤 Presence Detector Configuration:
────────────────────────────────────────────────────────────────────────────────
  0x0040 ( 64) │ Start Range               │ 0x0000012C (        300) │ Presence detection start distance (mm)
  0x0041 ( 65) │ End Range                 │ 0x00001388 (       5000) │ Presence detection end distance (mm)
  0x0042 ( 66) │ Intra Threshold           │ 0x00000708 (       1800) │ Fast motion detection threshold
  0x0043 ( 67) │ Inter Threshold           │ 0x00000640 (       1600) │ Slow motion detection threshold
================================================================================
```

## Detection Modes

| Mode | Range | Update Rate | Primary Use |
|------|-------|-------------|-------------|
| **Distance** | 0.1-3.0m | ~100ms | Precise range measurement |
| **Presence** | 0.06-7.0m | ~100ms | Motion/occupancy detection |
| **Breathing** | 0.3-1.5m | 5-20s | Breathing pattern analysis |

## Configuration Options

### I2C & Hardware

```bash
# Custom I2C bus and address
sudo xm125-radar-monitor -b 1 -a 0x53 status

# Custom GPIO pins for different hardware
sudo xm125-radar-monitor --gpio-reset 100 --gpio-boot 101 status

# Disable auto-reconnect for debugging
sudo xm125-radar-monitor --no-auto-reconnect status
```

### Output Formats

```bash
# Human-readable (default)
sudo xm125-radar-monitor presence

# JSON output for APIs
sudo xm125-radar-monitor --format json presence

# CSV output for data analysis
sudo xm125-radar-monitor --format csv presence
```

## Build & Deploy

```bash
# Cross-compile for ARM64
./build-aarch64.sh

# Deploy to target
scp target/aarch64-unknown-linux-gnu/release/xm125-radar-monitor user@target:/usr/local/bin/
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Device not found | `i2cdetect -y 2` to verify I2C bus |
| Permission denied | Run with `sudo` for I2C/GPIO access |
| Unknown command errors | Reset device: `sudo xm125-radar-monitor gpio reset-run` |
| Calibration timeout | Check hardware connections and power |
| Firmware update fails | Ensure device in bootloader mode: `sudo xm125-radar-monitor bootloader` |
| Register values incorrect | Use `--debug-registers` to verify configuration is applied |

Use `--verbose` for detailed I2C transaction logs and debugging information.

## Dependencies

- **Runtime**: `stm32flash`, `i2cdetect`, `i2cget`
- **Build**: Rust 1.70+, cross-compilation toolchain for ARM64, `csv` crate
- **Hardware**: Linux GPIO sysfs interface

## License

Licensed under GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

---

**Keywords**: Acconeer XM125, radar sensor, I2C communication, firmware management, distance detection, presence detection, embedded Linux, ARM64, cross-compilation, stm32flash, continuous monitoring, CSV export