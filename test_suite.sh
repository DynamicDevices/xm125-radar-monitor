#!/bin/bash
# XM125 Hardware Test Suite - Comprehensive validation for production deployment

set -euo pipefail

# Configuration
DEVICE="/usr/local/bin/xm125-radar-monitor"
LOG_DIR="test_results_$(date +%Y%m%d_%H%M%S)"
TEST_COUNT=50
TIMEOUT=30

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[$(date +'%H:%M:%S')] $1${NC}"; }
warn() { echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARNING: $1${NC}"; }
error() { echo -e "${RED}[$(date +'%H:%M:%S')] ERROR: $1${NC}"; }

# Test functions
test_device_communication() {
    log "Testing device communication..."
    timeout $TIMEOUT sudo $DEVICE status > "$LOG_DIR/communication.log" 2>&1
    timeout $TIMEOUT sudo $DEVICE info >> "$LOG_DIR/communication.log" 2>&1
}

test_firmware_management() {
    log "Testing firmware management..."
    for fw in distance presence breathing; do
        log "  Testing $fw firmware..."
        timeout $TIMEOUT sudo $DEVICE firmware update $fw --force > "$LOG_DIR/firmware_$fw.log" 2>&1
        timeout $TIMEOUT sudo $DEVICE firmware check >> "$LOG_DIR/firmware_$fw.log" 2>&1
    done
}

test_detection_modes() {
    log "Testing detection modes..."
    
    # Distance detection
    log "  Testing distance detection..."
    timeout $TIMEOUT sudo $DEVICE --mode distance measure > "$LOG_DIR/distance_test.log" 2>&1
    timeout $TIMEOUT sudo $DEVICE --mode distance monitor --count 10 >> "$LOG_DIR/distance_test.log" 2>&1
    
    # Presence detection  
    log "  Testing presence detection..."
    timeout $TIMEOUT sudo $DEVICE --mode presence presence > "$LOG_DIR/presence_test.log" 2>&1
    timeout $TIMEOUT sudo $DEVICE --mode presence monitor --count 10 >> "$LOG_DIR/presence_test.log" 2>&1
}

test_continuous_operation() {
    log "Testing continuous operation stability..."
    timeout $TIMEOUT sudo $DEVICE --mode presence monitor --count $TEST_COUNT --save-to "$LOG_DIR/stability_test.csv" > "$LOG_DIR/stability.log" 2>&1
}

test_error_recovery() {
    log "Testing error recovery..."
    # Test auto-reconnect by forcing a reset
    sudo /home/fio/xm125-control.sh --reset-run > "$LOG_DIR/recovery.log" 2>&1
    sleep 2
    timeout $TIMEOUT sudo $DEVICE --auto-reconnect status >> "$LOG_DIR/recovery.log" 2>&1
}

# Main test execution
main() {
    log "Starting XM125 Hardware Test Suite"
    log "Results will be saved to: $LOG_DIR"
    
    mkdir -p "$LOG_DIR"
    
    # Test suite execution
    local tests=(
        "test_device_communication"
        "test_firmware_management" 
        "test_detection_modes"
        "test_continuous_operation"
        "test_error_recovery"
    )
    
    local passed=0
    local failed=0
    
    for test in "${tests[@]}"; do
        if $test; then
            log "✅ $test PASSED"
            ((passed++))
        else
            error "❌ $test FAILED"
            ((failed++))
        fi
    done
    
    # Summary
    log "Test Summary: $passed passed, $failed failed"
    echo "Detailed logs available in: $LOG_DIR"
    
    if [ $failed -eq 0 ]; then
        log "🎉 All tests PASSED - Hardware validation successful"
        exit 0
    else
        error "⚠️  Some tests FAILED - Review logs for details"
        exit 1
    fi
}

# Cleanup on exit
trap 'log "Test suite interrupted"' INT TERM

# Check prerequisites
if [ ! -f "$DEVICE" ]; then
    error "XM125 radar monitor not found at $DEVICE"
    exit 1
fi

if [ "$EUID" -eq 0 ]; then
    warn "Running as root - tests will use sudo anyway"
fi

main "$@"