# GhostWin Command Reference

## Table of Contents

- [CLI Overview](#cli-overview)
- [Global Options](#global-options)
- [Commands](#commands)
  - [build](#build)
  - [gui](#gui)
  - [validate](#validate)
  - [tools](#tools)
- [Configuration File](#configuration-file)
- [Environment Variables](#environment-variables)
- [Exit Codes](#exit-codes)
- [Examples](#examples)

## CLI Overview

```
ghostwin [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

GhostWin provides a multi-command CLI interface for Windows deployment automation.

### Basic Usage

```bash
# Show help
ghostwin --help
ghostwin <command> --help

# Enable verbose logging
ghostwin --verbose <command>

# Show version
ghostwin --version
```

## Global Options

These options apply to all commands:

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--verbose` | `-v` | Enable debug logging | `false` |
| `--help` | `-h` | Show help information | - |
| `--version` | `-V` | Show version information | - |

### Examples

```bash
# Enable verbose output for any command
ghostwin --verbose build --source-iso Windows11.iso --output-dir build --output-iso GhostWin.iso

# Show help for specific command
ghostwin build --help
```

## Commands

### build

Build a custom Windows ISO with WinPE integration.

#### Syntax

```bash
ghostwin build [OPTIONS] --source-iso <SOURCE_ISO> --output-dir <OUTPUT_DIR> --output-iso <OUTPUT_ISO>
```

#### Required Arguments

| Argument | Description | Example |
|----------|-------------|---------|
| `--source-iso` | Path to source Windows ISO | `"C:\Windows11.iso"` |
| `--output-dir` | Directory for ISO extraction and building | `"C:\temp\build"` |
| `--output-iso` | Path for final ISO output | `"C:\GhostWin.iso"` |

#### Optional Arguments

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--extra-files <PATH>` | `-e` | Additional files to inject | None |
| `--config <PATH>` | `-c` | Configuration file path | `ghostwin.toml` |
| `--skip-packages` | | Skip WinPE package installation | `false` |
| `--skip-dpi-fix` | | Skip DPI scaling registry fix | `false` |

#### Examples

```bash
# Basic build
ghostwin build \
  --source-iso "C:\Downloads\Windows11.iso" \
  --output-dir "C:\temp\build" \
  --output-iso "C:\GhostWin.iso"

# Build with extra files
ghostwin build \
  --source-iso "Windows11.iso" \
  --output-dir "build" \
  --output-iso "GhostWin.iso" \
  --extra-files "C:\MyTools"

# Build with custom config
ghostwin build \
  --source-iso "Windows11.iso" \
  --output-dir "build" \
  --output-iso "GhostWin.iso" \
  --config "production.toml"

# Build without WinPE packages (faster)
ghostwin build \
  --source-iso "Windows11.iso" \
  --output-dir "build" \
  --output-iso "GhostWin.iso" \
  --skip-packages

# Build with verbose logging
ghostwin --verbose build \
  --source-iso "Windows11.iso" \
  --output-dir "build" \
  --output-iso "GhostWin.iso"
```

#### Build Process

The build command executes these steps:

1. **Input Validation** - Verify ISO exists, output directory writable
2. **ISO Extraction** - Extract source ISO using 7-Zip
3. **WIM Mounting** - Mount boot.wim using DISM
4. **Helper Files** - Copy GhostWin helper and detected tools
5. **Extra Files** - Copy additional files if specified
6. **WinPE Packages** - Inject configured packages (unless skipped)
7. **Registry Fixes** - Apply DPI scaling fix (unless skipped)
8. **WIM Unmounting** - Commit changes and unmount
9. **ISO Creation** - Build final ISO using oscdimg

#### Output

```
INFO  GhostWin v0.1.0 starting
INFO  Building Windows ISO with WinPE integration
INFO  Step 1: Extracting source ISO
INFO  Step 2: Mounting WIM image  
INFO  Step 3: Copying helper files
INFO  Step 4: Copying extra files
INFO  Step 5: Adding WinPE packages
INFO  Step 6: Applying DPI fix
INFO  Step 7: Unmounting and committing WIM
INFO  Step 8: Creating final ISO
INFO  ✅ GhostWin ISO build completed successfully!
INFO  Output: C:\GhostWin.iso
```

---

### gui

Launch the WinPE GUI interface.

#### Syntax

```bash
ghostwin gui
```

#### Description

Launches the Slint-based graphical interface for tool management and Windows installation within the WinPE environment.

#### Status

🚧 **In Development** - This command is planned for future implementation.

#### Examples

```bash
# Launch GUI
ghostwin gui
```

#### Expected Features

- Tool browser and launcher
- Install mode selection (Normal vs Automated)
- Script selection and execution
- System status display
- VNC server management
- Network configuration

---

### validate

Validate configuration and system dependencies.

#### Syntax

```bash
ghostwin validate
```

#### Description

Performs comprehensive validation of the system environment, dependencies, and configuration to ensure GhostWin can operate correctly.

#### Validation Checks

| Check | Description | Required |
|-------|-------------|----------|
| **Admin Privileges** | Verify running as administrator | ✅ Yes |
| **DISM** | Check DISM availability | ✅ Yes |
| **7-Zip** | Check 7z.exe in PATH | ✅ Yes |
| **oscdimg** | Check Windows ADK oscdimg tool | ✅ Yes |
| **Configuration** | Validate config file syntax | ⚠️ Warning only |
| **Tool Folders** | Check tool directory existence | ⚠️ Warning only |
| **ADK Path** | Verify custom ADK path | ⚠️ If specified |
| **Security** | Check password/VNC configuration | ⚠️ Warning only |

#### Examples

```bash
# Basic validation
ghostwin validate

# Validation with verbose output
ghostwin --verbose validate
```

#### Output

```
INFO  🔍 Validating GhostWin configuration and dependencies
INFO  ✅ Administrator privileges confirmed
INFO  ✅ All required dependencies found
INFO  ✅ Configuration loaded successfully
WARN  ⚠️  Tool folder not found: Tools (will be created during build)
WARN  ⚠️  No access protection configured
INFO  🔗 VNC server enabled on port 5950

📊 Validation Summary:
WARN  ⚠️  2 warning(s) found
```

#### Exit Codes

- `0` - Validation passed (warnings allowed)
- `1` - Validation failed (errors found)

---

### tools

Show detected tools and scripts.

#### Syntax

```bash
ghostwin tools
```

#### Description

Scans the configured tool directories and displays all detected tools, scripts, and their categories. Useful for verifying tool detection before building an ISO.

#### Tool Categories

- **🔧 Tools** - Manual execution tools
- **⚡ PEAutoRun** - Auto-run scripts
- **🏁 Logon** - Post-install scripts

#### File Type Icons

- **📋** - Executable files (.exe, .com, .bat, .cmd)
- **📄** - Script files (.ps1, .au3, .reg, .vbs)

#### Examples

```bash
# Scan for tools
ghostwin tools

# Scan with verbose output
ghostwin --verbose tools
```

#### Output

```
INFO  🔍 Scanning for tools and scripts
📁 Found 2 tool directories:
  - concept/windows-setup-helper-master/Helper/Tools
  - concept/windows-setup-helper-master/Helper/PEAutoRun

📂 Tools in concept/windows-setup-helper-master/Helper/Tools:
  🔧 7-zip.exe 📋
  🔧 Putty.exe 📋
  🔧 Autoruns64.exe 📋
  🔧 ReactOS Paint.exe 📋

📂 Tools in concept/windows-setup-helper-master/Helper/PEAutoRun:
  ⚡ vncserver.bat 📋 (auto-run)
  ⚡ netbird.exe 📋 (auto-run)

⚙️  Options for concept/windows-setup-helper-master/Helper/PEAutoRun:
  - Collapse tree view by default

📊 Summary: 6 tools found across 2 directories
```

## Configuration File

GhostWin uses TOML or JSON configuration files. Default names: `ghostwin.toml` or `ghostwin.json`.

### File Format

#### TOML Format (Recommended)

```toml
[iso]
wim_index = "Microsoft Windows Setup (amd64)"
mount_path = "C:\\temp\\WIMMount"  # Optional
adk_path = "C:\\Program Files (x86)\\Windows Kits\\10\\Assessment and Deployment Kit"  # Optional

[winpe]
packages = [
    "WinPE-WMI",
    "WinPE-NetFX",
    "WinPE-Scripting",
    "WinPE-PowerShell",
    "WinPE-StorageWMI",
    "WinPE-DismCmdlets"
]
disable_dpi_scaling = true
set_resolution = "1024x768"

[tools]
folders = ["Tools", "PEAutoRun", "Logon"]
auto_detect = true

[security]
password_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"  # Optional
access_secret = "mysecret"  # Optional
vnc_enabled = true
vnc_port = 5950
vnc_password = "vncwatch"
```

#### JSON Format

```json
{
  "iso": {
    "wim_index": "Microsoft Windows Setup (amd64)",
    "mount_path": "C:\\temp\\WIMMount",
    "adk_path": "C:\\Program Files (x86)\\Windows Kits\\10\\Assessment and Deployment Kit"
  },
  "winpe": {
    "packages": [
      "WinPE-WMI",
      "WinPE-NetFX",
      "WinPE-Scripting",
      "WinPE-PowerShell"
    ],
    "disable_dpi_scaling": true,
    "set_resolution": "1024x768"
  },
  "tools": {
    "folders": ["Tools", "PEAutoRun", "Logon"],
    "auto_detect": true
  },
  "security": {
    "vnc_enabled": true,
    "vnc_port": 5950,
    "vnc_password": "vncwatch"
  }
}
```

### Configuration Options

#### `[iso]` Section

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `wim_index` | String | WIM image index or name | `"Microsoft Windows Setup (amd64)"` |
| `mount_path` | String | Custom WIM mount directory | Temporary directory |
| `adk_path` | String | Custom Windows ADK path | Auto-detected |

#### `[winpe]` Section

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `packages` | Array | WinPE packages to inject | `["WinPE-WMI", "WinPE-NetFX", "WinPE-Scripting", "WinPE-PowerShell"]` |
| `disable_dpi_scaling` | Boolean | Apply DPI scaling fix | `true` |
| `set_resolution` | String | Default screen resolution | `"1024x768"` |

#### Available WinPE Packages

| Package | Description |
|---------|-------------|
| `WinPE-WMI` | Windows Management Instrumentation |
| `WinPE-NetFX` | .NET Framework support |
| `WinPE-Scripting` | Windows Script Host |
| `WinPE-PowerShell` | PowerShell environment |
| `WinPE-StorageWMI` | Storage management tools |
| `WinPE-DismCmdlets` | DISM PowerShell cmdlets |
| `WinPE-WinReCfg` | Windows Recovery tools |
| `WinPE-WiFi-Package` | WiFi networking support |
| `WinPE-Dot3Svc` | 802.1X authentication |
| `WinPE-PPPoE` | PPPoE networking |

#### `[tools]` Section

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `folders` | Array | Tool folder names to scan | `["Tools", "PEAutoRun", "Logon"]` |
| `auto_detect` | Boolean | Scan all drives for tool folders | `true` |

#### `[security]` Section

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `password_hash` | String | SHA-256 hash of access password | None |
| `access_secret` | String | Secret for challenge-response auth | None |
| `vnc_enabled` | Boolean | Enable VNC server | `true` |
| `vnc_port` | Integer | VNC server port | `5950` |
| `vnc_password` | String | VNC access password | `"vncwatch"` |

## Environment Variables

GhostWin respects these environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `PROGRAMFILES(X86)` | Program Files (x86) directory | Used for ADK detection |
| `TEMP` | Temporary directory | Used for WIM mounting |
| `PATH` | Executable search path | Used to find 7z.exe |

## Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Success | Command completed successfully |
| `1` | General Error | Command failed with error |
| `2` | Invalid Arguments | Invalid command line arguments |
| `3` | Permission Denied | Administrator privileges required |
| `4` | Dependency Missing | Required dependency not found |
| `5` | File Not Found | Required file does not exist |
| `6` | Configuration Error | Invalid configuration |

### Example Exit Code Handling

```bash
# Bash
ghostwin build --source-iso Windows11.iso --output-dir build --output-iso GhostWin.iso
if [ $? -eq 0 ]; then
    echo "Build successful"
else
    echo "Build failed with code $?"
fi

# PowerShell
ghostwin build --source-iso Windows11.iso --output-dir build --output-iso GhostWin.iso
if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful"
} else {
    Write-Host "Build failed with code $LASTEXITCODE"
}

# Batch
ghostwin build --source-iso Windows11.iso --output-dir build --output-iso GhostWin.iso
if %ERRORLEVEL% EQU 0 (
    echo Build successful
) else (
    echo Build failed with code %ERRORLEVEL%
)
```

## Examples

### Complete Workflows

#### Development Workflow

```bash
# 1. Validate environment
ghostwin validate
if [ $? -ne 0 ]; then exit 1; fi

# 2. Check tool detection  
ghostwin tools

# 3. Build test ISO
ghostwin --verbose build \
  --source-iso "Windows11.iso" \
  --output-dir "test-build" \
  --output-iso "test.iso" \
  --config "dev.toml"

# 4. Build production ISO
ghostwin build \
  --source-iso "Windows11.iso" \
  --output-dir "prod-build" \
  --output-iso "production.iso" \
  --config "production.toml" \
  --extra-files "C:\ProductionTools"
```

#### Automated Build Script

```bash
#!/bin/bash
set -e

# Configuration
SOURCE_ISO="$1"
OUTPUT_ISO="$2"
BUILD_DIR="$(mktemp -d)"

echo "Building GhostWin ISO..."
echo "Source: $SOURCE_ISO"
echo "Output: $OUTPUT_ISO"
echo "Build Directory: $BUILD_DIR"

# Validate
echo "Validating environment..."
ghostwin validate

# Build
echo "Building ISO..."
ghostwin build \
  --source-iso "$SOURCE_ISO" \
  --output-dir "$BUILD_DIR" \
  --output-iso "$OUTPUT_ISO"

# Cleanup
echo "Cleaning up..."
rm -rf "$BUILD_DIR"

echo "Build completed: $OUTPUT_ISO"
```

#### PowerShell Build Script

```powershell
param(
    [Parameter(Mandatory=$true)]
    [string]$SourceIso,
    
    [Parameter(Mandatory=$true)]
    [string]$OutputIso,
    
    [string]$Config = "ghostwin.toml",
    [string]$ExtraFiles,
    [switch]$Verbose
)

# Build arguments
$args = @(
    "build"
    "--source-iso", $SourceIso
    "--output-dir", "build-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
    "--output-iso", $OutputIso
    "--config", $Config
)

if ($ExtraFiles) {
    $args += "--extra-files", $ExtraFiles
}

# Global arguments
$globalArgs = @()
if ($Verbose) {
    $globalArgs += "--verbose"
}

# Execute
& ghostwin @globalArgs @args

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build completed successfully: $OutputIso" -ForegroundColor Green
} else {
    Write-Error "❌ Build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}
```

### Integration Examples

#### CI/CD Pipeline

```yaml
# GitHub Actions example
name: Build GhostWin ISO
on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Dependencies
      run: |
        # Install Windows ADK
        choco install windows-adk-winpe
        # Install 7-Zip
        choco install 7zip
    
    - name: Validate Environment
      run: cargo run -- validate
    
    - name: Build ISO
      run: |
        cargo run -- build \
          --source-iso "test-data/Windows11.iso" \
          --output-dir "build" \
          --output-iso "artifacts/GhostWin.iso"
    
    - name: Upload Artifact
      uses: actions/upload-artifact@v2
      with:
        name: ghostwin-iso
        path: artifacts/GhostWin.iso
```

#### Docker Build

```dockerfile
# Windows container for building ISOs
FROM mcr.microsoft.com/windows/servercore:ltsc2022

# Install dependencies
RUN powershell -Command \
    "Set-ExecutionPolicy Bypass -Force; \
     iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1')); \
     choco install -y windows-adk-winpe 7zip"

# Copy GhostWin
COPY ghostwin.exe C:/tools/
COPY ghostwin.toml C:/tools/

WORKDIR C:/build
ENTRYPOINT ["C:/tools/ghostwin.exe"]
```

This comprehensive command reference provides all the information needed to effectively use GhostWin's CLI interface.