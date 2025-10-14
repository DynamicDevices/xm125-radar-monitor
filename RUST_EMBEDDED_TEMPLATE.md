# 🦀 Rust Embedded Project Template - Context Initializer

**Based on: E-ink Power CLI v2.4.0**  
**Source Project**: Production-ready embedded Rust CLI for MCXC143VFM power management controller  
**Template Version**: 1.0.0  
**Generated**: 2025-01-14  

## 📋 Project Overview

This template is derived from a production-ready embedded Rust project that successfully:

- ✅ **Cross-compiles** for ARM64/AArch64 targets (i.MX93)
- ✅ **Communicates** with microcontrollers over serial UART
- ✅ **Deploys** to embedded Linux systems (Yocto/Foundries.io)
- ✅ **Integrates** with CI/CD pipelines and Docker
- ✅ **Provides** comprehensive CLI interface with multiple output formats
- ✅ **Handles** real-time hardware communication and power management
- ✅ **Supports** production deployment and systemd integration

## 🎯 Use This Template For

### ✅ Perfect Match
- **Embedded Linux CLI tools** (serial communication, GPIO control, sensor monitoring)
- **Hardware interface applications** (power management, device control)
- **Cross-platform embedded utilities** (ARM64, x86_64)
- **IoT device management tools** (remote monitoring, automation)
- **Industrial automation interfaces** (PLCs, controllers, sensors)
- **Embedded system diagnostics** (health monitoring, debugging)

### ⚠️ Consider Alternatives
- **Bare-metal firmware** (use embedded-hal templates instead)
- **Web applications** (use web framework templates)
- **Desktop GUI applications** (use egui/tauri templates)
- **Game development** (use Bevy/Macroquad templates)

## 🏗️ Template Structure

```
your-embedded-project/
├── 📦 Package Configuration
│   ├── Cargo.toml                 # Dependencies, metadata, build config
│   ├── Cargo.lock                 # Locked dependencies
│   └── .cargo/config.toml         # Cross-compilation settings
│
├── 🔧 Build & Deploy
│   ├── build-aarch64.sh           # ARM64 cross-compilation
│   ├── deploy-target.sh           # Target deployment
│   ├── deploy.sh                  # Generic deployment
│   └── dev.sh                     # Development helper script
│
├── 🐳 Docker Development
│   ├── Dockerfile                 # Development container
│   ├── docker-compose.yml         # Multi-service setup
│   └── .dockerignore             # Docker build exclusions
│
├── 📚 Documentation
│   ├── README.md                  # Comprehensive project docs
│   ├── CHANGELOG.md              # Version history
│   ├── IMPLEMENTATION.md         # Technical implementation
│   ├── CROSS_COMPILE.md          # Cross-compilation guide
│   └── LICENSE                   # Commercial/MIT/Apache license
│
├── 🦀 Source Code
│   └── src/
│       ├── main.rs               # CLI entry point & command routing
│       ├── lib.rs                # Library interface
│       ├── error.rs              # Centralized error handling
│       ├── cli/                  # Command-line interface
│       │   ├── mod.rs            # CLI module exports
│       │   ├── commands.rs       # Command definitions
│       │   └── parser.rs         # Argument parsing (optional)
│       ├── serial/               # Hardware communication
│       │   ├── mod.rs            # Serial module exports
│       │   ├── connection.rs     # UART/USB connection management
│       │   └── protocol.rs       # Communication protocol
│       ├── hardware/             # Hardware-specific modules
│       │   ├── mod.rs            # Hardware abstraction
│       │   ├── controller.rs     # Device controller logic
│       │   └── sensors.rs        # Sensor interfaces (optional)
│       └── utils/                # Utilities (optional)
│           ├── mod.rs            # Utility exports
│           ├── config.rs         # Configuration management
│           └── json.rs           # JSON response parsing
│
├── 🧪 Testing
│   ├── tests/
│   │   ├── integration_tests.rs  # Hardware integration tests
│   │   └── mock_hardware.rs      # Mock hardware for CI
│   └── examples/
│       ├── basic_usage.rs        # Library usage example
│       └── automation.rs         # Automation script example
│
└── 🔄 CI/CD
    └── .github/workflows/
        ├── ci.yml                # Main CI pipeline
        └── maintenance.yml       # Dependency updates
```

## 📦 Cargo.toml Template

```toml
[package]
name = "your-embedded-project"
version = "0.1.0"
edition = "2021"
license = "MIT" # or "Apache-2.0" or "Commercial"
description = "Embedded system interface for [YOUR_HARDWARE]"
homepage = "https://github.com/your-org/your-embedded-project"
repository = "https://github.com/your-org/your-embedded-project"
documentation = "https://github.com/your-org/your-embedded-project/blob/main/README.md"
readme = "README.md"
keywords = ["embedded", "hardware", "serial", "cli", "iot"]
categories = ["command-line-utilities", "embedded", "hardware-support"]
authors = ["Your Name <your.email@company.com>"]
publish = false  # Set to true for public crates

[package.metadata]
maintainer = "Your Name <your.email@company.com>"
contact = "info@company.com"
company = "Your Company Ltd"
copyright = "Copyright (c) 2025 Your Company Ltd"

[[bin]]
name = "your-embedded-project"
path = "src/main.rs"

[dependencies]
# CLI and argument parsing
clap = { version = "4.4", features = ["derive", "env", "color"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Serial communication
serialport = { version = "4.2", default-features = false }
tokio = { version = "1.35", features = ["full"] }
tokio-serial = "5.4"

# Error handling and utilities
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
env_logger = "0.10"

# Configuration
config = "0.13"
dirs = "5.0"
toml = "0.8"

# Time and monitoring
chrono = { version = "0.4", features = ["serde"] }
indicatif = "0.17"

# Additional utilities (choose as needed)
regex = "1.10"
uuid = { version = "1.6", features = ["v4"] }

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.11"
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"

[profile.release]
strip = true        # Remove debug symbols for smaller binaries
lto = true         # Link-time optimization
codegen-units = 1  # Better optimization
panic = "abort"    # Smaller binary size

[profile.dev]
debug = true       # Keep debug info in development
```

## 🔧 Cross-Compilation Setup

### .cargo/config.toml
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

[build]
target-dir = "target"

[env]
PKG_CONFIG_ALLOW_CROSS = "1"
```

### build-aarch64.sh
```bash
#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_status "Building for AArch64 (ARM64)"

# Install target if not present
rustup target add aarch64-unknown-linux-gnu

# Build for AArch64 target
PKG_CONFIG_ALLOW_CROSS=1 cargo build --target aarch64-unknown-linux-gnu --release

# Show binary information
BINARY_PATH="target/aarch64-unknown-linux-gnu/release/your-embedded-project"
if [ -f "$BINARY_PATH" ]; then
    print_success "Build completed successfully!"
    echo "Binary: $BINARY_PATH"
    echo "Size: $(ls -lh $BINARY_PATH | awk '{print $5}')"
    echo "Type: $(file $BINARY_PATH)"
else
    echo "Build failed - binary not found"
    exit 1
fi
```

## 🦀 Core Source Templates

### src/main.rs
```rust
use clap::Parser;
use log::{debug, error};
use std::process;

mod cli;
mod error;
mod hardware;
mod serial;

use cli::Cli;
use error::ProjectError;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    if !cli.quiet {
        println!("{} v{}", APP_NAME, VERSION);
        println!("Copyright (c) 2025 Your Company Ltd");
        println!();
    }

    if let Err(e) = run(cli).await {
        error!("Command failed: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), ProjectError> {
    debug!("Starting {} v{}", APP_NAME, VERSION);

    // Create hardware connection
    let connection = serial::Connection::new(&cli.device, cli.baud, cli.quiet)?;
    let mut controller = hardware::Controller::new(connection);

    // Execute command
    match cli.command {
        Some(cmd) => {
            debug!("Executing command: {:?}", cmd);
            execute_command(cmd, &mut controller, &cli).await?;
            Ok(())
        }
        None => {
            println!("No command provided. Use --help for usage information.");
            Ok(())
        }
    }
}

async fn execute_command(
    command: cli::Commands,
    controller: &mut hardware::Controller,
    cli: &Cli,
) -> Result<(), ProjectError> {
    use cli::Commands;

    match command {
        Commands::Status => {
            let response = controller.get_status().await?;
            output_response(cli, "status", &response, "📊", "System Status")?;
        }
        Commands::Connect => {
            let response = controller.connect().await?;
            output_response(cli, "connect", &response, "🔗", "Connection")?;
        }
        // Add your specific commands here
        _ => {
            println!("Command not yet implemented: {:?}", command);
        }
    }

    Ok(())
}

fn output_response(
    cli: &Cli,
    command: &str,
    response: &str,
    emoji: &str,
    title: &str,
) -> Result<(), ProjectError> {
    if cli.quiet {
        return Ok(());
    }

    match cli.format {
        cli::OutputFormat::Human => {
            println!("{} {}:", emoji, title);
            println!("{}", response);
        }
        cli::OutputFormat::Json => {
            let json_response = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "command": command,
                "status": "success",
                "data": response
            });
            println!("{}", serde_json::to_string_pretty(&json_response)?);
        }
        cli::OutputFormat::Csv => {
            println!("timestamp,command,status,response");
            println!(
                "{},{},success,\"{}\"",
                chrono::Utc::now().to_rfc3339(),
                command,
                response.replace("\"", "\"\"")
            );
        }
    }

    Ok(())
}
```

### src/error.rs
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Serial communication error: {0}")]
    Serial(#[from] serialport::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command timeout after {timeout}s")]
    Timeout { timeout: u64 },

    #[error("Invalid response from device: {response}")]
    InvalidResponse { response: String },

    #[error("Device returned error: {message}")]
    DeviceError { message: String },

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Hardware not connected")]
    NotConnected,

    #[error("Invalid command parameters: {0}")]
    InvalidParameters(String),
}

pub type Result<T> = std::result::Result<T, ProjectError>;
```

### src/cli/mod.rs
```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Serial device path
    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    pub device: String,

    /// Baud rate
    #[arg(short, long, default_value_t = 115200)]
    pub baud: u32,

    /// Command timeout in seconds
    #[arg(short, long, default_value_t = 3)]
    pub timeout: u64,

    /// Output format
    #[arg(short, long, default_value = "human")]
    pub format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Get system status
    Status,
    
    /// Connect to device
    Connect,
    
    /// Disconnect from device
    Disconnect,
    
    /// Get device information
    Info,
    
    /// Reset device
    Reset,
    
    // Add your specific commands here based on your hardware
    // Examples:
    // /// Read sensor data
    // ReadSensor {
    //     /// Sensor ID
    //     id: u8,
    // },
    // 
    // /// Control GPIO
    // Gpio {
    //     #[command(subcommand)]
    //     action: GpioCommands,
    // },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Csv,
}

// Example GPIO subcommands
// #[derive(Clone, Debug, Subcommand)]
// pub enum GpioCommands {
//     /// Get GPIO state
//     Get {
//         /// GPIO pin number
//         pin: u8,
//     },
//     /// Set GPIO state
//     Set {
//         /// GPIO pin number
//         pin: u8,
//         /// Value (0 or 1)
//         value: u8,
//     },
// }
```

### src/serial/connection.rs
```rust
use crate::error::{ProjectError, Result};
use log::{debug, warn};
use std::time::Duration;
use tokio::time::timeout;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct Connection {
    device_path: String,
    baud_rate: u32,
    timeout_duration: Duration,
    stream: Option<SerialStream>,
    quiet: bool,
}

impl Connection {
    pub fn new(device_path: &str, baud_rate: u32, quiet: bool) -> Result<Self> {
        Ok(Self {
            device_path: device_path.to_string(),
            baud_rate,
            timeout_duration: Duration::from_secs(3),
            stream: None,
            quiet,
        })
    }

    pub fn set_timeout(&mut self, timeout_secs: u64) {
        self.timeout_duration = Duration::from_secs(timeout_secs);
    }

    pub async fn connect(&mut self) -> Result<()> {
        if !self.quiet {
            debug!("Connecting to {} at {} baud", self.device_path, self.baud_rate);
        }

        let stream = tokio_serial::new(&self.device_path, self.baud_rate)
            .timeout(self.timeout_duration)
            .open_native_async()
            .map_err(|e| {
                warn!("Failed to open serial port {}: {}", self.device_path, e);
                ProjectError::Serial(e)
            })?;

        self.stream = Some(stream);

        if !self.quiet {
            debug!("Successfully connected to {}", self.device_path);
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if self.stream.take().is_some() {
            if !self.quiet {
                debug!("Disconnected from {}", self.device_path);
            }
        }
    }

    pub async fn send_command(&mut self, command: &str) -> Result<String> {
        if self.stream.is_none() {
            return Err(ProjectError::NotConnected);
        }

        // TODO: Implement actual serial communication
        // This is a placeholder - replace with your protocol implementation
        
        debug!("Sending command: {}", command);
        
        // Simulate command execution
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Return mock response - replace with actual protocol parsing
        Ok(format!("Response to: {}", command))
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if self.stream.is_some() {
            debug!("Connection dropped for {}", self.device_path);
        }
    }
}
```

### src/hardware/mod.rs
```rust
pub mod controller;

pub use controller::Controller;

// Add hardware-specific modules here
// pub mod sensors;
// pub mod gpio;
// pub mod power;
```

### src/hardware/controller.rs
```rust
use crate::error::{ProjectError, Result};
use crate::serial::Connection;
use log::debug;

pub struct Controller {
    connection: Connection,
}

impl Controller {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub async fn connect(&mut self) -> Result<String> {
        self.connection.connect().await?;
        Ok("Connected successfully".to_string())
    }

    pub async fn get_status(&mut self) -> Result<String> {
        if !self.connection.is_connected() {
            self.connection.connect().await?;
        }

        let response = self.connection.send_command("status").await?;
        debug!("Status response: {}", response);
        Ok(response)
    }

    pub async fn get_info(&mut self) -> Result<String> {
        let response = self.connection.send_command("info").await?;
        Ok(response)
    }

    pub async fn reset(&mut self) -> Result<String> {
        let response = self.connection.send_command("reset").await?;
        Ok(response)
    }

    // Add your hardware-specific methods here
    // pub async fn read_sensor(&mut self, sensor_id: u8) -> Result<SensorData> { ... }
    // pub async fn control_gpio(&mut self, pin: u8, value: bool) -> Result<()> { ... }
    // pub async fn set_power_mode(&mut self, mode: PowerMode) -> Result<()> { ... }
}
```

## 🐳 Docker Development Environment

### Dockerfile
```dockerfile
FROM rust:1.81-bullseye

LABEL maintainer="Your Name <your.email@company.com>"
LABEL description="Development container for embedded Rust project"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    cmake \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    libudev-dev \
    libssl-dev \
    git \
    curl \
    vim \
    minicom \
    socat \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components
RUN rustup component add clippy rustfmt rust-src rust-analysis

# Install cross-compilation targets
RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Install useful Cargo tools
RUN cargo install cargo-edit cargo-audit cargo-bloat cargo-watch

# Set up cross-compilation environment
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# Create development user
RUN useradd -m -s /bin/bash developer && \
    usermod -aG sudo developer && \
    echo "developer ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

WORKDIR /workspace
RUN chown developer:developer /workspace

USER developer

# Set up shell environment
RUN echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc && \
    echo 'export RUST_BACKTRACE=1' >> ~/.bashrc && \
    echo 'alias build-arm64="cargo build --release --target aarch64-unknown-linux-gnu"' >> ~/.bashrc

CMD ["/bin/bash"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  dev:
    build: .
    container_name: your-project-dev
    volumes:
      - .:/workspace
      - cargo-cache:/home/developer/.cargo/registry
      - target-cache:/workspace/target
    working_dir: /workspace
    environment:
      - RUST_BACKTRACE=1
      - CARGO_TERM_COLOR=always
    stdin_open: true
    tty: true

  dev-serial:
    build: .
    container_name: your-project-dev-serial
    volumes:
      - .:/workspace
      - cargo-cache:/home/developer/.cargo/registry
    devices:
      - /dev/ttyUSB0:/dev/ttyUSB0
      - /dev/ttyACM0:/dev/ttyACM0
    privileged: true
    stdin_open: true
    tty: true

volumes:
  cargo-cache:
  target-cache:
```

## 🧪 Testing Templates

### tests/integration_tests.rs
```rust
use your_embedded_project::{Controller, Connection};
use std::env;

#[tokio::test]
#[ignore] // Requires hardware
async fn test_hardware_connection() {
    let device = env::var("TEST_DEVICE").unwrap_or_else(|_| "/dev/ttyUSB0".to_string());
    
    let connection = Connection::new(&device, 115200, false)
        .expect("Failed to create connection");
    
    let mut controller = Controller::new(connection);
    
    match controller.connect().await {
        Ok(_) => println!("✅ Hardware connection test passed"),
        Err(e) => println!("⚠️ Hardware not available: {}", e),
    }
}

#[tokio::test]
async fn test_mock_operations() {
    // Test with mock hardware - always passes in CI
    // Implement your mock tests here
    assert!(true);
}
```

### examples/basic_usage.rs
```rust
use your_embedded_project::{Controller, Connection};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("Your Embedded Project - Basic Usage Example");
    
    let mut connection = Connection::new("/dev/ttyUSB0", 115200, false)?;
    let mut controller = Controller::new(connection);
    
    // Connect to device
    controller.connect().await?;
    
    // Get status
    let status = controller.get_status().await?;
    println!("Device Status: {}", status);
    
    // Get device info
    let info = controller.get_info().await?;
    println!("Device Info: {}", info);
    
    Ok(())
}
```

## 📋 Development Checklist

### ✅ Initial Setup
- [ ] Clone this template and customize project name
- [ ] Update `Cargo.toml` with your project details
- [ ] Install cross-compilation toolchain: `sudo apt install gcc-aarch64-linux-gnu`
- [ ] Add Rust target: `rustup target add aarch64-unknown-linux-gnu`
- [ ] Test native build: `cargo build`
- [ ] Test cross-compilation: `./build-aarch64.sh`

### ✅ Hardware Integration
- [ ] Define your hardware communication protocol in `src/serial/protocol.rs`
- [ ] Implement actual serial communication in `src/serial/connection.rs`
- [ ] Add hardware-specific commands in `src/cli/mod.rs`
- [ ] Implement command handlers in `src/hardware/controller.rs`
- [ ] Add error handling for your specific hardware errors

### ✅ Testing & Validation
- [ ] Create integration tests for your hardware in `tests/`
- [ ] Add mock tests for CI/CD pipeline
- [ ] Test deployment to your target hardware
- [ ] Validate cross-compilation and binary size
- [ ] Test Docker development environment

### ✅ Production Readiness
- [ ] Configure CI/CD pipeline in `.github/workflows/`
- [ ] Add comprehensive documentation in `README.md`
- [ ] Set up proper logging and error handling
- [ ] Create deployment scripts for your target
- [ ] Add systemd service configuration (if needed)

## 🚀 Quick Start Commands

```bash
# 1. Create new project from template
git clone <template-repo> your-embedded-project
cd your-embedded-project

# 2. Customize the template
sed -i 's/your-embedded-project/actual-project-name/g' Cargo.toml
sed -i 's/Your Company Ltd/Actual Company/g' **/*.rs

# 3. Build and test
cargo build                    # Native build
./build-aarch64.sh            # Cross-compile for ARM64
cargo test                     # Run tests

# 4. Docker development
docker-compose up dev          # Start development container

# 5. Deploy to target (customize deploy script first)
./deploy-target.sh your-target-ip your-username
```

## 📚 Key Patterns from Source Project

### 🎯 **Command Pattern**
- Comprehensive CLI with subcommands using `clap` derive macros
- Structured command execution with proper error handling
- Multiple output formats (human, JSON, CSV)

### 🔌 **Hardware Communication**
- Async serial communication with `tokio-serial`
- Connection management with automatic reconnection
- Protocol abstraction for different hardware types

### 🏗️ **Project Structure**
- Modular architecture with clear separation of concerns
- Error handling with `thiserror` for structured errors
- Configuration management with TOML files

### 🔄 **Cross-Compilation**
- Automated ARM64 cross-compilation for embedded targets
- Docker-based development environment for consistency
- Deployment scripts for target hardware

### 📊 **Production Features**
- Comprehensive logging and debugging
- CI/CD integration with GitHub Actions
- Professional documentation and versioning

## 🔗 Additional Resources

### 📖 **Learning Resources**
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Clap Documentation](https://docs.rs/clap/)
- [Cross-compilation Guide](https://github.com/rust-lang/rustup/blob/master/doc/cross-compilation.md)

### 🛠️ **Tools & Crates**
- **Serial Communication**: `tokio-serial`, `serialport`
- **CLI Framework**: `clap`, `structopt` (legacy)
- **Error Handling**: `thiserror`, `anyhow`
- **Async Runtime**: `tokio`, `async-std`
- **Testing**: `mockall`, `assert_cmd`, `predicates`

### 🎯 **Similar Projects**
- [probe-rs](https://github.com/probe-rs/probe-rs) - Embedded debugging
- [cargo-embed](https://github.com/probe-rs/cargo-embed) - Embedded development
- [serialport-rs](https://github.com/serialport/serialport-rs) - Serial communication

---

## 📞 **Template Support**

**Based on**: E-ink Power CLI by Dynamic Devices Ltd  
**Template Author**: [Your Name]  
**License**: MIT/Apache-2.0 (choose appropriate)  
**Version**: 1.0.0  

This template captures the production-ready patterns from a real embedded Rust project that successfully manages power controllers, communicates over serial interfaces, and deploys to ARM64 embedded Linux systems.

**🎯 Ready to build your embedded Rust project with confidence!**
