# XM125 Radar Monitor

A Rust application for reading distance measurements from the Acconeer XM125 radar module via I2C communication.

## Features

- **Real-time Distance Monitoring**: Continuous distance measurements with configurable intervals
- **I2C Communication**: Direct communication with XM125 via Linux I2C interface
- **Multiple Output Formats**: Human-readable, JSON, and CSV output formats
- **Cross-Platform**: Supports native x86_64 and cross-compilation for ARM64/AArch64
- **Robust Error Handling**: Comprehensive error handling for I2C communication issues
- **Automatic Calibration**: Handles sensor calibration automatically
- **CLI Interface**: Full command-line interface with multiple commands

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

# Connect to device
./target/release/xm125-radar-monitor connect

# Get device information
./target/release/xm125-radar-monitor info

# Single distance measurement
./target/release/xm125-radar-monitor measure

# Calibrate the sensor
./target/release/xm125-radar-monitor calibrate

# Continuous monitoring (1 second interval)
./target/release/xm125-radar-monitor monitor --interval 1000

# Monitor with specific count and custom I2C settings
./target/release/xm125-radar-monitor -d /dev/i2c-1 -a 0x52 monitor --count 100 --interval 500
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

- **I2C Device**: `-d, --i2c-device` (default: /dev/i2c-1)
- **I2C Address**: `-a, --i2c-address` (default: 0x52)
- **Output Format**: `-f, --format` (human, json, csv)
- **Verbose Logging**: `-v, --verbose`
- **Quiet Mode**: `-q, --quiet`

## XM125 Integration

This application implements the XM125 I2C protocol based on Acconeer's documentation:

### Register Map
- **Command Register**: 0x0000 - Send commands to XM125
- **Status Register**: 0x0002 - Read device status
- **Distance Result**: 0x0100 - Read measurement results
- **Configuration**: 0x0200 - Device configuration
- **Sensor Info**: 0x0300 - Device information

### Communication Protocol
1. **Initialization**: Check device presence and status
2. **Calibration**: Perform sensor calibration (automatic)
3. **Measurement**: Send measurement command and read results
4. **Error Handling**: Monitor status for errors and calibration needs

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
├── main.rs           # Application entry point
├── cli.rs            # Command-line interface
├── error.rs          # Error handling
├── i2c.rs            # I2C communication layer
└── radar.rs          # XM125 radar interface

references/           # XM125 documentation
├── doc/              # Acconeer documentation
├── XM125-datasheet.pdf
└── README.txt
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

Commercial License - see LICENSE file for details.

## Copyright

Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.

## Contact

For licensing, support, or commercial inquiries:
- **Email**: info@dynamicdevices.co.uk
- **Author**: Alex J Lennon <ajlennon@dynamicdevices.co.uk>
- **Company**: Dynamic Devices Ltd
