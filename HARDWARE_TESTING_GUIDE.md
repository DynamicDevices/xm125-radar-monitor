# 🧪 XM125 Radar Hardware Testing Guide

**Version**: 2.0.0  
**Date**: October 22, 2025  
**Target**: Hardware Testing Team  

## 📋 Overview

This guide provides comprehensive testing procedures for evaluating XM125 radar performance in three critical areas:
1. **Range Detection** - Maximum detection distance capability
2. **Angle Coverage** - Left/right detection field of view  
3. **False Positives** - Unwanted detections in empty scenarios

## 🎯 Testing Objectives

### **Performance Comparison Goals**
- Compare against Windows evaluation tool performance
- Identify performance gaps and optimization opportunities
- Validate detection reliability across different scenarios
- Generate detailed data for analysis and reporting

## 🔧 Enhanced Tool Features (v2.0.0)

### **Improved Console Output**
```
[19:45:26.383] #  1 Presence: NOT DETECTED | Distance: 0.00m | Fast: 0.00(  NONE) | Slow: 0.00(  NONE)
[19:45:26.858] #  2 Presence:    DETECTED | Distance: 1.84m | Fast: 1.26(MEDIUM) | Slow: 3.00(STRONG)
```

**Signal Strength Indicators:**
- `STRONG` (>2.0) - Excellent detection confidence
- `MEDIUM` (1.0-2.0) - Good detection confidence  
- `WEAK` (0.5-1.0) - Marginal detection
- `NONE` (<0.5) - No significant signal

### **Enhanced CSV Output**
```csv
timestamp,presence_detected,presence_distance_m,intra_score,inter_score,intra_strength,inter_strength,detection_confidence,measurement_number
2025-10-22 19:45:26.383,false,0,0,0,NONE,NONE,NONE,1
2025-10-22 19:45:26.858,true,1.84,1.26,3.00,MEDIUM,STRONG,HIGH,2
```

## 🧪 Test Procedures

---

## **TEST 1: Range Detection Testing**

### **Objective**: Determine maximum reliable detection distance

### **Equipment Needed**:
- Measuring tape (minimum 8 meters)
- Test subject (person)
- Stopwatch
- Notebook for manual observations

### **Test Commands**:

#### **1.1 Long Range Preset Test**
```bash
sudo xm125-radar-monitor presence --range long --continuous --save-to range_test_long.csv
```
- **Range**: 0.5m - 7.0m
- **Use Case**: Room occupancy detection

#### **1.2 Extended Range Test**
```bash
sudo xm125-radar-monitor presence --min-range 1.0 --max-range 10.0 --continuous --save-to range_test_extended.csv
```
- **Range**: 1.0m - 10.0m (testing beyond spec)
- **Use Case**: Maximum capability assessment

#### **1.3 Custom Range Test**
```bash
sudo xm125-radar-monitor presence --min-range 2.0 --max-range 8.0 --continuous --save-to range_test_custom.csv
```
- **Range**: 2.0m - 8.0m
- **Use Case**: Optimized mid-range detection

### **Test Procedure**:

1. **Setup**: Position radar at chest height (1.2m), clear line of sight
2. **Baseline**: Start with person at 0.5m from sensor
3. **Distance Testing**:
   - Move away in 0.5m increments (0.5m → 1.0m → 1.5m → ... → 8.0m)
   - At each position, stand still for 30 seconds
   - Record detection reliability (% of time detected)
   - Note signal strength indicators (STRONG/MEDIUM/WEAK)
4. **Movement Testing**:
   - Test slow walk (0.5 m/s)
   - Test normal walk (1.0 m/s)  
   - Test fast walk (1.5 m/s)
5. **Record Results**:
   - Maximum reliable detection distance
   - Distance where signal drops to WEAK
   - Distance where detection becomes intermittent

### **Expected Results**:
- **Reliable Detection**: Up to 6-7m with MEDIUM+ signal
- **Maximum Detection**: Up to 7-8m with WEAK signal
- **Performance Gap**: Compare with Windows tool maximum range

---

## **TEST 2: Angle Coverage Testing**

### **Objective**: Map left/right detection field of view

### **Equipment Needed**:
- Protractor or angle measuring device
- Measuring tape
- Masking tape for position marking
- Test subject

### **Test Commands**:

#### **2.1 Medium Range Angle Test**
```bash
sudo xm125-radar-monitor presence --range medium --continuous --count 500 --interval 200 --save-to angle_test_medium.csv
```
- **Range**: 0.2m - 2.0m
- **Measurement**: Every 200ms for detailed tracking

#### **2.2 Long Range Angle Test**
```bash
sudo xm125-radar-monitor presence --range long --continuous --count 300 --interval 300 --save-to angle_test_long.csv
```
- **Range**: 0.5m - 7.0m
- **Measurement**: Every 300ms

### **Test Procedure**:

1. **Setup**: 
   - Position radar at 1.2m height
   - Mark center line directly in front of sensor
   - Mark 1.5m distance from sensor (center of medium range)
2. **Angle Mapping**:
   - Start at center position (0°)
   - Move left in 15° increments: 0° → 15° → 30° → 45° → 60° → 75° → 90°
   - At each angle, stand still for 60 seconds
   - Record detection percentage and signal strength
   - Return to center, repeat for right side
3. **Distance Variation**:
   - Repeat angle test at 1.0m distance
   - Repeat angle test at 2.0m distance
4. **Movement Testing**:
   - Walk across detection field from left to right
   - Record where detection starts/stops

### **Expected Results**:
- **Full Detection**: ±30° with STRONG signal
- **Partial Detection**: ±45° with MEDIUM signal  
- **Edge Detection**: ±60° with WEAK signal
- **Performance Gap**: Compare field of view with Windows tool

---

## **TEST 3: False Positive Testing**

### **Objective**: Identify unwanted detections in empty scenarios

### **Equipment Needed**:
- Empty room
- Various environmental factors (fan, AC, lights)
- Timer for long-duration tests

### **Test Commands**:

#### **3.1 Empty Room Baseline**
```bash
sudo xm125-radar-monitor presence --range long --continuous --count 1800 --interval 1000 --save-to false_positive_baseline.csv
```
- **Duration**: 30 minutes (1800 measurements at 1-second intervals)
- **Environment**: Completely empty room

#### **3.2 Environmental Interference Test**
```bash
sudo xm125-radar-monitor presence --range medium --continuous --count 600 --interval 500 --save-to false_positive_interference.csv
```
- **Duration**: 5 minutes with various interference sources

#### **3.3 Verbose False Positive Analysis**
```bash
sudo xm125-radar-monitor --verbose presence --range long --continuous --count 300 --save-to false_positive_verbose.csv
```
- **Duration**: Detailed logging for analysis

### **Test Procedure**:

1. **Baseline Empty Room**:
   - Ensure room is completely empty
   - No moving objects (curtains, plants, etc.)
   - Run 30-minute test
   - Record any false detections
2. **Environmental Factors**:
   - Test with ceiling fan running
   - Test with air conditioning
   - Test with fluorescent lights
   - Test with other electronics nearby
   - Test with door/window vibrations
3. **Analysis**:
   - Count total false positives
   - Identify patterns (time-based, signal strength)
   - Note environmental correlations

### **Expected Results**:
- **Baseline**: <1% false positive rate in empty room
- **With Interference**: <5% false positive rate
- **Performance Gap**: Compare false positive rate with Windows tool

---

## 📊 Data Analysis

### **CSV Data Fields**:
- `timestamp` - Precise measurement time
- `presence_detected` - Boolean detection result
- `presence_distance_m` - Detected distance in meters
- `intra_score` - Fast motion detection score
- `inter_score` - Slow motion detection score  
- `intra_strength` - Fast motion signal strength (STRONG/MEDIUM/WEAK/NONE)
- `inter_strength` - Slow motion signal strength (STRONG/MEDIUM/WEAK/NONE)
- `detection_confidence` - Overall confidence (HIGH/MEDIUM/LOW/NONE)
- `measurement_number` - Sequential measurement count

### **Key Metrics to Calculate**:

#### **Range Performance**:
```bash
# Detection rate by distance
awk -F',' 'NR>1 && $2=="true" {print $3}' range_test_long.csv | sort -n | uniq -c

# Signal strength distribution  
awk -F',' 'NR>1 {print $6, $7}' range_test_long.csv | sort | uniq -c
```

#### **Angle Performance**:
```bash
# Detection percentage during angle sweeps
awk -F',' 'NR>1 {detected+=$2=="true"; total++} END {print "Detection Rate:", detected/total*100"%"}' angle_test_medium.csv
```

#### **False Positive Rate**:
```bash
# False positive percentage
awk -F',' 'NR>1 {false_pos+=$2=="true"; total++} END {print "False Positive Rate:", false_pos/total*100"%"}' false_positive_baseline.csv
```

---

## 🚀 Quick Start Commands

### **Basic Range Test** (5 minutes):
```bash
sudo xm125-radar-monitor presence --range long --continuous --count 100 --interval 3000 --save-to quick_range_test.csv
```

### **Basic Angle Test** (3 minutes):
```bash
sudo xm125-radar-monitor presence --range medium --continuous --count 60 --interval 3000 --save-to quick_angle_test.csv
```

### **Basic False Positive Test** (10 minutes):
```bash
sudo xm125-radar-monitor presence --range long --continuous --count 200 --interval 3000 --save-to quick_false_positive_test.csv
```

---

## 🔧 Troubleshooting

### **Common Issues**:

#### **No Detection**:
```bash
# Check sensor status
sudo xm125-radar-monitor status

# Check register configuration
sudo xm125-radar-monitor --debug-registers presence --range long
```

#### **Inconsistent Results**:
```bash
# Reset and recalibrate
sudo xm125-radar-monitor gpio reset-run
sudo xm125-radar-monitor presence --range long
```

#### **Permission Errors**:
```bash
# Ensure proper permissions
sudo chmod +x /usr/local/bin/xm125-radar-monitor
sudo usermod -a -G i2c fio
```

---

## 📋 Test Results Template

### **Range Test Results**:
- Maximum Detection Distance: _____ meters
- Reliable Detection Distance: _____ meters  
- Signal Strength at 5m: _____
- Signal Strength at 7m: _____
- Comparison with Windows Tool: _____

### **Angle Test Results**:
- Full Detection Angle: ±_____ degrees
- Partial Detection Angle: ±_____ degrees
- Edge Detection Angle: ±_____ degrees
- Comparison with Windows Tool: _____

### **False Positive Results**:
- Empty Room False Positive Rate: _____%
- With Interference False Positive Rate: _____%
- Most Common False Positive Trigger: _____
- Comparison with Windows Tool: _____

---

## 📞 Support

For technical issues or questions:
- **Tool Issues**: Check GitHub repository issues
- **Hardware Issues**: Contact hardware team lead
- **Data Analysis**: Use provided CSV analysis commands

**Remember**: All tests should be run with `sudo` for proper I2C access.

---

**Good luck with your testing! 🎯**
