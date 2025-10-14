# XM125 Radar Monitor

A Rust application for reading distance measurements and presence detection from the Acconeer XM125 radar module via I2C communication.

## Features

- **Real-time Distance Monitoring**: Continuous distance measurements with configurable intervals
- **Presence Detection**: Advanced presence sensing with intra/inter-presence scores
- **Combined Measurements**: Simultaneous distance and presence detection
- **I2C Communication**: Direct communication with XM125 via Linux I2C interface with flexible bus/address configuration
- **Multiple Output Formats**: Human-readable, JSON, and CSV output formats
- **Cross-Platform**: Supports native x86_64 and cross-compilation for ARM64/AArch64
- **Robust Error Handling**: Comprehensive error handling for I2C communication issues
- **Automatic Connection Management**: Auto-reconnect functionality with retry logic
- **Automatic Calibration**: Handles sensor calibration automatically
- **CLI Interface**: Full command-line interface with multiple detector modes and commands

## Hardware Requirements

- **XM125 Radar Module**: Acconeer A121-based radar sensor
- **I2C Interface**: Linux system with I2C support (e.g., Raspberry Pi, embedded Linux)
- **Connection**: XM125 connected to I2C bus (typically /dev/i2c-1)

## Installation

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cross-compilation tools (for ARM64 targets)
sudo apt install gcc-aarch64-linux-gnu

# Add ARM64 target
rustup target add aarch64-unknown-linux-gnu
```

### Build

```bash
# Native build (x86_64)
cargo build --release

# Cross-compile for ARM64
./build-aarch64.sh
```

## Usage

### Basic Commands

```bash
# Get device status
./target/release/xm125-radar-monitor status

# Connect to device (with auto-reconnect)
./target/release/xm125-radar-monitor --auto-reconnect connect --force

# Get device information
./target/release/xm125-radar-monitor info

# Single distance measurement
./target/release/xm125-radar-monitor measure

# Single presence detection
./target/release/xm125-radar-monitor presence

# Combined distance and presence measurement
./target/release/xm125-radar-monitor combined

# Calibrate the sensor
./target/release/xm125-radar-monitor calibrate

# Configure detector settings
./target/release/xm125-radar-monitor config --start 0.2 --length 1.0 --sensitivity 0.5

# Continuous monitoring (1 second interval)
./target/release/xm125-radar-monitor monitor --interval 1000

# Monitor with specific count and custom I2C settings
./target/release/xm125-radar-monitor -b 1 -a 0x52 monitor --count 100 --interval 500

# Monitor with presence detection mode
./target/release/xm125-radar-monitor --mode presence monitor --interval 2000
```

### Output Formats

```bash
# Human-readable output (default)
./target/release/xm125-radar-monitor measure

# JSON output
./target/release/xm125-radar-monitor --format json measure

# CSV output
./target/release/xm125-radar-monitor --format csv monitor --count 10
```

### Configuration Options

- **I2C Bus**: `-b, --i2c-bus` (specify bus number, e.g., 1 for /dev/i2c-1)
- **I2C Device**: `-d, --i2c-device` (full device path, e.g., /dev/i2c-1)
- **I2C Address**: `-a, --i2c-address` (default: 0x52, supports hex and decimal)
- **Detector Mode**: `--mode` (distance, presence, combined)
- **Auto-reconnect**: `--auto-reconnect` (enable automatic connection retry)
- **Output Format**: `-f, --format` (human, json, csv)
- **Verbose Logging**: `-v, --verbose`
- **Quiet Mode**: `-q, --quiet`

## XM125 Integration

This application implements the XM125 I2C protocol based on Acconeer's documentation:

### Register Map
- **Command Register**: 0x0000 - Send commands to XM125
- **Status Register**: 0x0002 - Read device status  
- **Distance Result**: 0x0100 - Read distance measurement results
- **Presence Result**: 0x0400 - Read presence detection results
- **Distance Configuration**: 0x0200 - Distance detector configuration
- **Presence Configuration**: 0x0500 - Presence detector configuration
- **Sensor Info**: 0x0300 - Device information

### Communication Protocol
1. **Initialization**: Check device presence and status
2. **Configuration**: Set detector mode (distance, presence, or combined)
3. **Calibration**: Perform sensor calibration (automatic or manual)
4. **Measurement**: Send measurement commands and read results
5. **Continuous Mode**: Optional continuous monitoring with configurable intervals
6. **Error Handling**: Monitor status for errors, calibration needs, and connection issues

## Cross-Compilation

For deployment to ARM64 embedded systems:

```bash
# Build for ARM64
./build-aarch64.sh

# Deploy to target (customize as needed)
scp target/aarch64-unknown-linux-gnu/release/xm125-radar-monitor user@target-device:/usr/local/bin/
```

## Development

### Project Structure

```
src/
├── main.rs           # Application entry point and command routing
├── cli.rs            # Command-line interface with clap
├── error.rs          # Centralized error handling  
├── i2c.rs            # I2C communication layer
└── radar.rs          # XM125 radar interface with detector modes

docs/                 # Project documentation
├── DEVELOPMENT.md    # Development tools and Git hooks
└── REFERENCE_POLICY.md # Policy for handling reference documents
```

### Testing

```bash
# Run tests
cargo test

# Run with hardware (requires XM125 connected)
TEST_DEVICE=/dev/i2c-1 cargo test --test integration_tests -- --ignored
```

## Troubleshooting

### Common Issues

1. **Permission Denied**: Add user to `i2c` group
   ```bash
   sudo usermod -a -G i2c $USER
   # Logout and login again
   ```

2. **Device Not Found**: Check I2C device exists
   ```bash
   ls -la /dev/i2c-*
   i2cdetect -y 1  # Scan I2C bus 1
   ```

3. **Communication Errors**: Verify wiring and I2C address
   ```bash
   # Enable verbose logging
   ./xm125-radar-monitor -v status
   ```

### Hardware Connections

Typical XM125 I2C connections:
- **VCC**: 3.3V power supply
- **GND**: Ground
- **SDA**: I2C data line
- **SCL**: I2C clock line
- **Address**: Configure I2C address (default 0x52)

## Documentation

- **[Development Guide](docs/DEVELOPMENT.md)** - Setup, Git hooks, and development workflow
- **[XM125 References](/data_drive/docs/references/)** - Hardware documentation and SDK references

## References

- [Acconeer XM125 Documentation](/data_drive/docs/references/doc/)
- [XM125 Datasheet](/data_drive/docs/references/XM125-datasheet.pdf)
- [A121 Distance Detector User Guide](/data_drive/docs/references/doc/A121%20Distance%20Detector%20User%20Guide.pdf)
- [XM125 Software User Guide](/data_drive/docs/references/doc/XM125%20Software%20User%20Guide.pdf)

## License

This project is licensed under the GNU General Public License v3.0 or later - see the [LICENSE](LICENSE) file for details.

## Copyright

Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.

## Contact

For licensing, support, or commercial inquiries:
- **Email**: info@dynamicdevices.co.uk
- **Author**: Alex J Lennon <ajlennon@dynamicdevices.co.uk>
- **Company**: Dynamic Devices Ltd
