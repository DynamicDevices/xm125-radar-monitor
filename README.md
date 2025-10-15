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

## Configuration

Default settings optimized for Sentai hardware:
- I2C bus: 2
- Device address: 0x52
- Auto-reconnect: enabled
- Presence range: Long (0.5-7m)

Override via CLI arguments:
```bash
sudo ./xm125-radar-monitor -b 1 -a 0x53 --no-auto-reconnect status
```

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
- GPIO sysfs interface
- Control script: `/home/fio/xm125-control.sh`

## License

Licensed under GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.