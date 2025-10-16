# XM125 Radar Monitor - Project Context

**Version**: v1.1.0  
**Status**: Production Ready  
**Last Updated**: October 2025

## 🎯 Project Overview

The XM125 Radar Monitor is a production-ready CLI tool for comprehensive testing and monitoring of Acconeer XM125 radar modules. It provides a unified interface for all three major detection modes with automatic firmware management and hardware control.

## 🏗️ Architecture

### Core Components
- **CLI Interface** (`src/cli.rs`): Comprehensive command-line interface with help system
- **Radar Control** (`src/radar.rs`): Multi-mode detection implementation with register protocols
- **I2C Communication** (`src/i2c.rs`): Low-level hardware interface with GPIO control
- **Firmware Management** (`src/firmware.rs`): Automatic firmware detection and updates
- **Error Handling** (`src/error.rs`): Comprehensive error types and recovery

### Detection Modes
1. **Distance Detection**: Precise range measurement (0.1-3.0m) with multi-peak analysis
2. **Presence Detection**: Motion/occupancy detection (0.5-7.0m) with sensitivity control
3. **Breathing Detection**: Breathing pattern analysis (0.3-1.5m) with BPM estimation

## 🔧 Technical Implementation

### Hardware Integration
- **I2C Protocol**: Direct register-level communication at 0x52 (run mode) / 0x48 (bootloader)
- **GPIO Control**: Hardware reset (124), interrupt (125), wake (139), bootloader (141)
- **Firmware Management**: Automatic switching between distance/presence/breathing binaries
- **Auto-Recovery**: Automatic reset to run mode when device not detected

### Software Architecture
- **Language**: Rust 2021 edition with strict linting (warnings as errors)
- **Cross-Compilation**: Native ARM64 builds for embedded Linux targets
- **Async Runtime**: Tokio for non-blocking I2C operations
- **CLI Framework**: Clap v4 with comprehensive help and validation
- **Output Formats**: Human-readable, JSON, and CSV export

## 📊 Current Status

### ✅ Completed Features
- **Multi-Mode Detection**: All three detection modes implemented and tested
- **Hardware Validation**: Successfully tested on Sentai i.MX8MM hardware
- **Firmware Management**: Automatic detection and switching between firmware types
- **Documentation**: Comprehensive guides for engineers and technicians
- **CI/CD Pipeline**: Full automated testing with security audits
- **Production Deployment**: Ready for field deployment

### 🎯 Detection Modes Status
| Mode | Implementation | Hardware Testing | Status |
|------|----------------|------------------|---------|
| **Presence** | ✅ Complete | ✅ Validated | 🟢 Production Ready |
| **Distance** | ✅ Complete | ✅ Validated | 🟢 Production Ready |
| **Breathing** | ✅ Complete | ✅ Validated | 🟢 Production Ready |

## 🚀 Key Achievements

### Technical Excellence
- **Zero Warnings**: Strict Clippy compliance with comprehensive error handling
- **Production Quality**: LTO-optimized builds with minimal binary size (2.4MB)
- **Hardware Integration**: Full GPIO control and automatic device recovery
- **Protocol Compliance**: Register-level compatibility with Acconeer specifications

### Documentation & Testing
- **Engineer Documentation**: Technical specifications and implementation details
- **Technician Guides**: Step-by-step testing procedures with expected outputs
- **Automated Testing**: Comprehensive test suite with data analysis
- **Hardware Validation**: Confirmed working on target hardware

## 🔮 Future Roadmap

### High-Priority Extensions (Next 6 months)
1. **Smart Presence**: Multi-zone detection with configurable areas
2. **Tank Level**: Liquid level measurement for industrial applications
3. **Parking Detection**: Vehicle presence with obstruction filtering

### Advanced Features (6-12 months)
4. **Hand Motion**: Gesture recognition for touchless interfaces
5. **Vibration Monitoring**: Frequency analysis for machinery health
6. **Low Power Modes**: Hibernate/sleep for battery applications

### Technical Enhancements
- **Configuration Persistence**: Save/load detector configurations
- **Web Interface**: Browser-based monitoring and configuration
- **Data Logging**: Long-term measurement storage and analysis
- **Multi-Device**: Support for multiple XM125 modules

## 📁 Project Structure

```
xm125-radar-monitor/
├── src/                    # Core application source
│   ├── main.rs            # Application entry point
│   ├── cli.rs             # Command-line interface
│   ├── radar.rs           # Multi-mode detection logic
│   ├── i2c.rs             # Hardware communication
│   ├── firmware.rs        # Firmware management
│   └── error.rs           # Error handling
├── docs/                   # Documentation
│   └── HARDWARE_TEST_GUIDE.md  # Technician testing procedures
├── scripts/               # Build and utility scripts
│   └── check-links.sh     # Documentation link validation
├── build-aarch64.sh       # ARM64 cross-compilation
├── build-native.sh        # Native x86_64 build
├── test_suite.sh          # Automated hardware testing
├── analyze_test_results.py # Test data analysis
└── README.md              # Project overview and usage
```

## 🛠️ Development Workflow

### Build System
```bash
# Native development build
./build-native.sh

# ARM64 production build
./build-aarch64.sh

# Deploy to target
scp target/aarch64-unknown-linux-gnu/release/xm125-radar-monitor user@target:/usr/local/bin/
```

### Testing Strategy
1. **Unit Tests**: Rust built-in testing framework
2. **Integration Tests**: CLI command validation
3. **Hardware Tests**: Automated test suite on target hardware
4. **CI/CD**: GitHub Actions with cross-compilation and security audits

### Quality Assurance
- **Linting**: Clippy pedantic mode with warnings as errors
- **Formatting**: Rustfmt with consistent style
- **Security**: Regular dependency audits
- **Documentation**: Link validation and technical accuracy

## 📞 Support & Maintenance

### Team Information
- **Maintainer**: Alex J Lennon (ajlennon@dynamicdevices.co.uk)
- **Company**: Dynamic Devices Ltd
- **Support**: info@dynamicdevices.co.uk
- **Repository**: https://github.com/DynamicDevices/xm125-radar-monitor

### Deployment Targets
- **Primary**: Sentai i.MX8MM (ARM64 Linux)
- **Secondary**: Any ARM64 Linux with I2C support
- **Development**: x86_64 Linux (limited functionality without hardware)

### License & Copyright
- **License**: GNU General Public License v3.0
- **Copyright**: © 2025 Dynamic Devices Ltd. All rights reserved.
- **Open Source**: Full source code available on GitHub

## 📈 Performance Metrics

### Hardware Requirements
- **CPU**: ARM64 or x86_64 architecture
- **Memory**: <10MB runtime footprint
- **Storage**: 2.4MB binary size
- **I2C**: Linux I2C subsystem support
- **GPIO**: Sysfs GPIO interface for hardware control

### Performance Characteristics
- **Startup Time**: <1 second to ready state
- **Measurement Rate**: Up to 100Hz (limited by firmware)
- **Accuracy**: ±5cm at <1m, ±10cm at 1-3m (distance mode)
- **Range**: 0.1-3.0m (distance), 0.5-7.0m (presence), 0.3-1.5m (breathing)

## 🎓 Key Learnings

### Technical Insights
1. **Endianness Matters**: XM125 uses big-endian for all multi-byte registers
2. **GPIO Timing**: Hardware reset requires proper timing sequences
3. **Firmware Compatibility**: Each detection mode requires specific firmware
4. **I2C Reliability**: Auto-reconnect essential for production deployment

### Development Best Practices
1. **Register Documentation**: Official Acconeer specs are authoritative
2. **Hardware Testing**: Early and frequent testing on target hardware
3. **Error Recovery**: Robust error handling with automatic recovery
4. **Documentation**: Engineer and technician audiences need different approaches

### Project Management
1. **Incremental Development**: Start with working baseline (presence detection)
2. **Comprehensive Testing**: Hardware validation critical for embedded systems
3. **Documentation First**: Good docs enable team collaboration
4. **CI/CD Investment**: Automated testing prevents regression issues

---

**This document serves as the definitive reference for project context, architecture, and development practices for the XM125 Radar Monitor project.**
