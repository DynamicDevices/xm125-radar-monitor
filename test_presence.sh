#!/bin/bash
# Simple XM125 Presence Detection Test
# Based on presence_reg_protocol.h specification

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
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

# XM125 device configuration
I2C_BUS=2
I2C_ADDR=0x52
DEVICE_PATH="/dev/i2c-${I2C_BUS}"

print_status "🎯 XM125 Presence Detection Test"
echo "Device: ${DEVICE_PATH} @ ${I2C_ADDR}"
echo

print_status "=== Step 1: Check if XM125 is responding ==="
if ! i2cdetect -y ${I2C_BUS} | grep -q 52; then
    print_error "XM125 not found on I2C bus ${I2C_BUS}"
    exit 1
fi
print_success "XM125 detected on I2C bus ${I2C_BUS}"

print_status "=== Step 2: Read Application ID (should be 2 for presence) ==="
# Register 65535 (0xFFFF) = Application ID
APP_ID=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0xFF 0xFF i 2>/dev/null || echo "0x00000000")
echo "Application ID: ${APP_ID}"
if [[ "${APP_ID}" == *"0x02"* ]]; then
    print_success "Presence detector application detected"
elif [[ "${APP_ID}" == *"0x01"* ]]; then
    print_error "Distance detector application - need presence detector firmware"
    exit 1
else
    print_error "Unknown application ID: ${APP_ID}"
fi

print_status "=== Step 3: Read Detector Status ==="
# Register 3 = Detector Status
STATUS=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0x03 i 2>/dev/null || echo "0x00000000")
echo "Detector Status: ${STATUS}"

# Decode status bits (from presence_reg_protocol.h)
STATUS_NUM=$(printf "%d" ${STATUS})
RSS_OK=$((STATUS_NUM & 0x01))
CONFIG_OK=$((STATUS_NUM & 0x02))
SENSOR_OK=$((STATUS_NUM & 0x04))
SENSOR_CAL_OK=$((STATUS_NUM & 0x08))
DETECTOR_OK=$((STATUS_NUM & 0x10))

echo "Status bits:"
echo "  RSS Register OK: $([[ $RSS_OK -ne 0 ]] && echo "✅" || echo "❌")"
echo "  Config Create OK: $([[ $CONFIG_OK -ne 0 ]] && echo "✅" || echo "❌")"
echo "  Sensor Create OK: $([[ $SENSOR_OK -ne 0 ]] && echo "✅" || echo "❌")"
echo "  Sensor Calibrate OK: $([[ $SENSOR_CAL_OK -ne 0 ]] && echo "✅" || echo "❌")"
echo "  Detector Create OK: $([[ $DETECTOR_OK -ne 0 ]] && echo "✅" || echo "❌")"

print_status "=== Step 4: Try APPLY_CONFIGURATION Command ==="
# Register 256 (0x0100) = Command Register
# Command 1 = APPLY_CONFIGURATION
echo "Sending APPLY_CONFIGURATION command (1) to register 256..."
if i2cset -y ${I2C_BUS} ${I2C_ADDR} 0x01 0x00 0x01 0x00 0x00 0x00 i 2>/dev/null; then
    print_success "Command sent successfully"
    
    # Wait and check status
    sleep 2
    NEW_STATUS=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0x03 i 2>/dev/null || echo "0x00000000")
    echo "New Status: ${NEW_STATUS}"
    
    if [[ "${STATUS}" != "${NEW_STATUS}" ]]; then
        print_success "Status changed - device responded to command!"
    else
        print_error "Status unchanged - device may not be processing commands"
    fi
else
    print_error "Failed to send command"
fi

print_status "=== Step 5: Try Reading Presence Result ==="
# Register 16 = Presence Result
PRESENCE_RESULT=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0x10 i 2>/dev/null || echo "0x00000000")
echo "Presence Result: ${PRESENCE_RESULT}"

# Decode presence result bits
RESULT_NUM=$(printf "%d" ${PRESENCE_RESULT})
PRESENCE_DETECTED=$((RESULT_NUM & 0x01))
PRESENCE_STICKY=$((RESULT_NUM & 0x02))
DETECTOR_ERROR=$((RESULT_NUM & 0x8000))

echo "Presence flags:"
echo "  Presence Detected: $([[ $PRESENCE_DETECTED -ne 0 ]] && echo "🎯 YES" || echo "❌ NO")"
echo "  Presence Sticky: $([[ $PRESENCE_STICKY -ne 0 ]] && echo "🔒 YES" || echo "❌ NO")"
echo "  Detector Error: $([[ $DETECTOR_ERROR -ne 0 ]] && echo "❌ ERROR" || echo "✅ OK")"

print_status "=== Step 6: Read Presence Distance and Scores ==="
# Register 17 = Presence Distance
DISTANCE=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0x11 i 2>/dev/null || echo "0x00000000")
echo "Presence Distance: ${DISTANCE}"

# Register 18 = Intra Presence Score (fast motion)
INTRA_SCORE=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0x12 i 2>/dev/null || echo "0x00000000")
echo "Intra Presence Score (fast): ${INTRA_SCORE}"

# Register 19 = Inter Presence Score (slow motion)  
INTER_SCORE=$(i2cget -y ${I2C_BUS} ${I2C_ADDR} 0x13 i 2>/dev/null || echo "0x00000000")
echo "Inter Presence Score (slow): ${INTER_SCORE}"

echo
print_status "=== XM125 Presence Test Complete ==="
echo "This test verifies:"
echo "✓ I2C communication with XM125"
echo "✓ Application ID detection"  
echo "✓ Detector status decoding"
echo "✓ Command sending capability"
echo "✓ Presence result reading"
echo
echo "Next steps for full presence detection:"
echo "1. Implement proper initialization sequence"
echo "2. Configure presence parameters (thresholds, range, etc.)"
echo "3. Start continuous detection mode"
echo "4. Poll presence results periodically"
