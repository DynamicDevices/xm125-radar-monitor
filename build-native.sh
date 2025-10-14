#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_status "Building XM125 Radar Monitor for native target"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Run pre-build checks
print_status "Running pre-build checks..."

# Check code formatting
print_status "Checking code formatting..."
if ! cargo fmt --all -- --check; then
    print_error "Code formatting check failed!"
    print_warning "Run 'cargo fmt --all' to fix formatting issues"
    exit 1
fi
print_success "Code formatting is correct"

# Run Clippy lints
print_status "Running Clippy lints..."
if ! cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic -A dead_code -A clippy::module_name_repetitions -A clippy::similar_names; then
    print_error "Clippy lints failed!"
    print_warning "Fix the linting issues above before building"
    exit 1
fi
print_success "Clippy lints passed"

# Run tests
print_status "Running tests..."
if ! cargo test; then
    print_error "Tests failed!"
    exit 1
fi
print_success "All tests passed"

# Build for native target
print_status "Building release binary..."
if ! cargo build --release; then
    print_error "Build failed!"
    exit 1
fi

# Show binary information
BINARY_PATH="target/release/xm125-radar-monitor"
if [ -f "$BINARY_PATH" ]; then
    print_success "Build completed successfully!"
    echo "Binary: $BINARY_PATH"
    echo "Size: $(ls -lh $BINARY_PATH | awk '{print $5}')"
    echo "Type: $(file $BINARY_PATH)"
    
    # Show target architecture
    TARGET_ARCH=$(rustc --version --verbose | grep "host:" | cut -d' ' -f2)
    echo "Target: $TARGET_ARCH"
    
    # Optional: Strip binary for smaller size (if strip is available)
    if command -v strip &> /dev/null; then
        ORIGINAL_SIZE=$(stat -c%s "$BINARY_PATH")
        strip "$BINARY_PATH"
        STRIPPED_SIZE=$(stat -c%s "$BINARY_PATH")
        SAVED_BYTES=$((ORIGINAL_SIZE - STRIPPED_SIZE))
        print_success "Binary stripped (saved $SAVED_BYTES bytes)"
        echo "Final size: $(ls -lh $BINARY_PATH | awk '{print $5}')"
    fi
    
else
    print_error "Build failed - binary not found"
    exit 1
fi

print_success "Native build completed successfully!"
