# GhostWin Troubleshooting Guide

This guide helps resolve common installation and build issues with GhostWin.

## Quick Solutions

### 🔧 Most Common Issue: "Updating crates.io index" Hangs

**Problem**: Cargo hangs at "Updating crates.io index" during build.

**NEW: Automatic Environment Optimization** (v0.3.3+):
The installer now automatically configures your Rust/Cargo environment for optimal Windows performance, including:
- Network retry and timeout settings
- Windows Defender exclusions (if admin)
- Git compatibility configuration
- Memory and CPU optimization
- Pre-compiled dependency caching

**Quick Fix**: Use pre-built binaries instead:
```powershell
# Download and run installer with pre-built binaries
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -PreBuilt
```

**Advanced Fix**: Reset Cargo configuration:
```powershell
# Fix Cargo network issues
./install.ps1 -FixCargo
# Then try normal install again (with automatic environment optimization)
./install.ps1
```

**Manual Environment Skip** (if automatic config causes issues):
```powershell
# Skip automatic environment configuration
./install.ps1 -SkipEnvConfig
```

## Installation Issues

### Automatic Environment Optimization (NEW)

**What It Does**: 
Starting with v0.3.3, the installer automatically optimizes your Rust/Cargo environment for Windows:

- **Network Settings**: Configures timeouts, retries, and connection handling
- **Performance**: Sets optimal job counts and memory usage
- **Security**: Adds Windows Defender exclusions for Rust tools (admin only)
- **Compatibility**: Configures Git for better Cargo integration
- **Caching**: Pre-warms dependency cache for faster builds

**Benefits**:
- Prevents "Updating crates.io index" hangs
- Faster compilation times
- Better corporate network compatibility
- Reduced antivirus interference

**Skip If Needed**:
```powershell
# Skip automatic configuration if it causes issues
./install.ps1 -SkipEnvConfig
```

### Visual Studio Build Tools Missing

**Symptoms**:
- "linker not found" error
- "MSVC not installed" error

**Solution**:
```powershell
# Auto-install Build Tools
./install.ps1  # Choose option 1 when prompted
```

**Manual Solution**:
1. Download Visual Studio Build Tools: https://aka.ms/vs/17/release/vs_buildtools.exe
2. Install with C++ workload: `vs_buildtools.exe --add Microsoft.VisualStudio.Workload.VCTools`

### Rust Installation Issues

**Symptoms**:
- "cargo: command not found"
- Rust toolchain errors

**Solution**:
```powershell
# Install Rust manually
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Restart PowerShell and try again
```

### Permission Denied Errors

**Symptoms**:
- "Access is denied"
- "Permission denied"

**Solutions**:
1. Run PowerShell as Administrator
2. Temporarily disable antivirus
3. Add exclusion for GhostWin folder in Windows Defender

## Build Issues

### Network/Firewall Problems

**Symptoms**:
- "failed to get" errors
- Timeouts during dependency download
- Corporate network issues

**Solutions**:

1. **Configure Cargo for corporate networks**:
```powershell
# Fix Cargo configuration
./install.ps1 -FixCargo
```

2. **Manual proxy configuration** (if behind corporate firewall):
```powershell
# Set proxy for Cargo
$env:HTTPS_PROXY = "http://your-proxy:port"
$env:HTTP_PROXY = "http://your-proxy:port"
# Then run installer
```

3. **Use pre-built binaries** (bypasses all network issues):
```powershell
./install.ps1 -PreBuilt
```

### Out of Disk Space

**Symptoms**:
- "No space left on device"
- Build fails during compilation

**Solutions**:
1. Free up at least 3GB of disk space
2. Change install location: `./install.ps1 -InstallPath "D:\GhostWin"`
3. Use pre-built binaries: `./install.ps1 -PreBuilt`

### Antivirus Interference

**Symptoms**:
- Random build failures
- Files being deleted during build
- Permission errors

**Solutions**:
1. Add GhostWin folder to antivirus exclusions
2. Temporarily disable real-time protection during build
3. Use pre-built binaries to avoid compilation

## Windows ADK/PE Issues

### ADK Installation Fails

**Symptoms**:
- "Windows ADK not found"
- PE add-on missing

**Solutions**:

1. **Automatic via winget** (Windows 10 1709+):
```powershell
winget install Microsoft.WindowsADK
winget install Microsoft.ADKPEAddOn
```

2. **Manual download**:
- ADK: https://aka.ms/adk
- PE Add-on: https://aka.ms/adkpe

3. **Skip ADK for now**:
```powershell
./install.ps1 -SkipBuild  # Just get the source code
```

## Advanced Troubleshooting

### Complete Reset

If all else fails, completely reset the environment:

```powershell
# 1. Remove existing installation
Remove-Item "C:\ProgramData\CKTech\GhostWin" -Recurse -Force -ErrorAction SilentlyContinue

# 2. Reset Cargo (if using source build)
./install.ps1 -FixCargo

# 3. Clean install with pre-built binaries
./install.ps1 -PreBuilt
```

### Manual Build

If the installer fails, try building manually:

```powershell
# 1. Clone repository
git clone https://github.com/CK-Technology/ghostwin.git
cd ghostwin

# 2. Configure Cargo
mkdir $env:USERPROFILE\.cargo\config
@"
[net]
retry = 3
git-fetch-with-cli = true

[http]
timeout = 300
multiplexing = false
"@ | Out-File $env:USERPROFILE\.cargo\config\config.toml

# 3. Build with verbose output
cargo build --release --verbose
```

### Check System Requirements

Ensure your system meets minimum requirements:

- **OS**: Windows 10 1709+ or Windows 11
- **RAM**: 8GB+ recommended
- **Disk**: 5GB free space
- **Network**: Internet connection for dependencies
- **Admin**: Administrator privileges recommended

### Get More Help

If problems persist:

1. **Check logs**: Look for detailed error messages in PowerShell output
2. **GitHub Issues**: https://github.com/CK-Technology/ghostwin/issues
3. **Documentation**: Review README.md and DOCS.md
4. **Community**: Join discussions on GitHub

### Common Error Messages

| Error | Solution |
|-------|----------|
| `Updating crates.io index` (hangs) | Use `-PreBuilt` or `-FixCargo` |
| `linker not found` | Install Visual Studio Build Tools |
| `permission denied` | Run as Administrator |
| `failed to get` | Check network/firewall settings |
| `cargo: command not found` | Install Rust properly |
| `MSVC not found` | Install Visual Studio Build Tools with C++ |

### Performance Tips

- **First build**: Takes 10-30 minutes depending on internet speed
- **Subsequent builds**: Should be much faster (2-5 minutes)
- **Pre-built option**: Downloads in 1-2 minutes, no compilation needed
- **Disk space**: Build artifacts can use 2-3GB, clean with `cargo clean`

---

## Installation Commands Quick Reference

```powershell
# Standard installation (compile from source with environment optimization)
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

# Fast installation (pre-built binaries)
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -PreBuilt

# Fix Cargo issues and reset environment
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -FixCargo

# Skip automatic environment optimization
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -SkipEnvConfig

# Custom install location
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -InstallPath "D:\Tools\GhostWin"

# Skip Rust check (if already installed)
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -SkipRust

# Download source only (no build)
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -SkipBuild
```
