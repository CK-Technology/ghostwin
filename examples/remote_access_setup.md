# Remote Access Setup Guide

## Directory Structure

```
tools/remote_access/
├── cktech/                    # CK Technology ScreenConnect
│   ├── cktech_client.exe     # ScreenConnect client executable
│   ├── cktech_client.msi     # ScreenConnect MSI installer
│   ├── install_cktech.bat    # Installation script
│   └── README.txt            # Connection instructions
├── resolvetech/              # Resolve Technology ScreenConnect  
│   ├── resolvetech_client.exe
│   ├── resolvetech_client.msi
│   ├── install_resolvetech.bat
│   └── README.txt
├── vnc/                      # VNC alternatives
│   ├── tightvnc_viewer.exe
│   ├── realvnc_viewer.exe
│   └── launch_vnc.bat
└── .options.txt              # GUI options: "CheckAll"
```

## Installation Scripts

### install_cktech.bat
```batch
@echo off
echo Installing CK Technology ScreenConnect Client...

REM Check if MSI exists, prefer MSI over EXE
if exist "%~dp0cktech_client.msi" (
    echo Installing via MSI...
    msiexec /i "%~dp0cktech_client.msi" /quiet /norestart
    if %errorlevel% equ 0 (
        echo ✅ CKTech ScreenConnect installed successfully
    ) else (
        echo ❌ MSI installation failed, trying EXE...
        goto :try_exe
    )
) else (
    :try_exe
    if exist "%~dp0cktech_client.exe" (
        echo Installing via EXE...
        "%~dp0cktech_client.exe" /S
        if %errorlevel% equ 0 (
            echo ✅ CKTech ScreenConnect installed successfully
        ) else (
            echo ❌ Installation failed
        )
    ) else (
        echo ❌ No CKTech client files found
    )
)

echo.
echo 📱 To connect remotely:
echo    1. Open CKTech ScreenConnect admin panel
echo    2. Look for this computer in the connected devices
echo    3. Double-click to connect
echo.
pause
```

### install_resolvetech.bat  
```batch
@echo off
echo Installing Resolve Technology ScreenConnect Client...

REM Check if MSI exists, prefer MSI over EXE
if exist "%~dp0resolvetech_client.msi" (
    echo Installing via MSI...
    msiexec /i "%~dp0resolvetech_client.msi" /quiet /norestart
    if %errorlevel% equ 0 (
        echo ✅ ResolveTech ScreenConnect installed successfully
    ) else (
        echo ❌ MSI installation failed, trying EXE...
        goto :try_exe
    )
) else (
    :try_exe
    if exist "%~dp0resolvetech_client.exe" (
        echo Installing via EXE...
        "%~dp0resolvetech_client.exe" /S
        if %errorlevel% equ 0 (
            echo ✅ ResolveTech ScreenConnect installed successfully
        ) else (
            echo ❌ Installation failed
        )
    ) else (
        echo ❌ No ResolveTech client files found
    )
)

echo.
echo 📱 To connect remotely:
echo    1. Open ResolveTech ScreenConnect admin panel
echo    2. Look for this computer in the connected devices  
echo    3. Double-click to connect
echo.
pause
```

### launch_vnc.bat
```batch
@echo off
echo Available VNC Viewers:
echo.
echo 1. TightVNC Viewer
echo 2. RealVNC Viewer  
echo 3. Exit
echo.
set /p choice="Select VNC viewer (1-3): "

if "%choice%"=="1" (
    if exist "%~dp0tightvnc_viewer.exe" (
        start "" "%~dp0tightvnc_viewer.exe"
    ) else (
        echo ❌ TightVNC Viewer not found
        pause
    )
) else if "%choice%"=="2" (
    if exist "%~dp0realvnc_viewer.exe" (
        start "" "%~dp0realvnc_viewer.exe"
    ) else (
        echo ❌ RealVNC Viewer not found
        pause
    )
) else if "%choice%"=="3" (
    exit /b
) else (
    echo Invalid choice
    pause
)
```

## README Templates

### CKTech README.txt
```
CK Technology ScreenConnect Client

INSTALLATION:
- Run install_cktech.bat to install the client
- Client will auto-connect to CKTech ScreenConnect server

CONNECTION INFO:
- Server: [Your CKTech ScreenConnect URL]
- Support: [Your CKTech contact info]

USAGE:
1. Install the client during Windows setup
2. Client runs automatically and appears in admin panel
3. Technician can connect remotely to assist with installation
4. Client can be uninstalled after setup is complete

SECURITY:
- Client uses encrypted connection
- Session can be terminated by end user
- No persistent access after uninstall
```

### ResolveTech README.txt  
```
Resolve Technology ScreenConnect Client

INSTALLATION:
- Run install_resolvetech.bat to install the client
- Client will auto-connect to ResolveTech ScreenConnect server

CONNECTION INFO:
- Server: [Your ResolveTech ScreenConnect URL]  
- Support: [Your ResolveTech contact info]

USAGE:
1. Install the client during Windows setup
2. Client runs automatically and appears in admin panel
3. Technician can connect remotely to assist with installation
4. Client can be uninstalled after setup is complete

SECURITY:
- Client uses encrypted connection
- Session can be terminated by end user
- No persistent access after uninstall
```

## GhostWin Configuration

Update your `ghostwin.toml` to include remote access in the tools configuration:

```toml
[tools]
folders = [
    "Tools", 
    "ToolsSystem",
    "ToolsHardware", 
    "ToolsNetwork",
    "ToolsRemoteAccess",  # Enable remote access category
    "ToolsNirsoft",
    "PEAutoRun", 
    "Logon"
]
auto_detect = true

[tools.categories]
system = { icon = "🔧", description = "System utilities and diagnostics" }
hardware = { icon = "💽", description = "Hardware diagnostics and benchmarks" }  
network = { icon = "🌐", description = "Network tools and connectivity" }
remote_access = { icon = "📱", description = "Remote support and access tools" }
nirsoft = { icon = "🔍", description = "NirSoft system utilities" }
```

## Security Considerations

### ScreenConnect Clients
- **Temporary Access**: Install only during setup, uninstall afterward
- **Encrypted**: All connections use TLS encryption
- **Audited**: All sessions are logged in ScreenConnect
- **User Control**: End user can terminate sessions

### VNC Viewers
- **Local Use**: For connecting TO other systems, not FROM
- **Password Protected**: Target systems should use VNC passwords
- **Firewall**: Ensure appropriate firewall rules

## Usage Workflow

1. **During PE Phase**: 
   - Install ScreenConnect client for remote assistance
   - Client auto-connects and appears in admin panel

2. **During Windows Install**:
   - Technician can connect remotely to monitor progress
   - Assist with disk partitioning, driver issues, etc.

3. **Post-Install**:
   - Continue remote assistance for software installation
   - Configure system settings remotely
   - Uninstall client when setup complete

## Auto-Installation Option

You can also add remote access clients to the `pe_autorun/services/` directory to auto-install at PE boot:

```
pe_autorun/services/remote_access/
├── auto_install_cktech.bat       # Auto-install CKTech client at PE boot
├── auto_install_resolvetech.bat  # Auto-install ResolveTech client at PE boot  
└── .options.txt                  # "CollapseTree" - minimize in GUI
```

This gives you flexible remote access options during the entire Windows deployment process!