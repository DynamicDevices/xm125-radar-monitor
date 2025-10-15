# XM125 Radar Monitor Testing Guide for Sentai Target

## Updated Features to Test

The latest version includes several important improvements:

### 🎯 New Defaults
- **I2C Bus 2**: Now uses `/dev/i2c-2` by default (where XM125 is located on Sentai)
- **Auto-reconnect**: Enabled by default for robust connection handling
- **Zero Warnings**: All compiler warnings eliminated for production quality

### 🔧 New CLI Options
- `--no-auto-reconnect`: Disable auto-reconnect for simple connection mode
- Better help text with Sentai-specific examples

## Testing Procedure

### 1. Deploy Latest Binary
```bash
# Copy the latest binary to Sentai target
scp -P 26 target/aarch64-unknown-linux-gnu/release/xm125-radar-monitor fio@62.3.79.162:/tmp/xm125-radar-monitor-latest
ssh -p 26 fio@62.3.79.162 'chmod +x /tmp/xm125-radar-monitor-latest'
```

### 2. Basic Functionality Tests

#### Test 1: Default Configuration (Should work without flags)
```bash
# These should now work without specifying -b 2
sudo /tmp/xm125-radar-monitor-latest status
sudo /tmp/xm125-radar-monitor-latest info
```

#### Test 2: Verify I2C Bus Detection
```bash
# Confirm it's using the correct bus
sudo /tmp/xm125-radar-monitor-latest -v status
# Should show: "Using I2C device: /dev/i2c-2 at address 0x52"
```

#### Test 3: Help and Documentation
```bash
# Check updated help text
/tmp/xm125-radar-monitor-latest --help
# Should show default bus 2 and auto-reconnect enabled
```

### 3. Connection Tests

#### Test 4: Auto-reconnect (Default Behavior)
```bash
# Should use auto-reconnect by default
sudo /tmp/xm125-radar-monitor-latest connect --force
```

#### Test 5: Simple Connection Mode
```bash
# Test the new --no-auto-reconnect flag
sudo /tmp/xm125-radar-monitor-latest --no-auto-reconnect connect
```

### 4. Output Format Tests

#### Test 6: JSON Output
```bash
sudo /tmp/xm125-radar-monitor-latest --format json status
sudo /tmp/xm125-radar-monitor-latest --format json info
```

#### Test 7: CSV Output
```bash
sudo /tmp/xm125-radar-monitor-latest --format csv status
```

#### Test 8: Quiet Mode
```bash
sudo /tmp/xm125-radar-monitor-latest --quiet status
echo "Exit code: $?"
```

### 5. Configuration Tests

#### Test 9: Detector Configuration
```bash
sudo /tmp/xm125-radar-monitor-latest config --start 0.3 --length 2.5 --sensitivity 0.7
```

#### Test 10: Different Detector Modes
```bash
sudo /tmp/xm125-radar-monitor-latest --mode distance status
sudo /tmp/xm125-radar-monitor-latest --mode presence status
sudo /tmp/xm125-radar-monitor-latest --mode combined status
```

### 6. Hardware Verification

#### Test 11: I2C Device Scan
```bash
# Verify XM125 is still detected on bus 2
sudo i2cdetect -y 2
# Should show device at address 0x52
```

#### Test 12: Measurement Attempts
```bash
# Try basic measurements (may timeout if device still initializing)
sudo /tmp/xm125-radar-monitor-latest measure
sudo /tmp/xm125-radar-monitor-latest presence
```

## Expected Results

### ✅ Should Work
- Status and info commands with default settings
- Verbose logging showing I2C bus 2 usage
- All output formats (JSON, CSV, human-readable)
- Configuration updates
- Help text showing correct defaults

### ⚠️ May Still Have Issues
- Connection and measurement commands if XM125 hardware is stuck in initialization
- This is a hardware/firmware issue, not software

### 🔍 Key Improvements to Verify
1. **No need for `-b 2` flag** - should work automatically
2. **Auto-reconnect enabled** - should see retry attempts on connection failures  
3. **Better error messages** - more descriptive output
4. **Zero warnings** - clean compilation

## Troubleshooting

If tests fail:

1. **Check I2C permissions**: Ensure user is in `i2c` group
2. **Verify hardware**: Run `sudo i2cdetect -y 2` to confirm XM125 presence
3. **Check device status**: Look for "Initializing" vs other status codes
4. **Try verbose mode**: Add `-v` flag for detailed debugging output

## Performance Comparison

Compare the new version against the old one:
- Faster startup (no need to specify bus)
- More robust connection handling
- Better error reporting
- Cleaner output formatting

## Notes for Production Use

- The binary is now **production-ready** with zero warnings
- All code passes strict linting (Clippy pedantic mode)
- Cross-compilation tested for ARM64
- Documentation auto-generates correctly
- Pre-commit hooks ensure code quality
