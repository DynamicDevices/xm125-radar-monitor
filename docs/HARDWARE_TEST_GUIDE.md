# XM125 Radar Monitor - Hardware Testing Guide

**Target Audience**: Technicians and Test Engineers  
**Version**: v1.2.0  
**Date**: October 2025  
**Hardware**: Sentai i.MX8MM with XM125 Radar Module

## 🎯 Quick Start for Technicians

### Prerequisites
- Sentai board with XM125 radar module connected
- SSH access to Sentai board (fio@62.3.79.162:26)
- XM125 radar monitor v1.2.0 installed at `/usr/local/bin/xm125-radar-monitor`

### Basic Test Sequence
```bash
# 1. Check device status and current firmware
sudo /usr/local/bin/xm125-radar-monitor status
sudo /usr/local/bin/xm125-radar-monitor firmware check

# 2. Test presence detection (auto-update firmware if needed)
sudo /usr/local/bin/xm125-radar-monitor --mode presence --auto-update-firmware presence

# 3. Test distance measurement (auto-update firmware if needed)
sudo /usr/local/bin/xm125-radar-monitor --mode distance --auto-update-firmware measure

# 4. Test breathing detection (auto-update firmware if needed)
sudo /usr/local/bin/xm125-radar-monitor --mode breathing --auto-update-firmware breathing
```

## 📋 Detailed Test Procedures

### Test 1: System Status Check

**Purpose**: Verify hardware connectivity and basic system health

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor status
```

**Expected Output (Success)**:
```
xm125-radar-monitor v1.2.0
Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.
XM125 Radar Module Monitor
Mode: Distance | I2C: /dev/i2c-2 @ 0x52 | Auto-reconnect: ON

✅ XM125 Status: CONNECTED
Device ID: XM125, Firmware: Distance Detector (App ID: 1)
I2C Address: 0x52, Status: 0x800000FF
Temperature: 18°C, Calibration: OK
```

**Expected Output (Hardware Issue)**:
```
Error: I2C communication error: ENXIO: No such device or address
```

**Troubleshooting**:
- If "No such device or address": Check I2C connections, run `sudo i2cdetect -y 2`
- If permission denied: Ensure running with `sudo`
- If device at wrong address: Module may be in bootloader mode (0x48)

---

### Test 2: Presence Detection (Baseline Test)

**Purpose**: Verify presence detection functionality (known working mode)

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor --mode presence presence
```

**Expected Output (No Objects)**:
```
xm125-radar-monitor v1.1.0
Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.
XM125 Radar Module Monitor
Mode: Presence | I2C: /dev/i2c-2 @ 0x52 | Auto-reconnect: ON

👁️ Presence Detection:
Presence: NOT DETECTED, Distance: 0.00m, Intra: 0.00, Inter: 0.00
```

**Expected Output (Object Detected)**:
```
👁️ Presence Detection:
Presence: DETECTED, Distance: 1.25m, Intra: 2.34, Inter: 1.67
```

**Test Procedure**:
1. Run command with no objects in front of sensor (expect NOT DETECTED)
2. Place hand 1m in front of sensor (expect DETECTED with distance ~1.0m)
3. Move hand slowly (expect Inter score > 0)
4. Move hand quickly (expect Intra score > 0)

---

### Test 3: Distance Measurement

**Purpose**: Test precise distance measurement functionality

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor --mode distance measure
```

**Expected Output (No Objects)**:
```
xm125-radar-monitor v1.1.0
Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.
XM125 Radar Module Monitor
Mode: Distance | I2C: /dev/i2c-2 @ 0x52 | Auto-reconnect: ON

📏 Distance Measurement:
Distance: 0.00m, Strength: 0.0dB, Temperature: 18°C
```

**Expected Output (Object Detected)**:
```
📏 Distance Measurement:
Distance: 1.23m, Strength: 15.2dB, Temperature: 18°C
```

**Test Procedure**:
1. Run command with no objects (expect 0.00m)
2. Place flat object (book/clipboard) at 0.5m (expect ~0.50m ±0.05m)
3. Move object to 1.0m (expect ~1.00m ±0.05m)
4. Move object to 2.0m (expect ~2.00m ±0.10m)

**Acceptable Ranges**:
- 0.1m - 3.0m detection range
- ±5cm accuracy at <1m
- ±10cm accuracy at 1-3m
- Strength should be >10dB for good reflectors

---

### Test 4: Breathing Detection

**Purpose**: Test breathing pattern analysis functionality

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor --mode breathing breathing
```

**Expected Output (Initializing)**:
```
xm125-radar-monitor v1.1.0
Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.
XM125 Radar Module Monitor
Mode: Breathing | I2C: /dev/i2c-2 @ 0x52 | Auto-reconnect: ON

🫁 Breathing Detection:
State: Initializing, Rate: 1.7 BPM, Ready: YES, Temperature: 18°C
```

**Expected Output (No Presence)**:
```
🫁 Breathing Detection:
State: No Presence, Rate: 0.0 BPM, Ready: YES, Temperature: 18°C
```

**Expected Output (Breathing Detected)**:
```
🫁 Breathing Detection:
State: Estimating Breathing Rate, Rate: 16.5 BPM, Ready: YES, Temperature: 18°C
```

**Test Procedure**:
1. Run command with no person present (expect "No Presence")
2. Have person sit 1m from sensor, breathing normally
3. Wait 20-30 seconds for analysis (expect "Estimating Breathing Rate")
4. Verify breathing rate is reasonable (12-20 BPM for resting adult)

**Application States**:
- **Initializing**: System starting up
- **No Presence**: No person detected
- **Presence Detected**: Person detected, analyzing
- **Determining Distance**: Finding optimal measurement distance
- **Estimating Breathing Rate**: Active breathing analysis

---

### Test 5: Continuous Monitoring

**Purpose**: Test real-time monitoring capabilities

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor --mode presence monitor --count 10 --interval 500
```

**Expected Output**:
```
xm125-radar-monitor v1.1.0
Copyright (c) 2025 Dynamic Devices Ltd. All rights reserved.
XM125 Radar Module Monitor
Mode: Presence | I2C: /dev/i2c-2 @ 0x52 | Auto-reconnect: ON

[08:47:47] Presence: NOT DETECTED | Distance: 0.00m | Intra: 0.00 | Inter: 0.00
[08:47:48] Presence: NOT DETECTED | Distance: 0.00m | Intra: 0.00 | Inter: 0.00
[08:47:49] Presence: DETECTED | Distance: 1.25m | Intra: 2.34 | Inter: 1.67
[08:47:49] Presence: DETECTED | Distance: 1.23m | Intra: 2.41 | Inter: 1.72
...
```

**Test Procedure**:
1. Start monitoring with no objects
2. Introduce movement at different distances
3. Verify real-time updates every 500ms
4. Check that detection state changes appropriately

---

### Test 6: Data Export (CSV)

**Purpose**: Test data logging and export functionality

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor --mode presence monitor --count 5 --format csv
```

**Expected Output**:
```
timestamp,presence_detected,distance_m,intra_score,inter_score,measurement_count
2025-10-16T08:47:47.123Z,false,0.00,0.00,0.00,0
2025-10-16T08:47:48.124Z,false,0.00,0.00,0.00,1
2025-10-16T08:47:49.125Z,true,1.25,2.34,1.67,2
2025-10-16T08:47:50.126Z,true,1.23,2.41,1.72,3
2025-10-16T08:47:51.127Z,false,0.00,0.00,0.00,4
```

**Test Procedure**:
1. Run CSV export command
2. Verify CSV header is present
3. Verify data format is correct
4. Import into Excel/spreadsheet to validate

---

### Test 7: Firmware Management

**Purpose**: Test firmware detection and management

**Command**:
```bash
sudo /usr/local/bin/xm125-radar-monitor firmware check
```

**Expected Output**:
```
🔍 Firmware Check:
Current firmware: Distance Detector (App ID: 1)
Firmware file: /lib/firmware/acconeer/i2c_distance_detector.bin
Status: Compatible with distance mode
```

**Other Firmware Types**:
- **App ID: 1** = Distance Detector
- **App ID: 2** = Presence Detector  
- **App ID: 3** = Breathing Detector

---

## 🔄 Firmware Management & Validation

### Understanding Firmware Types

The XM125 module requires different firmware binaries for different detection modes:

| Detection Mode | Required Firmware | App ID | Binary File |
|----------------|-------------------|--------|-------------|
| **Distance** | Distance Detector | 1 | `i2c_distance_detector.bin` |
| **Presence** | Presence Detector | 2 | `i2c_presence_detector.bin` |
| **Breathing** | Breathing Monitor | 3 | `i2c_ref_app_breathing.bin` |
| **Combined** | Presence Detector | 2 | `i2c_presence_detector.bin` |

### Firmware Validation (New in v1.2.0)

**The application now automatically validates firmware compatibility:**

✅ **Correct Firmware**: Detection works normally  
❌ **Wrong Firmware**: Clear error message with instructions  
🔄 **Auto-Update**: Use `--auto-update-firmware` to automatically switch firmware

### Firmware Commands

```bash
# Check current firmware
sudo /usr/local/bin/xm125-radar-monitor firmware check

# Manual firmware update
sudo /usr/local/bin/xm125-radar-monitor firmware update --type presence

# Verify firmware integrity
sudo /usr/local/bin/xm125-radar-monitor firmware verify --type presence

# List all firmware checksums
sudo /usr/local/bin/xm125-radar-monitor firmware checksums
```

### Firmware Mismatch Examples

**❌ Wrong Firmware Error**:
```
Error: Invalid command parameters: Firmware mismatch: Current firmware is 
Distance Detector (App ID: 1), but presence mode requires Presence Detector 
(App ID: 2). Use --auto-update-firmware to automatically update firmware, 
or manually update with 'firmware update' command.
```

**✅ Auto-Update Solution**:
```bash
# This will automatically update firmware and run the test
sudo /usr/local/bin/xm125-radar-monitor --mode presence --auto-update-firmware presence
```

**✅ Manual Update Solution**:
```bash
# Update firmware first, then run test
sudo /usr/local/bin/xm125-radar-monitor firmware update --type presence
sudo /usr/local/bin/xm125-radar-monitor --mode presence presence
```

---

## 🔧 Troubleshooting Guide

### Common Issues and Solutions

| Issue | Symptoms | Solution |
|-------|----------|----------|
| **Firmware Mismatch** | `Firmware mismatch: Current firmware is...` | Use `--auto-update-firmware` flag or `firmware update --type <mode>` |
| **Device Not Found** | `ENXIO: No such device or address` | Check I2C connections, run `sudo i2cdetect -y 2` |
| **Permission Denied** | `Permission denied` error | Always run with `sudo` |
| **Wrong Firmware** | Mode doesn't work as expected | Use `firmware check` and update if needed |
| **Calibration Failed** | `Calibration timeout` error | Check hardware connections, try reset |
| **No Detection** | Always shows 0.00m or NOT DETECTED | Check sensor orientation, remove obstructions |
| **Inconsistent Readings** | Erratic distance/presence values | Check for vibrations, electromagnetic interference |
| **Firmware Update Failed** | `Firmware update failed` | Check GPIO connections, verify binaries exist, try manual reset |

### Debug Commands

**Verbose Logging**:
```bash
sudo /usr/local/bin/xm125-radar-monitor --verbose --mode presence presence
```

**I2C Bus Scan**:
```bash
sudo i2cdetect -y 2
```
Expected: Device at 0x52 (run mode) or 0x48 (bootloader mode)

**GPIO Status Check**:
```bash
sudo /home/fio/xm125-control.sh --status
```

**Hardware Reset**:
```bash
sudo /home/fio/xm125-control.sh --reset-run
```

---

## 📊 Test Results Template

### Test Session Information
- **Date**: ___________
- **Technician**: ___________
- **Hardware**: Sentai Board S/N: ___________
- **Software Version**: ___________

### Test Results Checklist

- [ ] **System Status**: Device detected and responsive
- [ ] **Presence Detection**: Baseline functionality confirmed
- [ ] **Distance Measurement**: Accurate readings within spec
- [ ] **Breathing Detection**: State machine functioning
- [ ] **Continuous Monitoring**: Real-time updates working
- [ ] **Data Export**: CSV format correct
- [ ] **Firmware Management**: Current firmware identified

### Performance Measurements

| Test | Expected | Actual | Pass/Fail | Notes |
|------|----------|--------|-----------|-------|
| Distance @ 0.5m | 0.50m ±0.05m | _____ | _____ | _____ |
| Distance @ 1.0m | 1.00m ±0.05m | _____ | _____ | _____ |
| Distance @ 2.0m | 2.00m ±0.10m | _____ | _____ | _____ |
| Presence Range | 0.5m - 7.0m | _____ | _____ | _____ |
| Breathing Rate | 12-20 BPM | _____ | _____ | _____ |
| Temperature | Ambient ±5°C | _____ | _____ | _____ |

### Issues Found
- Issue 1: ________________________________
- Issue 2: ________________________________
- Issue 3: ________________________________

### Overall Assessment
- [ ] **PASS**: All tests completed successfully
- [ ] **FAIL**: Critical issues found (specify above)
- [ ] **PARTIAL**: Some functionality working (specify above)

---

## 📞 Support Information

**Technical Support**: info@dynamicdevices.co.uk  
**Documentation**: https://github.com/DynamicDevices/xm125-radar-monitor  
**Issue Reporting**: GitHub Issues or email support

**Emergency Contact**: Alex J Lennon (ajlennon@dynamicdevices.co.uk)
