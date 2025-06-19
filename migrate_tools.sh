#!/bin/bash

# GhostWin Tool Migration Script
# Migrates tools from concept/ and exported/ to new organized structure

set -e

echo "🚀 Starting GhostWin tool migration..."

CONCEPT_DIR="concept/windows-setup-helper-master/Helper"
EXPORTED_DIR="exported/HelperOld stuff"

# Function to copy file with progress
copy_tool() {
    local src="$1"
    local dest="$2"
    local name="$3"
    
    if [ -f "$src" ]; then
        echo "  ✅ Copying $name..."
        cp "$src" "$dest"
    elif [ -d "$src" ]; then
        echo "  ✅ Copying $name (directory)..."
        cp -r "$src" "$dest"
    else
        echo "  ⚠️  Skipping $name (not found)"
    fi
}

echo ""
echo "🔧 === MIGRATING SYSTEM TOOLS ==="

# 7-Zip
copy_tool "$CONCEPT_DIR/Tools/7-Zip" "tools/system/7-zip" "7-Zip archive suite"

# Sysinternals Tools
copy_tool "$CONCEPT_DIR/Tools/Autoruns64.exe" "tools/system/autoruns64.exe" "Autoruns64"
copy_tool "$CONCEPT_DIR/Tools/disk2vhd64.exe" "tools/system/disk2vhd64.exe" "Disk2VHD"

# Password Reset
copy_tool "$CONCEPT_DIR/Tools/NTPWEdit64.exe" "tools/system/ntpwedit64.exe" "NT Password Editor"

echo ""
echo "💽 === MIGRATING HARDWARE TOOLS ==="

# Disk Tools
copy_tool "$CONCEPT_DIR/Tools/CrystalDiskInfo-SFX64.exe" "tools/hardware/crystaldiskinfo.exe" "CrystalDiskInfo"
copy_tool "$CONCEPT_DIR/Tools/CrystalDiskMark-SFX64.exe" "tools/hardware/crystaldiskmark.exe" "CrystalDiskMark"
copy_tool "$CONCEPT_DIR/Tools/gsmartcontrol-SFX64.exe" "tools/hardware/gsmartcontrol.exe" "GSmartControl"

echo ""
echo "🔧 === MIGRATING DRIVER TOOLS ==="

# Driver Management (from exported)
copy_tool "$EXPORTED_DIR/Scripts/DriverStoreExplorer.exe" "tools/drivers/driverstoreexplorer.exe" "Driver Store Explorer"
copy_tool "$EXPORTED_DIR/Scripts/NVCleanstall.exe" "tools/drivers/nvcleanstall.exe" "NVIDIA CleanInstall"

echo ""
echo "🌐 === MIGRATING NETWORK TOOLS ==="

# Putty Suite
copy_tool "$CONCEPT_DIR/Tools/Putty" "tools/network/putty" "Putty SSH/Telnet Suite"

# Network Scanners
copy_tool "$CONCEPT_DIR/Tools/AutoAngryIPScanner.exe" "tools/network/ipscanner.exe" "Angry IP Scanner"

# VNC Tools
copy_tool "$CONCEPT_DIR/Tools/VNCHelper" "tools/network/vnchelper" "VNC Helper"

echo ""
echo "📱 === MIGRATING REMOTE ACCESS TOOLS ==="

# CKTech ScreenConnect (from exported)
copy_tool "$EXPORTED_DIR/Scripts/CKTechSupport.msi" "tools/remote_access/cktech/cktech_support.msi" "CKTech Support MSI"
copy_tool "$EXPORTED_DIR/Scripts/cksupport-rdesk.msi" "tools/remote_access/cktech/cktech_rdesk.msi" "CKTech Remote Desktop"

# ResolveTech (from exported)
copy_tool "$EXPORTED_DIR/Scripts/ResolveTechSupportTool.exe" "tools/remote_access/resolvetech/resolvetech_support.exe" "ResolveTech Support Tool"

# VNC components from concept
if [ -d "$CONCEPT_DIR/PEAutoRun/vncserver" ]; then
    echo "  ✅ Copying VNC Server components..."
    cp "$CONCEPT_DIR/PEAutoRun/vncserver/tvnserver.exe" "tools/remote_access/vnc/" 2>/dev/null || true
    cp "$CONCEPT_DIR/PEAutoRun/vncserver/screenhooks64.dll" "tools/remote_access/vnc/" 2>/dev/null || true
fi

echo ""
echo "🔍 === MIGRATING NIRSOFT TOOLS ==="

# NirSoft Utilities
copy_tool "$CONCEPT_DIR/Tools/DevManView.exe" "tools/nirsoft/devmanview.exe" "Device Manager View"
copy_tool "$CONCEPT_DIR/Tools/FullEventLogView.exe" "tools/nirsoft/fulleventlogview.exe" "Full Event Log View"
copy_tool "$CONCEPT_DIR/Tools/SearchMyFiles.exe" "tools/nirsoft/searchmyfiles.exe" "Search My Files"

echo ""
echo "💿 === MIGRATING INSTALL TOOLS ==="

# Installation Utilities
copy_tool "$CONCEPT_DIR/Tools/Rufus.exe" "tools/install/rufus.exe" "Rufus USB Creator"
copy_tool "$CONCEPT_DIR/Tools/Bypass Win11 Requirments.bat" "tools/install/bypass_win11_req.bat" "Windows 11 Bypass"

echo ""
echo "🎨 === MIGRATING MEDIA TOOLS ==="

# Media Tools
copy_tool "$CONCEPT_DIR/Tools/ReactOS Paint.exe" "tools/media/reactos_paint.exe" "ReactOS Paint"
copy_tool "$CONCEPT_DIR/Tools/Seamonkey Web Browser.exe" "tools/media/seamonkey_browser.exe" "SeaMonkey Browser"

echo ""
echo "⚡ === MIGRATING PE AUTORUN COMPONENTS ==="

# System Setup Registry Files
copy_tool "$CONCEPT_DIR/PEAutoRun/Explorer++.reg" "pe_autorun/system_setup/explorer++.reg" "Explorer++ Settings"
copy_tool "$CONCEPT_DIR/PEAutoRun/FontFix.reg" "pe_autorun/system_setup/fontfix.reg" "Font Fix"
copy_tool "$CONCEPT_DIR/PEAutoRun/TaskMgrPrefs.reg" "pe_autorun/system_setup/taskmgr_prefs.reg" "Task Manager Preferences"

# System Setup Scripts
copy_tool "$CONCEPT_DIR/PEAutoRun/MakeSystemProfileFolders.bat" "pe_autorun/system_setup/make_profile_folders.bat" "Profile Folders Script"

# Services
copy_tool "$CONCEPT_DIR/PEAutoRun/vncserver" "pe_autorun/services/vnc_server" "VNC Server"
copy_tool "$CONCEPT_DIR/PEAutoRun/Drivers" "pe_autorun/services/drivers" "Driver Installation"
copy_tool "$CONCEPT_DIR/PEAutoRun/NetBird" "pe_autorun/services/netbird" "NetBird Mesh Networking"

echo ""
echo "📝 === MIGRATING SCRIPTS ==="

# Basic Scripts - Audio
copy_tool "$CONCEPT_DIR/Scripts - Basic/01 Reduce Volume.ps1" "scripts/basic/audio/reduce_volume.ps1" "Volume Reduction"

# Basic Scripts - Desktop
copy_tool "$CONCEPT_DIR/Scripts - Basic/02 Set Background Red.bat" "scripts/basic/desktop/set_background.bat" "Desktop Background"
copy_tool "$CONCEPT_DIR/Scripts - Basic/Admin Desktop Shortcuts.ps1" "scripts/basic/desktop/admin_shortcuts.ps1" "Admin Shortcuts"

# Basic Scripts - Registry
copy_tool "$CONCEPT_DIR/Scripts - Basic/Disable Auto Logon.reg" "scripts/basic/registry/disable_auto_logon.reg" "Disable Auto Logon"
copy_tool "$CONCEPT_DIR/Scripts - Basic/Disable Edge FirstRun.reg" "scripts/basic/registry/disable_edge_firstrun.reg" "Disable Edge FirstRun"
copy_tool "$CONCEPT_DIR/Scripts - Basic/Disable ModernLockScreen.reg" "scripts/basic/registry/disable_modern_lockscreen.reg" "Disable Modern Lock Screen"
copy_tool "$CONCEPT_DIR/Scripts - Basic/Enable Delete Confirm.reg" "scripts/basic/registry/enable_delete_confirm.reg" "Enable Delete Confirmation"
copy_tool "$CONCEPT_DIR/Scripts - Basic/Remove Run Registry.ps1" "scripts/basic/registry/remove_run_history.ps1" "Remove Run History"

# Advanced Scripts - Updates
copy_tool "$CONCEPT_DIR/Scripts/Start Windows Updates.bat" "scripts/advanced/updates/start_windows_updates.bat" "Start Windows Updates"
copy_tool "$CONCEPT_DIR/Scripts/Windows Update Settings.reg" "scripts/advanced/updates/windows_update_settings.reg" "Windows Update Settings"
copy_tool "$CONCEPT_DIR/Scripts/Set TargetVersion Win11-24H2.bat" "scripts/advanced/updates/set_target_version.bat" "Set Target Version"

# Advanced Scripts - System
copy_tool "$CONCEPT_DIR/Scripts/Debloat Example[system].bat" "scripts/advanced/system/debloat_system.bat" "System Debloat"
copy_tool "$CONCEPT_DIR/Scripts/Power Settings.bat" "scripts/advanced/system/power_settings.bat" "Power Settings (Conservative)"

# Advanced Scripts - System (from exported)
copy_tool "$EXPORTED_DIR/Scripts/PowerOptions.bat" "scripts/advanced/system/power_aggressive.bat" "Power Settings (Aggressive)"
copy_tool "$EXPORTED_DIR/Scripts/WindowsActivation.bat" "scripts/advanced/system/windows_activation.bat" "Windows Activation"

# Advanced Scripts - Network (from exported)
copy_tool "$EXPORTED_DIR/Scripts/ChangeNetConnectionProfile.ps1" "scripts/advanced/network/change_connection_profile.ps1" "Change Network Profile"

# Advanced Scripts - Vendor
copy_tool "$CONCEPT_DIR/Scripts/Dell Command Update (Online).ps1" "scripts/advanced/vendor/dell/command_update.ps1" "Dell Command Update"

echo ""
echo "⚙️ === MIGRATING CONFIGURATION FILES ==="

# Configuration Files
copy_tool "$CONCEPT_DIR/autounattend.xml" "config/autounattend.xml" "Windows Install Automation"
copy_tool "$CONCEPT_DIR/../Windows/System32/winpeshl.ini" "config/winpeshl.ini" "PE Shell Configuration"

# Resources
copy_tool "$CONCEPT_DIR/../Windows/Fonts/segoeui.ttf" "resources/fonts/segoeui.ttf" "Segoe UI Font"

echo ""
echo "📋 === CREATING OPTIONS FILES ==="

# Create .options.txt files for different categories
echo "CheckAll" > tools/system/.options.txt
echo "CheckAll" > tools/hardware/.options.txt
echo "CheckAll" > tools/drivers/.options.txt
echo "CheckAll" > tools/remote_access/.options.txt
echo "CheckAll" > tools/nirsoft/.options.txt
echo "CollapseTree" > pe_autorun/system_setup/.options.txt
echo "CheckAll" > scripts/basic/.options.txt
echo "CheckAll" > scripts/advanced/.options.txt

echo "  ✅ Created options files for GUI behavior"

echo ""
echo "📄 === CREATING DOCUMENTATION ==="

# Create documentation files
cat > tools/drivers/README.md << 'EOF'
# Driver Management Tools

## DriverStoreExplorer.exe
- **Purpose**: View and remove Windows driver packages
- **Usage**: Launch from tools menu to manage installed drivers
- **Features**: List, remove, and backup driver packages

## NVCleanstall.exe  
- **Purpose**: Customize NVIDIA driver installations
- **Usage**: Clean install NVIDIA drivers with custom options
- **Features**: Remove bloatware, select components, offline installs
EOF

cat > tools/remote_access/README.md << 'EOF'
# Remote Access Tools

## CKTech ScreenConnect
- **Files**: cktech_support.msi, cktech_rdesk.msi
- **Purpose**: Remote support via CK Technology ScreenConnect
- **Usage**: Install during Windows setup for remote assistance

## ResolveTech Support
- **File**: resolvetech_support.exe
- **Purpose**: Remote support via Resolve Technology
- **Usage**: Install for remote troubleshooting and support

## VNC Components
- **Files**: tvnserver.exe, screenhooks64.dll
- **Purpose**: Direct VNC access for remote control
- **Usage**: Manual VNC server setup when needed
EOF

cat > scripts/advanced/system/README.md << 'EOF'
# System Configuration Scripts

## Power Management
- **power_settings.bat**: Conservative power settings (from concept)
- **power_aggressive.bat**: Aggressive power settings (from exported)

## Windows Activation
- **windows_activation.bat**: KMS activation script
- **Note**: For educational/testing purposes only - ensure proper licensing

## System Optimization
- **debloat_system.bat**: Remove Windows bloatware and unnecessary features
EOF

echo "  ✅ Created documentation files"

echo ""
echo "🎯 === MIGRATION SUMMARY ==="

# Count migrated files
tool_count=$(find tools -type f | wc -l)
script_count=$(find scripts -type f | wc -l)
pe_count=$(find pe_autorun -type f | wc -l)
config_count=$(find config -type f | wc -l)

echo "📊 Migration Statistics:"
echo "  🔧 Tools migrated: $tool_count"
echo "  📝 Scripts migrated: $script_count"  
echo "  ⚡ PE AutoRun items: $pe_count"
echo "  ⚙️ Config files: $config_count"
echo ""
echo "✅ Migration completed successfully!"
echo ""
echo "🔍 Next steps:"
echo "  1. Run: ghostwin tools (to verify detection)"
echo "  2. Test build: ghostwin build --source-iso <iso> --output-dir build --output-iso test.iso"
echo "  3. Remove concept/ and exported/ directories when satisfied"
echo ""