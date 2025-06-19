# GhostWin Documentation

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Requirements](#requirements)
- [Getting Started](#getting-started)
- [Configuration](#configuration)
- [Workflows](#workflows)
- [Tool Management](#tool-management)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)

## Overview

GhostWin is a modern, Rust-powered Windows deployment toolkit that builds custom Windows installer ISOs with embedded WinPE environments. It provides both CLI and GUI workflows for IT professionals who need to automate Windows installations with custom tools, scripts, and configurations.

### Key Features

- **🔧 Modern CLI** - Replace batch scripts with a robust command-line interface
- **🏗️ WIM Management** - Automated Windows Image (WIM) mounting and modification
- **📦 Tool Integration** - Automatic detection and packaging of tools and scripts
- **⚡ WinPE Enhancement** - Package injection and system modifications
- **🔐 Security** - Password protection and VNC remote access
- **📋 Validation** - Comprehensive system and configuration checking

## Installation

### Prerequisites

1. **Windows ADK** (Assessment and Deployment Kit)
   - Download from [Microsoft](https://docs.microsoft.com/en-us/windows-hardware/get-started/adk-install)
   - Install both ADK and WinPE Add-on
   - Ensure versions match your target Windows version

2. **7-Zip** 
   - Download from [7-zip.org](https://www.7-zip.org/)
   - Ensure `7z.exe` is in your PATH

3. **Administrator Privileges**
   - Required for DISM operations and WIM mounting

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd ghostwin

# Build the project
cargo build --release

# The executable will be at target/release/ghostwin.exe
```

### Verification

```bash
# Check installation
ghostwin validate

# View help
ghostwin --help
```

## Requirements

### System Requirements

- **Windows 10/11** (for WIM operations)
- **4GB+ RAM** (8GB+ recommended for large ISOs)
- **20GB+ free disk space** (for ISO extraction and building)
- **Administrator privileges**

### Software Dependencies

| Tool | Purpose | Auto-Detected |
|------|---------|---------------|
| **DISM** | WIM mounting/unmounting | ✅ |
| **7-Zip** | ISO extraction | ✅ |
| **oscdimg** | ISO creation | ✅ |
| **Windows ADK** | WinPE packages | ✅ |

Run `ghostwin validate` to check all dependencies.

## Getting Started

### Quick Start

1. **Prepare your environment:**
   ```bash
   # Validate system
   ghostwin validate
   ```

2. **Create a basic configuration:**
   ```bash
   # This creates ghostwin.toml in current directory
   # (Configuration is optional - defaults will be used)
   ```

3. **Build your first ISO:**
   ```bash
   ghostwin build \
     --source-iso "C:\Windows11.iso" \
     --output-dir "C:\temp\build" \
     --output-iso "C:\GhostWin.iso"
   ```

4. **Check what tools were detected:**
   ```bash
   ghostwin tools
   ```

### Directory Structure

GhostWin expects this directory structure for maximum compatibility:

```
project/
├── ghostwin.exe
├── ghostwin.toml          # Optional configuration
├── concept/               # Optional: AutoIt compatibility
│   └── windows-setup-helper-master/
│       └── Helper/
│           ├── Tools/     # Manual tools
│           ├── PEAutoRun/ # Auto-run scripts
│           └── Logon/     # Post-install scripts
├── Tools/                 # Additional tools folder
├── PEAutoRun/            # Additional auto-run folder
└── Logon/                # Additional logon folder
```

## Configuration

### Configuration File

GhostWin supports both TOML and JSON configuration files. By default, it looks for:
- `ghostwin.toml`
- `ghostwin.json`

### Example Configuration

```toml
# ghostwin.toml

[iso]
# WIM image to modify (usually boot.wim)
wim_index = "Microsoft Windows Setup (amd64)"
# Optional: Custom mount path (default: temp directory)
# mount_path = "C:\\temp\\WIMMount"
# Optional: Custom ADK path
# adk_path = "C:\\Program Files (x86)\\Windows Kits\\10\\Assessment and Deployment Kit"

[winpe]
# WinPE packages to inject
packages = [
    "WinPE-WMI",           # Windows Management Instrumentation
    "WinPE-NetFX",         # .NET Framework
    "WinPE-Scripting",     # Scripting support
    "WinPE-PowerShell",    # PowerShell
    "WinPE-StorageWMI",    # Storage management
    "WinPE-DismCmdlets"    # DISM PowerShell cmdlets
]
# Fix DPI scaling issues in WinPE
disable_dpi_scaling = true
# Set default resolution
set_resolution = "1024x768"

[tools]
# Folders to scan for tools (supports prefixes like "ToolsCustom")
folders = ["Tools", "PEAutoRun", "Logon"]
# Auto-detect tools on all drives
auto_detect = true

[security]
# SHA-256 hash of access password (optional)
# password_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
# Secret for challenge-response authentication (optional)
# access_secret = "mysecret"
# VNC server configuration
vnc_enabled = true
vnc_port = 5950
vnc_password = "vncwatch"
```

### Configuration Sections

#### `[iso]` Section
- **`wim_index`**: Which WIM image to modify (usually boot.wim)
- **`mount_path`**: Custom directory for mounting WIM (optional)
- **`adk_path`**: Custom Windows ADK installation path (optional)

#### `[winpe]` Section
- **`packages`**: Array of WinPE packages to inject
- **`disable_dpi_scaling`**: Fix DPI scaling issues (recommended: true)
- **`set_resolution`**: Default screen resolution for WinPE

#### `[tools]` Section
- **`folders`**: Tool folder names to scan for
- **`auto_detect`**: Scan all drives for matching folders

#### `[security]` Section
- **`password_hash`**: SHA-256 hash for access protection
- **`access_secret`**: Secret for challenge-response auth
- **`vnc_enabled`**: Enable VNC server in WinPE
- **`vnc_port`**: VNC server port (default: 5950)
- **`vnc_password`**: VNC access password

## Workflows

### Basic ISO Building

```bash
# Simple build with defaults
ghostwin build \
  --source-iso "Windows11.iso" \
  --output-dir "build" \
  --output-iso "GhostWin.iso"
```

### Advanced ISO Building

```bash
# Build with custom configuration and extra files
ghostwin build \
  --source-iso "Windows11.iso" \
  --output-dir "build" \
  --output-iso "GhostWin.iso" \
  --config "my-config.toml" \
  --extra-files "C:\MyCustomTools" \
  --verbose
```

### Build Process Steps

The build process follows these steps:

1. **Validation** - Check dependencies and inputs
2. **ISO Extraction** - Extract source ISO to working directory
3. **WIM Mounting** - Mount boot.wim for modification
4. **File Copying** - Copy helper files and tools
5. **Package Injection** - Add WinPE packages
6. **Registry Fixes** - Apply system modifications
7. **WIM Unmounting** - Commit changes and unmount
8. **ISO Creation** - Build final bootable ISO

### Customization Workflow

1. **Prepare Tools:**
   ```bash
   # Create tool directories
   mkdir Tools PEAutoRun Logon
   
   # Add your tools
   cp MyTool.exe Tools/
   cp AutoScript.bat PEAutoRun/
   cp PostInstall.ps1 Logon/
   ```

2. **Configure Options:**
   ```bash
   # Create .Options.txt in tool folders
   echo "CheckAll" > Tools/.Options.txt
   echo "CollapseTree" > PEAutoRun/.Options.txt
   ```

3. **Test Detection:**
   ```bash
   ghostwin tools
   ```

4. **Build ISO:**
   ```bash
   ghostwin build --source-iso Windows11.iso --output-dir build --output-iso Custom.iso
   ```

## Tool Management

### Tool Categories

GhostWin organizes tools into three categories:

#### **Tools** - Manual Execution
- Shown in WinPE GUI for manual launch
- Examples: Disk utilities, network tools, diagnostics
- Folder naming: `Tools`, `ToolsNetwork`, `ToolsDisk`

#### **PEAutoRun** - Automatic Execution  
- Run automatically when WinPE starts
- Examples: Network setup, driver installation, system prep
- Folder naming: `PEAutoRun`, `PEAutoRunNetwork`, `PEAutoRunDrivers`

#### **Logon** - Post-Install Execution
- Run after Windows installation completes
- Examples: Software installation, configuration scripts
- Folder naming: `Logon`, `LogonSoftware`, `LogonConfig`

### Supported File Types

| Extension | Type | Execution |
|-----------|------|-----------|
| `.exe` | Executable | Direct execution |
| `.com` | Executable | Direct execution |
| `.bat` | Batch Script | Command processor |
| `.cmd` | Command Script | Command processor |
| `.ps1` | PowerShell | PowerShell host |
| `.au3` | AutoIt Script | AutoIt interpreter |
| `.reg` | Registry File | Registry import |
| `.vbs` | VBScript | Script host |

### Tool Options

Create `.Options.txt` in any tool folder to customize behavior:

```
# Check all items by default
CheckAll

# Collapse tree view in GUI
CollapseTree

# Check specific items by default
MyTool.exe
ImportantScript.bat
```

### Special File Naming

- **Hidden files**: Start with `.` (e.g., `.HiddenTool.exe`)
- **System context**: Add `[system]` to logon scripts (e.g., `Setup[system].bat`)
- **Background execution**: Add `[background]` to scripts (e.g., `Monitor[background].exe`)

### Multi-Drive Detection

GhostWin automatically scans all drives for tool folders:

```
C:\Helper\Tools\          ✅ Detected
D:\Helper\Tools\          ✅ Detected  
E:\Helper\ToolsNetwork\   ✅ Detected
F:\MyStuff\Tools\         ❌ Not detected (wrong path)
```

## Advanced Usage

### Custom WinPE Packages

Add specialized WinPE packages for enhanced functionality:

```toml
[winpe]
packages = [
    # Base packages
    "WinPE-WMI",
    "WinPE-NetFX", 
    "WinPE-Scripting",
    "WinPE-PowerShell",
    
    # Storage and disk management
    "WinPE-StorageWMI",
    "WinPE-DismCmdlets",
    
    # Network and remote access
    "WinPE-WinReCfg",
    "WinPE-WiFi-Package",
    
    # Development and debugging
    "WinPE-Dot3Svc",
    "WinPE-PPPoE"
]
```

### Registry Modifications

The DPI scaling fix applies these registry changes:

```registry
[HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\SideBySide]
"PreferExternalManifest"=dword:00000001

[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\Configuration]
"DisableScalingOptimizations"=dword:00000001
```

### Security Configuration

#### Password Protection
```bash
# Generate password hash (SHA-256)
echo -n "mypassword" | sha256sum
# Add to config file
password_hash = "89e01536ac207279409d4de1e5253e01f4a1769e696db0d6062ca9b8f56767c8"
```

#### Challenge-Response Authentication
```toml
[security]
access_secret = "mysecretkey"
# GUI will show challenge code, user must provide response
```

### VNC Remote Access

Enable remote access to WinPE:

```toml
[security]
vnc_enabled = true
vnc_port = 5950
vnc_password = "vncwatch"
```

Connect using any VNC client:
```bash
# Connect to WinPE system
vncviewer <ip-address>:5950
```

### Batch Operations

Process multiple ISOs:

```bash
# Build script
for iso in *.iso; do
    echo "Building $iso..."
    ghostwin build \
        --source-iso "$iso" \
        --output-dir "build-$(basename "$iso" .iso)" \
        --output-iso "ghost-$(basename "$iso")"
done
```

## Troubleshooting

### Common Issues

#### "Administrator privileges required"
```bash
# Run from elevated command prompt
# Right-click Command Prompt → "Run as administrator"
```

#### "DISM mount failed" 
```bash  
# Check if WIM is already mounted
dism /Get-MountedImageInfo

# Clean up if needed
dism /Cleanup-Wim
dism /Cleanup-Mountpoints
```

#### "7-Zip extraction failed"
```bash
# Ensure 7z.exe is in PATH
where 7z

# Or specify full path in config
```

#### "Package not found"
```bash
# Check ADK installation
dir "C:\Program Files (x86)\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment\amd64\WinPE_OCs\"

# Update package names in config
```

### Debug Mode

Enable verbose logging:

```bash
ghostwin --verbose build --source-iso Windows11.iso --output-dir build --output-iso GhostWin.iso
```

### Log Analysis

GhostWin provides structured logging:

```
INFO  GhostWin v0.1.0 starting
INFO  Building Windows ISO with WinPE integration  
INFO  Step 1: Extracting source ISO
DEBUG Extracting ISO Windows11.iso to build
INFO  Step 2: Mounting WIM image
DEBUG Mounting WIM: build/sources/boot.wim (index: Microsoft Windows Setup (amd64))
```

### Recovery Procedures

#### Clean up failed build:
```bash
# Stop any running DISM operations
taskkill /f /im dism.exe

# Clean mounted images
dism /Cleanup-Wim
dism /Cleanup-Mountpoints

# Remove temporary files
rmdir /s build
```

#### Reset configuration:
```bash
# Remove config file to use defaults
del ghostwin.toml

# Validate fresh setup
ghostwin validate
```

