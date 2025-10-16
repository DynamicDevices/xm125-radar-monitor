# Changelog

## [1.5.0] - 2025-10-16

### Control Script Installation & Path Fixes
- **🔧 Fixed Bootloader Command**: Restored missing `xm125-control.sh` and `xm125-firmware-flash.sh` scripts to Yocto recipe
- **📁 Standard Path Installation**: Scripts now install to `/usr/bin/` instead of user home directories
- **🚫 Removed Confusing Symlinks**: Eliminated symlinks that pointed to Rust binary instead of shell scripts
- **🛡️ Enhanced Error Handling**: Added comprehensive validation when control script is missing or not executable
- **⚡ Early Command Processing**: Handle bootloader command before I2C initialization to avoid permission errors

### Technical Improvements
- **📍 Updated Script Paths**: Changed hardcoded `/home/fio/` paths to standard `/usr/bin/` locations
- **🔍 Script Validation**: Check script existence and executable permissions with clear error messages
- **📋 Installation Instructions**: Provide helpful troubleshooting guidance when scripts are missing
- **🎯 Proper Error Propagation**: Improved error handling throughout the application

### Yocto Recipe Updates
- **📦 Script Installation**: Added `xm125-control.sh` and `xm125-firmware-flash.sh` to recipe sources
- **🗂️ Clean Package Structure**: Removed symlink confusion and properly package actual scripts
- **✅ Production Ready**: Scripts installed with correct permissions in standard system locations

## [1.4.0] - 2025-10-16

### Status Command Enhancement & User Experience Improvements
- **🎯 Smart Status Detection**: Status command now automatically detects and displays actual firmware mode instead of CLI default
- **🔧 Default Mode Optimization**: Changed default mode from distance to presence to match typical hardware configurations
- **📋 Improved Banner Logic**: Enhanced startup banner with accurate mode detection and cleaner output
- **🚀 Seamless User Experience**: Eliminates confusion between CLI defaults and actual device firmware

### Technical Improvements
- **✅ Firmware Auto-Detection**: Reads Application ID register to determine installed firmware type
- **🔄 Graceful Fallbacks**: Falls back to CLI mode if firmware detection fails
- **📊 Accurate Status Reporting**: Status command shows "Presence Detector" when presence firmware is loaded
- **🎨 Clean Output**: Prevents duplicate banners and provides consistent user interface

### Deployment & Testing
- **✅ Hardware Validated**: Successfully tested on Sentai target hardware
- **🔧 Production Ready**: Deployed as v1.4.0 with comprehensive functionality testing
- **📈 Improved Defaults**: Presence detection works out-of-the-box without mode specification

## [1.1.0] - 2025-10-16

### Complete Multi-Mode Detection Suite
- **🫁 Breathing Detection**: Full implementation of breathing pattern analysis with BPM estimation
- **📏 Distance Detection**: Precise range measurement with multi-peak analysis and CFAR processing
- **👁️ Enhanced Presence Detection**: Improved motion detection with configurable sensitivity
- **🔄 Unified CLI**: Seamless switching between all three detection modes with `--mode` parameter

### Advanced Features
- **📊 Comprehensive Documentation**: Detection modes comparison table and future applications roadmap
- **🧪 Hardware Testing Guide**: Detailed technician procedures with expected outputs and troubleshooting
- **🔧 Firmware Management**: Automatic detection and switching between distance/presence/breathing binaries
- **📈 Continuous Monitoring**: Real-time measurement streaming with CSV export capabilities
- **🎛️ Configuration Logging**: Detailed startup configuration display for all detection modes

### Technical Achievements
- **✅ Complete CI/CD**: All automated tests passing with comprehensive quality checks
- **🚀 Production Deployment**: Successfully tested on Sentai hardware with all three modes
- **📋 Professional Documentation**: Engineer-focused guides with practical testing workflows
- **🔍 Hardware Validation**: Confirmed working on target hardware with GPIO control and I2C communication

### Detection Modes Status
- **✅ Presence Detection**: Fully tested and working (baseline functionality)
- **✅ Distance Detection**: Implemented and hardware validated
- **✅ Breathing Detection**: Implemented and hardware validated
- **🎯 Future Modes**: Roadmap for 10+ additional detection modes (smart presence, tank level, parking, etc.)

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