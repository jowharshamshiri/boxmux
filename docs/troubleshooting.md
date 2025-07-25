# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with BoxMux.

## Common Issues

### Installation Problems

#### Rust Not Found

**Problem**: `cargo: command not found`
**Solution**:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Build Failures

**Problem**: Compilation errors during `cargo build`
**Solution**:

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### Runtime Issues

#### Configuration File Not Found

**Problem**: `Error: Configuration file not found`
**Solution**:

- Verify file path: `ls -la layouts/dashboard.yaml`
- Use absolute path: `./run_boxmux.sh /full/path/to/config.yaml`
- Check file permissions: `chmod 644 layouts/dashboard.yaml`

#### YAML Syntax Errors

**Problem**: `Error parsing YAML`
**Solution**:

```bash
# Validate YAML syntax
yamllint layouts/dashboard.yaml

# Common fixes:
# - Check indentation (use spaces, not tabs)
# - Ensure proper quoting of strings
# - Verify array and object syntax
```

#### Script Execution Fails

**Problem**: Panel scripts don't execute
**Solution**:

- Check script permissions: `chmod +x script.sh`
- Verify shell path: Use absolute paths like `/bin/bash`
- Test script independently: `bash -c "your_script_here"`

### Performance Issues

#### High CPU Usage

**Problem**: BoxMux uses too much CPU
**Solution**:

- Increase refresh intervals in configuration
- Optimize scripts to run faster
- Reduce number of panels with scripts
- Check for infinite loops in scripts

### Display Issues

#### Corrupted Display

**Problem**: Interface appears broken or corrupted
**Solution**:

- Clear terminal: `clear` or `Ctrl+L`
- Resize terminal window
- Restart BoxMux
- Check terminal compatibility

#### Colors Not Working

**Problem**: Colors appear incorrect or missing
**Solution**:

- Check terminal color support: `echo $TERM`
- Use standard color names
- Test with different terminal emulators
- Check terminal theme settings

## Debugging

### Enable Debug Logging

```bash
# Run with debug output
RUST_LOG=debug ./run_boxmux.sh layouts/dashboard.yaml

# Filter specific modules
RUST_LOG=boxmux::draw_utils=debug ./run_boxmux.sh layouts/dashboard.yaml
```

### Check Application Logs

```bash
# View application logs
tail -f app.log

# Search for errors
grep -i error app.log
```

### Validate Configuration

```bash
# Check YAML syntax
python -c "import yaml; yaml.safe_load(open('layouts/dashboard.yaml'))"

# Or use online validator
cat layouts/dashboard.yaml | curl -X POST -H "Content-Type: application/yaml" -d @- https://yaml-validator.com/
```

### Test Individual Components

```bash
# Test script execution
bash -c "your_script_command_here"

# Test socket connection
echo '{"GetStatus": {}}' | nc -U /tmp/boxmux.sock
```

## Getting Help

### Before Asking for Help

1. **Check this troubleshooting guide**
2. **Search existing GitHub issues**
3. **Try the solution with minimal configuration**
4. **Gather relevant information**

### Information to Include

When reporting issues, include:

- BoxMux version: `cargo --version`
- Operating system: `uname -a`
- Terminal emulator and version
- Configuration file (if relevant)
- Steps to reproduce the issue
- Expected vs. actual behavior
- Error messages and logs

### Where to Get Help

- **GitHub Issues**: Bug reports and feature requests
- **Documentation**: Check all docs files
- **Examples**: Review example configurations

### Creating a Minimal Example

When reporting issues, create a minimal configuration that reproduces the problem:

```yaml
app:
  layouts:
    - id: 'test'
      root: true
      children:
        - id: 'problem_panel'
          title: 'Problem Panel'
          position:
            x1: 10%
            y1: 10%
            x2: 90%
            y2: 90%
          content: 'This demonstrates the issue'
```

## Recovery Procedures

### Reset Configuration

```bash
# Backup current configuration
cp layouts/dashboard.yaml layouts/dashboard.yaml.backup

# Use example configuration
cp layouts/dashboard.yaml.example layouts/dashboard.yaml
```

### Clear Application State

```bash
# Remove temporary files
rm -f /tmp/boxmux.sock
rm -f app.log

# Clear terminal
clear
```

### Emergency Stop

```bash
# Force quit BoxMux
pkill -f boxmux

# Or use Ctrl+C in terminal
# Or close terminal window
```
