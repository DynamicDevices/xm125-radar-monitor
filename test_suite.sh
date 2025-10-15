#!/bin/bash
# XM125 Automated Hardware Test Suite
# Author: Dynamic Devices Ltd
# Version: 1.0

set -e

# Configuration
DEVICE_PATH="/dev/i2c-2"
DEVICE_ADDR="0x52"
TEST_DIR="xm125_test_results_$(date +%Y%m%d_%H%M%S)"
MONITOR_CMD="./xm125-radar-monitor --mode presence"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

record_result() {
    local test_name="$1"
    local result="$2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" = "PASS" ]; then
        log_success "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        log_error "$test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    echo "$test_name,$result,$(date)" >> "$TEST_DIR/test_summary.csv"
}

# Setup test environment
setup_test_environment() {
    log_info "Setting up test environment..."
    
    # Create test results directory
    mkdir -p "$TEST_DIR"
    
    # Initialize results file
    echo "Test Name,Result,Timestamp" > "$TEST_DIR/test_summary.csv"
    
    # Check device connectivity
    if ! sudo i2cdetect -y 2 | grep -q 52; then
        log_error "XM125 not detected on I2C bus. Check connections."
        exit 1
    fi
    
    # Check tool availability
    if [ ! -f "./xm125-radar-monitor" ]; then
        log_error "xm125-radar-monitor tool not found in current directory"
        exit 1
    fi
    
    log_success "Test environment ready"
}

# Test 1: Device Status and Info
test_device_status() {
    log_info "Running Device Status Test..."
    
    # Test device status
    if sudo $MONITOR_CMD status > "$TEST_DIR/device_status.log" 2>&1; then
        record_result "Device Status Check" "PASS"
    else
        record_result "Device Status Check" "FAIL"
        return 1
    fi
    
    # Test device info
    if sudo $MONITOR_CMD info > "$TEST_DIR/device_info.log" 2>&1; then
        # Check if it's presence detector firmware
        if grep -q "Presence Detector" "$TEST_DIR/device_info.log"; then
            record_result "Presence Detector Firmware" "PASS"
        else
            record_result "Presence Detector Firmware" "FAIL"
        fi
    else
        record_result "Device Info Check" "FAIL"
    fi
}

# Test 2: Basic Detection Test
test_basic_detection() {
    log_info "Running Basic Detection Test..."
    log_warning "Please wave your hand in front of the sensor when prompted"
    
    echo "Press Enter when ready to start detection test..."
    read
    
    # Take 10 samples
    sudo $MONITOR_CMD monitor --count 10 --interval 500 \
        --save-to "$TEST_DIR/basic_detection.csv" > "$TEST_DIR/basic_detection.log" 2>&1
    
    # Analyze results
    local detections=$(grep -c "DETECTED" "$TEST_DIR/basic_detection.log" || echo 0)
    local total=10
    local success_rate=$((detections * 100 / total))
    
    echo "Detection Success Rate: $success_rate% ($detections/$total)" >> "$TEST_DIR/basic_detection_analysis.txt"
    
    if [ $success_rate -ge 70 ]; then
        record_result "Basic Detection Test ($success_rate%)" "PASS"
    else
        record_result "Basic Detection Test ($success_rate%)" "FAIL"
    fi
}

# Test 3: Range Testing
test_detection_range() {
    log_info "Running Detection Range Test..."
    
    local distances=(1.0 2.0 3.0 4.0 5.0)
    local range_results="$TEST_DIR/range_test_results.txt"
    
    echo "Distance,Detections,Total,Success_Rate" > "$TEST_DIR/range_test_summary.csv"
    
    for distance in "${distances[@]}"; do
        log_info "Testing at ${distance}m distance..."
        echo "Position target at ${distance}m and press Enter..."
        read
        
        # Take measurements
        sudo $MONITOR_CMD monitor --count 8 --interval 400 \
            --save-to "$TEST_DIR/range_${distance}m.csv" > "$TEST_DIR/range_${distance}m.log" 2>&1
        
        # Analyze results
        local detections=$(grep -c "DETECTED" "$TEST_DIR/range_${distance}m.log" || echo 0)
        local total=8
        local success_rate=$((detections * 100 / total))
        
        echo "${distance}m: $success_rate% success ($detections/$total)" >> "$range_results"
        echo "$distance,$detections,$total,$success_rate" >> "$TEST_DIR/range_test_summary.csv"
        
        # Check if meets criteria (>80% success for distances up to 4m)
        if [ "${distance%.*}" -le 4 ] && [ $success_rate -ge 80 ]; then
            record_result "Range Test ${distance}m ($success_rate%)" "PASS"
        elif [ "${distance%.*}" -gt 4 ] && [ $success_rate -ge 60 ]; then
            record_result "Range Test ${distance}m ($success_rate%)" "PASS"
        else
            record_result "Range Test ${distance}m ($success_rate%)" "FAIL"
        fi
    done
}

# Test 4: False Positive Test
test_false_positives() {
    log_info "Running False Positive Test..."
    log_warning "Ensure NO movement in front of sensor during this test"
    
    echo "Clear the sensor area and press Enter to start false positive test..."
    read
    
    # Take baseline measurements with no targets
    sudo $MONITOR_CMD monitor --count 20 --interval 300 \
        --save-to "$TEST_DIR/false_positive_test.csv" > "$TEST_DIR/false_positive_test.log" 2>&1
    
    # Analyze false positives
    local false_positives=$(grep -c "DETECTED" "$TEST_DIR/false_positive_test.log" || echo 0)
    local total=20
    local false_positive_rate=$((false_positives * 100 / total))
    
    echo "False Positive Rate: $false_positive_rate% ($false_positives/$total)" >> "$TEST_DIR/false_positive_analysis.txt"
    
    if [ $false_positive_rate -le 10 ]; then
        record_result "False Positive Test ($false_positive_rate%)" "PASS"
    else
        record_result "False Positive Test ($false_positive_rate%)" "FAIL"
    fi
}

# Test 5: Motion Sensitivity Test
test_motion_sensitivity() {
    log_info "Running Motion Sensitivity Test..."
    
    local motion_types=("slow" "normal" "fast")
    
    for motion in "${motion_types[@]}"; do
        log_info "Testing ${motion} motion detection..."
        case $motion in
            "slow") echo "Perform SLOW hand movement (very gentle wave)..." ;;
            "normal") echo "Perform NORMAL hand movement (regular wave)..." ;;
            "fast") echo "Perform FAST hand movement (quick wave)..." ;;
        esac
        read
        
        # Take measurements
        sudo $MONITOR_CMD monitor --count 6 --interval 400 \
            --save-to "$TEST_DIR/motion_${motion}.csv" > "$TEST_DIR/motion_${motion}.log" 2>&1
        
        # Analyze results
        local detections=$(grep -c "DETECTED" "$TEST_DIR/motion_${motion}.log" || echo 0)
        local total=6
        local success_rate=$((detections * 100 / total))
        
        if [ $success_rate -ge 70 ]; then
            record_result "Motion Test ${motion} ($success_rate%)" "PASS"
        else
            record_result "Motion Test ${motion} ($success_rate%)" "FAIL"
        fi
    done
}

# Test 6: Stability Test (Short Duration)
test_stability() {
    log_info "Running Stability Test (2 minutes)..."
    log_warning "Perform consistent motion every 10 seconds during this test"
    
    # Run 2-minute stability test
    sudo $MONITOR_CMD monitor --count 24 --interval 5000 \
        --save-to "$TEST_DIR/stability_test.csv" > "$TEST_DIR/stability_test.log" 2>&1 &
    
    local monitor_pid=$!
    
    # Wait for test to complete
    wait $monitor_pid
    
    # Analyze stability
    local detections=$(grep -c "DETECTED" "$TEST_DIR/stability_test.log" || echo 0)
    local total=24
    local success_rate=$((detections * 100 / total))
    
    echo "Stability Test Success Rate: $success_rate% ($detections/$total)" >> "$TEST_DIR/stability_analysis.txt"
    
    if [ $success_rate -ge 80 ]; then
        record_result "Stability Test ($success_rate%)" "PASS"
    else
        record_result "Stability Test ($success_rate%)" "FAIL"
    fi
}

# Generate test report
generate_report() {
    log_info "Generating test report..."
    
    local report_file="$TEST_DIR/XM125_TEST_REPORT.txt"
    
    cat > "$report_file" << EOF
XM125 PRESENCE DETECTOR TEST REPORT
==================================

Test Date: $(date)
Test Directory: $TEST_DIR
Device: XM125 @ $DEVICE_ADDR on $DEVICE_PATH

SUMMARY:
--------
Total Tests: $TOTAL_TESTS
Passed: $TESTS_PASSED
Failed: $TESTS_FAILED
Success Rate: $((TESTS_PASSED * 100 / TOTAL_TESTS))%

OVERALL RESULT: $([ $TESTS_FAILED -eq 0 ] && echo "PASS" || echo "FAIL")

DETAILED RESULTS:
----------------
EOF
    
    # Append detailed results
    cat "$TEST_DIR/test_summary.csv" >> "$report_file"
    
    echo "" >> "$report_file"
    echo "FILES GENERATED:" >> "$report_file"
    echo "---------------" >> "$report_file"
    ls -la "$TEST_DIR" >> "$report_file"
    
    log_success "Test report generated: $report_file"
}

# Interactive test menu
interactive_menu() {
    while true; do
        echo ""
        echo "XM125 Hardware Test Suite"
        echo "========================"
        echo "1. Run All Tests (Automated)"
        echo "2. Device Status & Info"
        echo "3. Basic Detection Test"
        echo "4. Range Testing"
        echo "5. False Positive Test"
        echo "6. Motion Sensitivity Test"
        echo "7. Stability Test"
        echo "8. Generate Report"
        echo "9. Exit"
        echo ""
        read -p "Select test (1-9): " choice
        
        case $choice in
            1) run_all_tests ;;
            2) test_device_status ;;
            3) test_basic_detection ;;
            4) test_detection_range ;;
            5) test_false_positives ;;
            6) test_motion_sensitivity ;;
            7) test_stability ;;
            8) generate_report ;;
            9) exit 0 ;;
            *) log_error "Invalid choice. Please select 1-9." ;;
        esac
    done
}

# Run all tests in sequence
run_all_tests() {
    log_info "Starting comprehensive XM125 hardware test suite..."
    
    test_device_status
    test_basic_detection
    test_detection_range
    test_false_positives
    test_motion_sensitivity
    test_stability
    
    generate_report
    
    echo ""
    if [ $TESTS_FAILED -eq 0 ]; then
        log_success "ALL TESTS PASSED! ($TESTS_PASSED/$TOTAL_TESTS)"
    else
        log_error "SOME TESTS FAILED! ($TESTS_FAILED/$TOTAL_TESTS failed)"
    fi
}

# Main execution
main() {
    echo "XM125 Radar Module Hardware Test Suite v1.0"
    echo "============================================"
    
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        log_error "This script requires root privileges for I2C access"
        echo "Please run with: sudo $0"
        exit 1
    fi
    
    setup_test_environment
    
    # Check command line arguments
    if [ "$1" = "--auto" ]; then
        run_all_tests
    else
        interactive_menu
    fi
}

# Run main function
main "$@"
