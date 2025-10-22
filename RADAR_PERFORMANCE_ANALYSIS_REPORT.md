# XM125 Radar Module Performance Analysis Report

**To**: Acconeer AB - XM125 Engineering Team  
**From**: Dynamic Devices Ltd - Hardware Engineering  
**Date**: October 22, 2025  
**Subject**: Performance Optimization Request - XM125 Presence Detection

## Executive Summary

We have developed comprehensive testing capabilities for the XM125 radar module and identified performance gaps compared to Acconeer's Windows evaluation tool. This report outlines our current testing methodology, observed performance limitations, and requests technical guidance for optimization.

## Current Testing Capabilities

### Hardware Platform
- **Target**: i.MX8MM (ARM Cortex-A53) running Yocto Linux
- **Interface**: I2C communication at 400kHz
- **Firmware**: Presence Detector Application (App ID: 2)
- **GPIO Control**: Hardware reset, bootloader mode, wake control
- **Power**: 3.3V supply with proper decoupling

### Testing Framework
We have implemented a comprehensive testing suite with the following capabilities:

#### 1. Range Detection Testing
```bash
# Configurable detection ranges
--range short   # 0.06m - 0.7m
--range medium  # 0.2m - 2.0m  
--range long    # 0.5m - 7.0m
--min-range X --max-range Y  # Custom ranges (0.06m - 7.0m)
```

#### 2. Signal Quality Analysis
- **Fast Motion Detection** (intra_presence_score): Rapid movement detection
- **Slow Motion Detection** (inter_presence_score): Gradual movement detection  
- **Signal Strength Classification**: STRONG (>2.0), MEDIUM (1.0-2.0), WEAK (0.5-1.0), NONE (<0.5)
- **Detection Confidence**: HIGH/MEDIUM/LOW based on combined scores

#### 3. Continuous Monitoring & Data Logging
- High-precision timestamping (millisecond accuracy)
- CSV export with 9 data fields for analysis
- Configurable measurement intervals (100ms - 10s)
- Long-duration testing (hours) for statistical analysis

#### 4. Register-Level Debugging
Full register dump capability for presence detector configuration:
```
0x0052 (82) │ Start Point     │ 0x000001F4 (500)  │ Start distance in mm
0x0053 (83) │ End Point       │ 0x00001B58 (7000) │ End distance in mm  
0x0046 (70) │ Intra Threshold │ 0x00000514 (1300) │ Fast motion threshold
0x0047 (71) │ Inter Threshold │ 0x000003E8 (1000) │ Slow motion threshold
```

## Performance Gaps Identified

### 1. Detection Range Limitations
- **Observed Maximum**: ~6m reliable detection in optimal conditions
- **Expected Performance**: 7-8m based on datasheet specifications
- **Signal Degradation**: Rapid drop-off beyond 5m (STRONG → WEAK)

### 2. Angular Coverage Concerns  
- **Current Field of View**: Estimated ±30-45° reliable detection
- **Comparison Needed**: Windows evaluation tool shows superior angular coverage
- **Edge Detection**: Inconsistent performance at detection boundaries

### 3. False Positive Sensitivity
- **Empty Room Performance**: Occasional false detections (~1-2% rate)
- **Environmental Sensitivity**: Increased false positives with HVAC, fluorescent lighting
- **Threshold Optimization**: Current thresholds may not be optimal for our use case

## Technical Questions for Acconeer

### 1. Register Configuration Optimization
**Question**: Are our current register settings optimal for room occupancy detection?

**Current Configuration**:
```
Intra Threshold: 1300 (0x514)
Inter Threshold: 1000 (0x3E8)  
Frame Rate: 12Hz (12000 mHz)
Sweeps Per Frame: 16
```

**Request**: Recommended register values for:
- Maximum detection range (7m+ reliable)
- Reduced false positive rate in office environments
- Optimal balance between sensitivity and stability

### 2. Hardware Implementation Validation
**Question**: Is our I2C implementation following best practices?

**Current Implementation**:
- 400kHz I2C clock frequency
- Standard Linux i2cdev driver
- Register read/write with proper timing
- Hardware reset sequence: 10ms LOW, 100ms HIGH, 100ms wake delay

**Request**: 
- Validation of timing parameters
- Recommended I2C frequency for optimal performance
- GPIO timing requirements verification

### 3. Firmware Configuration Analysis
**Question**: Are there undocumented registers or configuration options that could improve performance?

**Current Access**: Standard presence detector registers (0x0040-0x0080)

**Request**:
- Advanced configuration registers documentation
- Calibration procedures for specific environments
- Performance tuning parameters not in standard documentation

### 4. Environmental Optimization
**Question**: How can we optimize for typical office/industrial environments?

**Target Environment**:
- Room size: 3m x 4m to 10m x 15m
- Ceiling height: 2.4m - 3.5m
- Interference: LED lighting, HVAC systems, WiFi, computers
- Detection requirement: Human presence (stationary and moving)

**Request**:
- Environment-specific tuning recommendations
- Interference mitigation strategies
- Multi-zone detection configuration guidance

## Proposed Collaboration

### 1. Performance Benchmarking
We propose to conduct side-by-side testing with:
- Our Linux-based implementation
- Acconeer Windows evaluation tool
- Identical test conditions and environments
- Shared test data and analysis

### 2. Register Configuration Optimization
**Deliverable**: Optimized register configuration file
**Timeline**: 2-3 weeks testing cycle
**Metrics**: Range, angular coverage, false positive rate

### 3. Documentation Enhancement
**Request**: Enhanced technical documentation covering:
- Advanced register descriptions and interactions
- Environment-specific optimization guidelines  
- Troubleshooting guide for common performance issues
- Reference implementations for embedded Linux platforms

## Technical Data We Can Provide

### 1. Detailed Performance Metrics
- Range vs. signal strength curves
- Angular detection patterns
- False positive correlation with environmental factors
- Long-term stability measurements (24+ hours)

### 2. Register Analysis Data
- Current vs. optimal register configurations
- Performance impact of individual register changes
- Environmental sensitivity analysis per register setting

### 3. Hardware Implementation Details
- Complete schematic of XM125 integration
- PCB layout and signal integrity measurements
- Power supply characteristics and noise analysis
- GPIO timing measurements

## Requested Deliverables from Acconeer

### 1. Immediate (1-2 weeks)
- Validation of our current register configuration
- Recommended optimization for office environment detection
- Confirmation of I2C implementation best practices

### 2. Medium-term (1 month)
- Advanced configuration documentation
- Environment-specific tuning guide
- Performance optimization consultation session

### 3. Long-term (Ongoing)
- Technical support for performance optimization
- Early access to firmware updates or configuration tools
- Collaboration on embedded Linux reference implementation

## Success Metrics

We propose to measure optimization success using:

| Metric | Current | Target | Measurement Method |
|--------|---------|--------|--------------------|
| **Max Reliable Range** | ~6m | 7-8m | Controlled distance testing |
| **Detection Angle** | ±30° | ±45° | Angular sweep testing |
| **False Positive Rate** | 1-2% | <0.5% | 24-hour empty room testing |
| **Signal Quality at 5m** | MEDIUM | STRONG | Signal strength analysis |
| **Environmental Stability** | Variable | Consistent | Multi-condition testing |

## Contact Information

**Primary Contact**: Hardware Engineering Team  
**Technical Lead**: [Contact Details]  
**Testing Facility**: Available for remote collaboration  
**Timeline**: Ready to begin optimization testing immediately

## Appendix

### A. Testing Framework Code
- Complete source code available on GitHub
- Automated testing scripts and analysis tools
- CSV data export for detailed analysis

### B. Hardware Documentation  
- Complete schematics and PCB layout
- Bill of materials and component specifications
- Signal integrity and power analysis reports

### C. Current Test Results
- Baseline performance measurements
- Environmental sensitivity data
- Comparative analysis framework

---

**We look forward to collaborating with Acconeer's engineering team to optimize XM125 performance for our embedded Linux platform and achieve the full potential of this excellent radar technology.**
