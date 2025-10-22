# XM125 Radar Monitor

[![Build Status](https://github.com/DynamicDevices/xm125-radar-monitor/workflows/CI/badge.svg)](https://github.com/DynamicDevices/xm125-radar-monitor/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-ARM64%20Linux-green.svg)](https://github.com/DynamicDevices/xm125-radar-monitor)

Production-ready CLI tool for Acconeer XM125 radar modules with automatic firmware management and comprehensive configuration options.

**Version**: 1.7.0  
**Maintainer**: Alex J Lennon (ajlennon@dynamicdevices.co.uk)  
**Copyright**: © 2025 Dynamic Devices Ltd. All rights reserved.

## Features

- **Multi-mode Detection**: Distance, presence, and breathing detection
- **Automatic Firmware Management**: Auto-detects and updates firmware via `stm32flash`
- **Comprehensive Configuration**: Direct parameter control for presence detection
- **Internal GPIO Control**: Hardware reset and bootloader control without external scripts
- **I2C Communication**: Direct hardware interface with robust error handling
- **Real-time Monitoring**: Continuous measurements with CSV export
- **Register Debugging**: Complete register dumps for evaluation tool comparison
- **Cross-compilation**: Native ARM64 builds for embedded targets

## Quick Start

```bash
# Check device status
sudo xm125-radar-monitor status

# Basic presence detection
sudo xm125-radar-monitor presence

# Presence with long range and high sensitivity
sudo xm125-radar-monitor presence --presence-range long --sensitivity 2.0

# Custom range configuration
sudo xm125-radar-monitor presence --min-range 0.3 --max-range 5.0 --frame-rate 20.0

# Continuous monitoring with CSV output
sudo xm125-radar-monitor monitor --count 100 --save-to data.csv
```

## Hardware Requirements

- **Target**: ARM64 Linux (tested on Sentai i.MX8MM)
- **I2C**: `/dev/i2c-2` at address `0x52` (run mode) / `0x48` (bootloader)
- **GPIO**: 124 (reset), 125 (interrupt), 139 (wake), 141 (boot)
- **Firmware**: `/lib/firmware/acconeer/*.bin`

## Presence Detection Configuration

### Range Options

```bash
# Preset ranges
--presence-range short    # 6cm - 70cm (close proximity)
--presence-range medium   # 20cm - 2m (balanced)
--presence-range long     # 50cm - 7m (room occupancy)

# Custom ranges (both required)
--min-range 0.3 --max-range 5.0   # Custom 30cm - 5m range
```

### Detection Parameters

```bash
# Sensitivity control (0.1 = low, 5.0 = high)
--sensitivity 1.5

# Frame rate control (1.0 - 60.0 Hz)
--frame-rate 20.0
```

### Complete Examples

```bash
# High-sensitivity close proximity detection
sudo xm125-radar-monitor presence --presence-range short --sensitivity 2.5

# Power-efficient room occupancy monitoring
sudo xm125-radar-monitor presence --presence-range long --sensitivity 0.8 --frame-rate 5.0

# Custom range with balanced settings
sudo xm125-radar-monitor presence --min-range 0.5 --max-range 3.0 --sensitivity 1.2 --frame-rate 15.0
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
# Debug presence registers
sudo xm125-radar-monitor --debug-registers presence

# Debug with custom configuration
sudo xm125-radar-monitor --debug-registers presence --presence-range medium --sensitivity 1.8
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

Use `--verbose` for detailed I2C transaction logs and debugging information.

## Dependencies

- **Runtime**: `stm32flash`, `i2cdetect`, `i2cget`
- **Build**: Rust 1.70+, cross-compilation toolchain for ARM64
- **Hardware**: Linux GPIO sysfs interface

## License

Licensed under GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

---

**Keywords**: Acconeer XM125, radar sensor, I2C communication, firmware management, distance detection, presence detection, embedded Linux, ARM64, cross-compilation, stm32flash