# GhostWin Setup Guide

## Directory Organization

### 📁 Recommended Structure

```
ghostwin/                       # Main project directory
├── ghostwin.exe               # The Rust executable
├── ghostwin.toml              # Configuration file
│
├── Tools/                     # 🔧 Manual tools (shown in GUI)
│   ├── MyDiskTool.exe
│   ├── NetworkDiag.bat  
│   ├── SystemInfo.ps1
│   └── .Options.txt           # Optional: "CheckAll", "CollapseTree"
│
├── PEAutoRun/                 # ⚡ Auto-run scripts (run at WinPE boot)
│   ├── NetworkSetup.bat
│   ├── DriverInstall.ps1
│   ├── VPNConnect.exe
│   └── .Options.txt
│
├── Logon/                     # 🏁 Post-install scripts (run after Windows install)
│   ├── SoftwareInstall.ps1
│   ├── ConfigureOS.bat
│   ├── AdminSetup[system].ps1  # [system] = run as SYSTEM
│   ├── Monitor[background].exe  # [background] = don't wait
│   └── .Options.txt
│
└── concept/                   # 🔄 Legacy AutoIt tools (optional to keep)
    └── windows-setup-helper-master/
        └── Helper/
            ├── Tools/         # Existing tools will be detected
            ├── PEAutoRun/     # Existing auto-run scripts
            └── Scripts/       # Will be treated as Logon scripts
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

This setup gives you maximum flexibility while maintaining compatibility with existing AutoIt tools.