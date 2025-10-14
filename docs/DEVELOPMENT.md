# Development Tools Setup

This document describes the development tools and hooks available for the XM125 Radar Monitor project.

## Git Hooks

The project includes Git hooks to automatically run quality checks and catch issues before they reach CI/CD.

### Quick Setup
```bash
# Hooks are already installed in .git/hooks/
# Just make sure they're executable (should be done automatically)
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
```

### Pre-commit Hook
Runs automatically before each commit:
- ✅ Code formatting check (`cargo fmt --check`)
- ✅ Clippy lints (`cargo clippy`)
- ✅ Compilation check (`cargo check`)
- ✅ Unit tests (`cargo test`)
- ⚠️ Warns about TODOs, debug prints, large files

### Pre-push Hook
Runs automatically before each push:
- **Main branch**: Comprehensive checks including documentation, security audit, cross-compilation
- **Feature branches**: Basic test suite

## Pre-commit Framework (Optional)

For advanced hook management, install the pre-commit framework:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks from .pre-commit-config.yaml
pre-commit install

# Run all hooks manually
pre-commit run --all-files
```

## Manual Quality Checks

Run these commands manually for development:

```bash
# Format code
cargo fmt --all

# Run lints
cargo clippy --all-targets --all-features -- -D warnings -A dead_code

# Run tests
cargo test --all-features

# Check documentation
cargo doc --no-deps --document-private-items --all-features

# Security audit (install with: cargo install cargo-audit)
cargo audit

# Cross-compile for ARM64
cargo check --target aarch64-unknown-linux-gnu --release
```

## CI/CD Integration

The project uses GitHub Actions for CI/CD with the following jobs:
1. **Security Audit** - Check for vulnerabilities
2. **Test and Quality Checks** - Format, lint, test, documentation
3. **Native Build** - Fast x86_64 build for code validation
4. **Cross-compile** - ARM64 build for embedded targets
5. **Release** - Create GitHub releases on tags

## Development Workflow

1. **Make changes** to the code
2. **Pre-commit hook** runs automatically on `git commit`
   - Fixes any formatting issues with `cargo fmt`
   - Fix any linting issues reported by Clippy
3. **Pre-push hook** runs automatically on `git push`
   - Ensures comprehensive quality for main branch
4. **CI/CD pipeline** runs on GitHub
   - Should pass quickly since local hooks caught issues early

## Bypassing Hooks

Only in emergencies:
```bash
# Skip pre-commit checks
git commit --no-verify -m "emergency fix"

# Skip pre-push checks
git push --no-verify
```

## Benefits

- 🚀 **Faster CI builds** - Issues caught locally before CI
- 🔧 **Consistent quality** - Automated formatting and linting
- 👥 **Team efficiency** - Prevent broken code in shared branches
- 🛡️ **Early detection** - Security and compatibility issues found early
