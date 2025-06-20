# GhostWin Setup Guide

[![Windows](https://img.shields.io/badge/Platform-Windows-0078D4?style=flat-square&logo=windows)](https://www.microsoft.com/windows)
[![Automation](https://img.shields.io/badge/Automation-Ready-00D4AA?style=flat-square&logo=automattic)](https://github.com/yourusername/ghostwin)

**Complete setup and deployment guide for GhostWin**

## Quick Start

### 🚀 Concept

1. **Transfer to Windows machine** - Copy entire project folder
2. **Install Rust** - Download from [rustup.rs](https://rustup.rs/)
3. **Build project** - Run `cargo build --release`
4. **Launch GUI** - Run `./target/release/ghostwin.exe gui`
5. **Demo ready!** - Professional interface with deep ocean blue theme

### 📁 Directory Organization

#### Recommended Project Structure

```
ghostwin/                       # Main project directory
├── ghostwin.exe               # The Rust executable (after build)
├── ghostwin.toml              # Configuration file
├── assets/                    # Screenshots and documentation assets
│   └── ghostwin.png          # GUI screenshot
│
├── tools/                     # 🔧 System tools (shown in GUI)
│   ├── system/
│   ├── network/
│   ├── hardware/
│   └── remote_access/
│
├── pe_autorun/               # ⚡ Auto-run scripts (run at WinPE boot)
│   ├── services/             # VNC, NetBird, etc.
│   ├── system_setup/         # Registry tweaks, profiles
│   └── associations/         # File associations
│
├── scripts/                  # 🏁 Post-install automation
│   ├── basic/                # Simple setup scripts
│   ├── advanced/             # Complex automation
│   └── vendor/               # Vendor-specific configs
│
└── config/                   # 🔧 System configurations
    ├── autounattend.xml      # Windows installation automation
    └── winpeshl.ini          # WinPE shell configuration
```

### 🗂️ External Storage

```
# Store Windows ISOs separately (NOT in ghostwin directory):
C:\WindowsISOs\
├── Windows11-24H2.iso
├── Windows10-22H2.iso  
└── Server2022.iso

# Optional: External tool collections
C:\MyGhostWinTools\
├── Tools/
├── PEAutoRun/
└── Logon/
```

## Setup Steps

### 1. Create Tool Directories

```bash
# In your ghostwin directory:
mkdir Tools PEAutoRun Logon
```

### 2. Add Your Scripts

#### Tools/ - Manual execution tools
```bash
# Examples:
copy "C:\MyTools\DiskUtility.exe" Tools\
copy "C:\Scripts\NetworkCheck.bat" Tools\
copy "C:\Utils\SystemInfo.ps1" Tools\

# Optional: Configure defaults
echo CheckAll > Tools\.Options.txt
```

#### PEAutoRun/ - Auto-run at WinPE boot  
```bash
# Examples:
copy "C:\Scripts\NetworkSetup.bat" PEAutoRun\
copy "C:\Drivers\InstallDrivers.ps1" PEAutoRun\

# Optional: Collapse tree in GUI
echo CollapseTree > PEAutoRun\.Options.txt
```

#### Logon/ - Post-Windows-install scripts
```bash
# Examples:
copy "C:\Scripts\InstallSoftware.ps1" Logon\
copy "C:\Scripts\ConfigureWindows.bat" Logon\

# System context script (runs as SYSTEM before user logon):
copy "C:\Scripts\SystemSetup.bat" "Logon\SystemSetup[system].bat"

# Background script (doesn't block next script):  
copy "C:\Scripts\StartMonitoring.exe" "Logon\StartMonitoring[background].exe"
```

### 3. Test Tool Detection

```bash
# Verify tools are detected
ghostwin tools

# Should show:
# 📁 Found X tool directories
# 📂 Tools in Tools/: 🔧 DiskUtility.exe 📋
# 📂 Tools in PEAutoRun/: ⚡ NetworkSetup.bat 📋 (auto-run)
# 📂 Tools in Logon/: 🏁 InstallSoftware.ps1 📄
```

### 4. Build Your First ISO

```bash
# Basic build
ghostwin build \
  --source-iso "C:\WindowsISOs\Windows11-24H2.iso" \
  --output-dir "C:\temp\build" \
  --output-iso "C:\GhostWin.iso"

# With external tools
ghostwin build \
  --source-iso "C:\WindowsISOs\Windows11-24H2.iso" \
  --output-dir "C:\temp\build" \
  --output-iso "C:\GhostWin.iso" \
  --extra-files "C:\MyGhostWinTools"
```

## File Types Supported

| Extension | Category | Execution Method |
|-----------|----------|------------------|
| `.exe`, `.com` | Executable | Direct execution |
| `.bat`, `.cmd` | Batch | Command processor |
| `.ps1` | PowerShell | PowerShell host |
| `.au3` | AutoIt | AutoIt interpreter (if present) |
| `.reg` | Registry | Registry import |
| `.vbs` | VBScript | Windows Script Host |

## Special File Naming

- **Hidden files**: Start with `.` (e.g., `.HiddenTool.exe`)
- **System context**: Add `[system]` suffix (e.g., `Setup[system].bat`) 
- **Background execution**: Add `[background]` suffix (e.g., `Monitor[background].exe`)

## Migration from AutoIt

### Current AutoIt Tools

Your existing AutoIt tools in `concept/windows-setup-helper-master/Helper/` are automatically detected:

- ✅ **Tools/** - Already works
- ✅ **PEAutoRun/** - Already works  
- ✅ **Scripts/** - Treated as Logon scripts
- ✅ **Scripts - Basic/** - Also treated as Logon scripts

### Gradual Migration

You can migrate tools gradually:

1. **Keep existing** - `concept/` folder for compatibility
2. **Add new** - Put new tools in `Tools/`, `PEAutoRun/`, `Logon/`
3. **Test mixed** - Both will be detected and included
4. **Full migration** - Eventually move everything to new structure
5. **Remove old** - Delete `concept/` when no longer needed

## Tool Options (.Options.txt)

Create `.Options.txt` in any tool folder:

```
# Check all items by default
CheckAll

# Collapse tree view in GUI  
CollapseTree

# Check specific items by default
MyTool.exe
ImportantScript.bat
CriticalFix.reg
```

## Advanced Configurations

### Multiple Tool Sources

```toml
# ghostwin.toml
[tools]
folders = [
    "Tools", 
    "ToolsNetwork",     # Custom category
    "ToolsDisk",        # Custom category
    "PEAutoRun",
    "PEAutoRunDrivers", # Custom category
    "Logon",
    "LogonSoftware"     # Custom category
]
auto_detect = true      # Also scan all drives for Helper\Tools\ etc.
```

### Security Settings

```toml
[security]
# Password protect the WinPE interface
password_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

# Enable VNC remote access
vnc_enabled = true
vnc_port = 5950
vnc_password = "vncwatch"
```

## Troubleshooting

### "No tools found"
```bash
# Check tool detection
ghostwin tools

# Verify file extensions are supported
# Verify files are in correct directories
# Check .Options.txt syntax
```

### "Build failed"
```bash
# Validate environment first
ghostwin validate

# Use verbose mode for details
ghostwin --verbose build --source-iso ... --output-dir ... --output-iso ...
```

### "Permission denied"
```bash
# Run as Administrator
# Required for DISM operations
```

## 🎯 Demo Workflow for Boss Presentation

### Step-by-Step Demo Script

#### 1. **Opening - Professional Introduction**
```
"This is GhostWin - our new Windows deployment toolkit built with Rust. 
It replaces our old batch script approach with a modern, branded interface."
```

#### 2. **Show the Main Interface**
- Launch: `ghostwin gui`
- **Highlight**: Deep ocean blue professional theme
- **Point out**: GhostWin branding and clean layout
- **Explain**: "This runs directly in WinPE, replacing the standard Windows setup"

#### 3. **Demonstrate Installation Modes**
- **Normal Install**: "Standard Windows installation - no modifications"
- **Automated Install**: "Full automation with our custom scripts and tools"
- **Emphasize**: "One-click deployment with complete control"

#### 4. **Show Tool Management**
- Click "Tools Manager" in sidebar
- **System Tools section**: "All our diagnostic and repair utilities"
- **Automation Scripts section**: "Registry tweaks, configurations, post-install automation"
- **Demo**: Click a few tool launch buttons

#### 5. **VNC Remote Access**
- **Show VNC controls** in sidebar
- **Explain**: "Built-in remote access for difficult deployments"
- **Highlight**: Connection status and one-click control

#### 6. **System Status Panel**
- **Point out**: Tool count, VNC status, deployment readiness
- **Explain**: "Real-time system information at a glance"

#### 7. **Technical Benefits Summary**
```
"Built with Rust for reliability and speed
Professional interface suitable for client demonstrations  
Replaces fragile batch scripts with robust automation
Integrates all our tools in one cohesive interface
Remote access capability for complex deployments"
```

#### 8. **Business Impact**
```
"Faster deployments = more machines per day
Professional appearance for client sites
Reduced training time for new technicians
Standardized deployment process across all jobs"
```

### 💡 Demo Tips

- **Keep it moving** - Don't dwell on technical details
- **Focus on visual appeal** - The dark theme looks professional
- **Emphasize automation** - "One click instead of manual steps"
- **Highlight branding** - "This represents our company professionally"
- **Show remote capability** - "We can assist deployments remotely"

### 🎬 Demo Duration: 5-7 minutes optimal