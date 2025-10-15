# XM125 Hardware Test Plan

Comprehensive validation tests for XM125 radar module performance and reliability.

## Test Environment

- **Hardware**: Sentai i.MX8MM + XM125 module
- **Firmware**: All three detector types (distance, presence, breathing)
- **Test Duration**: 30 minutes per test suite
- **Data Collection**: Automated via `test_suite.sh`

## Core Functionality Tests

### 1. Device Communication
```bash
# I2C connectivity and register access
sudo ./xm125-radar-monitor status
sudo ./xm125-radar-monitor info
```

### 2. Firmware Management
```bash
# Test all firmware types
sudo ./xm125-radar-monitor firmware update distance --force
sudo ./xm125-radar-monitor firmware update presence --force
sudo ./xm125-radar-monitor firmware update breathing --force
```

### 3. Detection Range Tests

#### Distance Detection
- **Range**: 0.18m - 3.18m
- **Target**: Metal plate (20cm x 20cm)
- **Test Points**: 0.2m, 0.5m, 1.0m, 2.0m, 3.0m
- **Expected Accuracy**: ±2cm

#### Presence Detection
- **Range**: 0.5m - 7.0m (Long mode)
- **Target**: Human movement
- **Sensitivity**: Fast motion (intra) and slow motion (inter)
- **Detection Threshold**: >90% at 1m distance

## Performance Tests

### 1. Measurement Stability
```bash
# 100 consecutive measurements
sudo ./xm125-radar-monitor --mode presence monitor --count 100 --save-to stability_test.csv
```

### 2. Environmental Conditions
- **Temperature Range**: -10°C to +60°C
- **Humidity**: 10% to 90% RH
- **Vibration**: Operational during normal handling

### 3. Power Consumption
- **Active Mode**: <50mA @ 3.3V
- **Sleep Mode**: <1mA @ 3.3V

## Automated Test Suite

Run comprehensive tests:
```bash
./test_suite.sh
python3 analyze_test_results.py
```

### Test Coverage
- ✅ Basic connectivity and initialization
- ✅ All detector modes (distance, presence, breathing)
- ✅ Firmware update and verification
- ✅ Range accuracy across detection zones
- ✅ False positive/negative rates
- ✅ Continuous operation stability
- ✅ GPIO control and reset functionality

## Pass/Fail Criteria

| Test | Pass Criteria | Fail Threshold |
|------|---------------|----------------|
| Range Accuracy | ±5cm @ 1m | >±10cm |
| Detection Rate | >95% @ optimal range | <90% |
| False Positives | <2% in empty environment | >5% |
| Stability | <1% measurement variance | >3% |
| Firmware Update | 100% success rate | Any failure |

## Troubleshooting

### Common Issues
1. **Calibration Failures**: Check for nearby metal objects
2. **Range Inaccuracy**: Verify target material and size
3. **Detection Gaps**: Adjust sensitivity settings
4. **I2C Errors**: Verify hardware connections and power

### Debug Commands
```bash
# Verbose I2C logging
sudo ./xm125-radar-monitor --verbose status

# Hardware reset sequence
sudo /home/fio/xm125-control.sh --reset-run

# Manual I2C scan
sudo i2cdetect -y 2
```

## Test Data Analysis

The `analyze_test_results.py` script provides:
- Statistical analysis of measurement accuracy
- Detection performance metrics
- Environmental correlation analysis
- Performance trend visualization
- Pass/fail assessment with detailed reporting

## Validation Checklist

- [ ] Device responds to I2C commands
- [ ] All three firmware types flash successfully
- [ ] Distance measurements within ±5cm accuracy
- [ ] Presence detection >95% success rate
- [ ] No false positives in empty environment
- [ ] Continuous operation for 30+ minutes
- [ ] GPIO control functions correctly
- [ ] Firmware verification passes
- [ ] Temperature readings within expected range
- [ ] Power consumption within specifications
