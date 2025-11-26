# 🚀 XM125 Testing Quick Reference Card

## 📋 Essential Commands

### **Range Testing**
```bash
# Long range (0.5m - 7.0m) - 10 minutes
sudo xm125-radar-monitor presence --range long --continuous --count 200 --interval 3000 --save-to range_test.csv

# Custom range (test beyond spec)
sudo xm125-radar-monitor presence --min-range 1.0 --max-range 10.0 --continuous --save-to extended_range.csv
```

### **Angle Testing**  
```bash
# Medium range angle sweep - 5 minutes
sudo xm125-radar-monitor presence --range medium --continuous --count 150 --interval 2000 --save-to angle_test.csv

# With debug info
sudo xm125-radar-monitor --debug-registers presence --range medium --continuous --count 50 --save-to angle_debug.csv
```

### **False Positive Testing**
```bash
# Empty room test - 30 minutes
sudo xm125-radar-monitor presence --range long --continuous --count 900 --interval 2000 --save-to false_positive.csv

# Quick false positive check - 5 minutes  
sudo xm125-radar-monitor presence --range long --continuous --count 100 --interval 3000 --save-to quick_false_pos.csv
```

## 🎯 Signal Strength Guide

| Strength | Score Range | Meaning | Action |
|----------|-------------|---------|---------|
| **STRONG** | >2.0 | Excellent detection | ✅ Reliable |
| **MEDIUM** | 1.0-2.0 | Good detection | ✅ Usable |
| **WEAK** | 0.5-1.0 | Marginal detection | ⚠️ Edge case |
| **NONE** | <0.5 | No detection | ❌ Out of range |

## 📊 Quick Analysis Commands

### **Detection Rate**
```bash
# Calculate detection percentage
awk -F',' 'NR>1 {detected+=$2=="true"; total++} END {print "Detection Rate:", detected/total*100"%"}' your_test.csv
```

### **False Positive Rate**
```bash
# Calculate false positive percentage (empty room)
awk -F',' 'NR>1 {false_pos+=$2=="true"; total++} END {print "False Positive Rate:", false_pos/total*100"%"}' false_positive.csv
```

### **Distance Analysis**
```bash
# Show detected distances
awk -F',' 'NR>1 && $2=="true" {print $3}' range_test.csv | sort -n | uniq -c
```

## 🔧 Troubleshooting

### **No Detection**
```bash
sudo xm125-radar-monitor status
sudo xm125-radar-monitor --debug-registers presence --range long
```

### **Reset Sensor**
```bash
sudo xm125-radar-monitor gpio reset-run
```

### **Check Configuration**
```bash
sudo xm125-radar-monitor --debug-registers presence --range medium
```

## 📋 Test Checklist

### **Range Test** ✅
- [ ] Test at 1m, 2m, 3m, 4m, 5m, 6m, 7m distances
- [ ] Record maximum reliable detection distance
- [ ] Note signal strength at each distance
- [ ] Test with slow, normal, and fast movement
- [ ] Compare with Windows tool results

### **Angle Test** ✅  
- [ ] Test at 0°, ±15°, ±30°, ±45°, ±60°, ±75° angles
- [ ] Record detection reliability at each angle
- [ ] Test at 1.0m, 1.5m, 2.0m distances
- [ ] Map detection field of view
- [ ] Compare with Windows tool coverage

### **False Positive Test** ✅
- [ ] 30-minute empty room baseline
- [ ] Test with fan running
- [ ] Test with AC running  
- [ ] Test with fluorescent lights
- [ ] Test with electronics nearby
- [ ] Record all false detections
- [ ] Compare with Windows tool false positive rate

## 🎯 Expected Performance Targets

| Metric | Target | Comparison |
|--------|--------|------------|
| **Max Range** | 6-7m reliable | vs Windows tool |
| **Detection Angle** | ±45° reliable | vs Windows tool |
| **False Positive Rate** | <1% empty room | vs Windows tool |
| **Signal Quality** | MEDIUM+ at 5m | vs Windows tool |

## 📞 Emergency Commands

### **Stop All Tests**
```bash
Ctrl+C  # Stop current test
```

### **Quick Status Check**
```bash
sudo xm125-radar-monitor status
```

### **Full Reset**
```bash
sudo xm125-radar-monitor gpio reset-run
sudo xm125-radar-monitor presence --range long  # Test basic functionality
```

---

**Remember**: Always use `sudo` and save results to CSV files for analysis! 📊
