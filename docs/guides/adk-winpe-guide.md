# Windows ADK and WinPE Installation Guide

This guide covers manual installation of Windows Assessment and Deployment Kit (ADK) and Windows PE add-on for GhostWin.

## Overview

GhostWin requires two components from Microsoft:

1. **Windows ADK** - Assessment and Deployment Kit containing deployment tools (DISM, oscdimg, etc.)
2. **Windows PE Add-on** - Preinstallation Environment for creating bootable media

These are separate downloads since Windows 10 version 1809.

## Automatic vs Manual Installation

### Automatic (Recommended)

The GhostWin installer handles ADK/PE installation automatically:

```powershell
irm https://win.cktech.sh | iex
```

The installer targets **Windows 11 25H2** (ADK 10.1.26100.2454). This works for:
- Windows 11 25H2, 24H2, and all earlier supported versions
- Windows Server 2025 and 2022

### When to Install Manually

You may need manual installation if:
- You're on an older Windows version and need a specific ADK version
- You're in an air-gapped environment
- You want to use a different ADK version than the installer provides
- The automatic installation fails

---

## ADK Version Compatibility

| ADK Version | Supported Windows Versions |
|-------------|---------------------------|
| **10.1.26100.2454** (Dec 2024) | Windows 11 25H2, 24H2, 23H2, 22H2 + Windows 10 + Server 2025/2022 |
| **10.1.25398.1** (Jan 2025) | Windows 11 23H2 and earlier + Windows 10 + Server 2022 |
| **Windows 11 22H2** | Windows 11 22H2 and earlier + Windows 10 |
| **Windows 10 2004** | Windows 10 2004 and later versions of Windows 10 |

**Rule of thumb**: Use the ADK version matching your target Windows version, or the latest ADK for mixed environments.

---

## Download Links

### Latest (Windows 11 25H2) - Used by GhostWin Installer

| Component | Download |
|-----------|----------|
| ADK 10.1.26100.2454 | [Download](https://go.microsoft.com/fwlink/?linkid=2289980) |
| WinPE Add-on | [Download](https://go.microsoft.com/fwlink/?linkid=2289981) |

### Windows 11 23H2

| Component | Download |
|-----------|----------|
| ADK 10.1.25398.1 | [Download](https://go.microsoft.com/fwlink/?linkid=2243390) |
| WinPE Add-on | [Download](https://go.microsoft.com/fwlink/?linkid=2243391) |

### Windows 11 22H2

| Component | Download |
|-----------|----------|
| ADK for Windows 11 22H2 | [Download](https://go.microsoft.com/fwlink/?linkid=2196127) |
| WinPE Add-on | [Download](https://go.microsoft.com/fwlink/?linkid=2196224) |

### Windows Server 2022

| Component | Download |
|-----------|----------|
| ADK for Server 2022 | [Download](https://go.microsoft.com/fwlink/?linkid=2162950) |
| WinPE Add-on | [Download](https://go.microsoft.com/fwlink/?linkid=2163233) |

### Windows 10 (version 2004)

| Component | Download |
|-----------|----------|
| ADK for Windows 10 2004 | [Download](https://go.microsoft.com/fwlink/?linkid=2120254) |
| WinPE Add-on | [Download](https://go.microsoft.com/fwlink/?linkid=2120253) |

---

## Manual Installation Steps

### Step 1: Download Both Installers

Download the ADK installer and WinPE Add-on installer for your target Windows version from the links above.

### Step 2: Install Windows ADK

1. Run `adksetup.exe`
2. Accept the license agreement
3. Select installation location (default: `C:\Program Files (x86)\Windows Kits\10`)
4. Select these features (minimum for GhostWin):
   - **Deployment Tools** (required)
   - **User State Migration Tool (USMT)** (optional)
   - **Windows Performance Toolkit** (optional)
5. Click **Install**

### Step 3: Install Windows PE Add-on

1. Run `adkwinpesetup.exe`
2. Accept the license agreement
3. Use the same installation location as ADK
4. The installer will add Windows PE components
5. Click **Install**

### Step 4: Verify Installation

Open PowerShell and check:

```powershell
# Check DISM is available
Get-Command dism

# Check oscdimg is available
$oscdimg = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
Test-Path $oscdimg

# Check WinPE files exist
$winpe = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment\amd64"
Test-Path $winpe
```

---

## Silent Installation

For automated deployments:

```powershell
# Install ADK silently (Deployment Tools only)
.\adksetup.exe /quiet /features OptionId.DeploymentTools

# Install WinPE Add-on silently
.\adkwinpesetup.exe /quiet
```

---

## What GhostWin Uses

GhostWin's `ghostwin build` command uses these ADK components:

| Component | Location | Purpose |
|-----------|----------|---------|
| **DISM** | System PATH | Mount/unmount WIM images, inject drivers/packages |
| **oscdimg** | `Deployment Tools\amd64\Oscdimg\` | Create bootable ISO files |
| **WinPE base image** | `Windows Preinstallation Environment\amd64\` | Source for boot.wim customization |
| **WinPE packages** | `Windows Preinstallation Environment\amd64\WinPE_OCs\` | Optional components (scripting, networking, etc.) |

---

## Troubleshooting

### "DISM not found"

DISM should be in your system PATH after ADK installation. If not:

```powershell
$env:PATH += ";${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\DISM"
```

### "oscdimg not found"

GhostWin looks for oscdimg in the standard ADK location. Verify:

```powershell
Test-Path "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
```

### "WinPE files missing"

Make sure you installed the **WinPE Add-on** separately after ADK. It's not included in the base ADK installer.

### 32-bit WinPE not available

32-bit Windows PE was removed starting with ADK for Windows 11 22H2. The last version with 32-bit WinPE support is ADK for Windows 10 version 2004.

---

## Windows SDK vs Windows ADK

These are different products:

| Product | Purpose | GhostWin Uses |
|---------|---------|---------------|
| **Windows SDK** | Development APIs, headers, libraries for building Windows apps | No |
| **Windows ADK** | Deployment and assessment tools (DISM, WinPE, USMT) | Yes |

GhostWin only needs the **Windows ADK** and **WinPE Add-on**.

---

## References

- [Microsoft: Download and install the Windows ADK](https://learn.microsoft.com/en-us/windows-hardware/get-started/adk-install)
- [Microsoft: Windows PE (WinPE) Overview](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/winpe-intro)
- [Microsoft: Windows SDK Downloads](https://learn.microsoft.com/en-us/windows/apps/windows-sdk/downloads)
