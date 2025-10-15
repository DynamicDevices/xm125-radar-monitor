# Changelog

## [1.0.0] - 2025-10-15

### Production Release
- **Complete XM125 Radar Monitor**: Production-ready CLI tool for Acconeer XM125 radar modules
- **Automatic Firmware Management**: Seamless firmware updates with `stm32flash` integration
- **Multi-Mode Detection**: Distance, presence, and breathing detection with auto-switching
- **Hardware Control**: GPIO-based reset, bootloader, and wake control
- **Comprehensive CI/CD**: GitHub Actions with cross-compilation, testing, and security audits
- **Professional Documentation**: Engineer-focused guides with troubleshooting and examples
- **GPL v3 License**: Open source with proper copyright and maintainer information

### Key Features
- **Auto-Reconnect**: Robust I2C communication with automatic recovery
- **Firmware Verification**: MD5 checksums and application ID validation  
- **Hardware Test Suite**: Automated testing framework with data analysis
- **Cross-Platform**: Native x86_64 and ARM64 builds with optimized profiles
- **Security Audited**: Regular vulnerability scanning and dependency updates

### Technical Excellence
- **Zero Warnings**: Strict Clippy compliance with comprehensive error handling
- **Optimized Builds**: LTO-enabled release builds with minimal binary size
- **Complete Coverage**: Unit tests, integration tests, and hardware validation
- **Professional Standards**: Proper licensing, copyright, and maintainer information

## [0.2.0] - 2025-10-15

### Added
- **Automatic Firmware Management**: Complete integration with `stm32flash` for seamless firmware updates
- **Multi-Firmware Support**: Distance, presence, and breathing detector firmware types
- **GPIO Control**: Hardware reset and bootloader mode switching via GPIO pins
- **Auto-Update Mode**: Automatically updates firmware when detector mode doesn't match
- **Firmware Verification**: MD5 checksum validation and application ID verification
- **Enhanced CLI**: Comprehensive help system with examples and troubleshooting
- **Hardware Test Suite**: Automated testing with data analysis and reporting

### Improved
- **Error Handling**: Robust I2C communication with auto-reconnect capability
- **Presence Detection**: Corrected register parsing and status interpretation
- **Calibration**: Reduced timeout and improved error messages
- **Connection Persistence**: Maintains connection state across command executions
- **Configuration Logging**: Detailed startup configuration display

### Fixed
- **Byte Order Issues**: Corrected endianness for all register operations
- **Status Flag Interpretation**: Updated to match official Acconeer specifications
- **Compiler Warnings**: All Clippy warnings resolved with proper annotations
- **Temperature Handling**: Removed invalid temperature readings from presence mode

### Technical
- **Code Quality**: Comprehensive linting with strict warning-as-error policy
- **Documentation**: Engineer-focused documentation with minimal token usage
- **Project Structure**: Cleaned up obsolete files and reorganized documentation
- **Build System**: Optimized cross-compilation for ARM64 targets

## [0.1.0] - 2025-10-01

### Initial Release
- Basic XM125 radar communication via I2C
- Distance and presence detection modes
- CLI interface with multiple output formats
- Cross-compilation support for ARM64
- Basic error handling and logging