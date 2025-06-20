# GhostWin One-Line Installer for Windows
# Usage: iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

param(
    [switch]$SkipRust,
    [switch]$SkipBuild,
    [switch]$PreBuilt,
    [string]$InstallPath = "C:\ProgramData\CKTech\GhostWin",
    [switch]$Help
)

if ($Help) {
    Write-Host @"
GhostWin Installer

This installer automatically handles all dependencies including:
• Visual Studio Build Tools (for Windows compilation)
• Rust toolchain (if compiling from source)
• Windows ADK and PE add-on (via winget or manual download)

Usage:
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex                # Full install with dependency checks
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -PreBuilt      # Download pre-built binaries (faster)
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -SkipRust      # Skip Rust install
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -InstallPath "C:\Tools\GhostWin"

Options:
  -PreBuilt      Download pre-built binaries instead of compiling from source
  -SkipRust      Skip Rust installation (if already installed)
  -SkipBuild     Skip the build process (download source only)
  -InstallPath   Custom installation directory
  -Help          Show this help

Dependencies handled automatically:
• Uses winget for Windows ADK/PE installation (with manual fallback)
• Checks for Visual Studio Build Tools
• Installs Rust if needed (for source compilation)
"@
    exit 0
}

$ErrorActionPreference = "Stop"

Write-Host "🚀 GhostWin Installation Script" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan

# Check if running as administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "⚠️  This script requires Administrator privileges for optimal setup." -ForegroundColor Yellow
    Write-Host "   Some features may not work without admin rights." -ForegroundColor Yellow
    Write-Host ""
}

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Check for Visual Studio Build Tools (required for Windows builds)
Write-Host "🔧 Checking for Visual Studio Build Tools..." -ForegroundColor Yellow

$vsBuildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasVSBuildTools = $false

if (Test-Path $vsBuildToolsPath) {
    try {
        $vsInstallations = & $vsBuildToolsPath -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json 2>$null | ConvertFrom-Json
        if ($vsInstallations -and $vsInstallations.Count -gt 0) {
            $hasVSBuildTools = $true
            Write-Host "✅ Visual Studio Build Tools with C++ support found!" -ForegroundColor Green
        }
    } catch {
        Write-Host "   Failed to query Visual Studio installations: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

if (-not $hasVSBuildTools) {
    Write-Host "⚠️  Visual Studio Build Tools with C++ support not found!" -ForegroundColor Red
    Write-Host "   This is required for building Rust applications on Windows." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  1. Auto-install Build Tools (Recommended for development)" -ForegroundColor Green
    Write-Host "  2. Download pre-built GhostWin binaries (Faster, no build required)" -ForegroundColor Cyan
    Write-Host "  3. Install Build Tools manually" -ForegroundColor Gray
    Write-Host ""
    
    $installChoice = Read-Host "Choose option (1-3)"
    switch ($installChoice) {
        "1" {
            Write-Host "📦 Downloading Visual Studio Build Tools..." -ForegroundColor Yellow
            
            $buildToolsUrl = "https://aka.ms/vs/17/release/vs_buildtools.exe"
            $buildToolsPath = "$env:TEMP\vs_buildtools.exe"
            
            try {
                Write-Host "   Downloading installer..." -ForegroundColor Gray
                Invoke-WebRequest -Uri $buildToolsUrl -OutFile $buildToolsPath -UseBasicParsing
                
                Write-Host "   Installing Visual Studio Build Tools with C++ workload..." -ForegroundColor Gray
                Write-Host "   This will take 5-15 minutes depending on your internet connection." -ForegroundColor Gray
                
                $installArgs = @(
                    "--quiet"
                    "--wait" 
                    "--add", "Microsoft.VisualStudio.Workload.VCTools"
                    "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
                    "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22621"
                )
                
                $process = Start-Process -FilePath $buildToolsPath -ArgumentList $installArgs -Wait -PassThru
                
                if ($process.ExitCode -eq 0) {
                    Write-Host "✅ Visual Studio Build Tools installed successfully!" -ForegroundColor Green
                    Remove-Item $buildToolsPath -Force -ErrorAction SilentlyContinue
                } else {
                    Write-Host "⚠️  Build Tools installation may have had issues (exit code: $($process.ExitCode))" -ForegroundColor Yellow
                    Write-Host "   Continuing anyway - Rust installation will verify if tools are working." -ForegroundColor Gray
                }
            } catch {
                Write-Host "❌ Failed to download/install Build Tools: $($_.Exception.Message)" -ForegroundColor Red
                Write-Host ""
                Write-Host "Options:" -ForegroundColor Yellow
                Write-Host "  1. Use pre-built GhostWin binaries (no compilation needed)" -ForegroundColor Green
                Write-Host "  2. Install Build Tools manually and continue with source build" -ForegroundColor Gray
                Write-Host "  3. Exit and try again later" -ForegroundColor Gray
                Write-Host ""
                
                $fallbackChoice = Read-Host "Choose option (1-3)"
                switch ($fallbackChoice) {
                    "1" {
                        Write-Host "📦 Switching to pre-built binary installation..." -ForegroundColor Cyan
                        $PreBuilt = $true
                    }
                    "2" {
                        Write-Host "Please install Build Tools manually from:" -ForegroundColor Yellow
                        Write-Host "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Gray
                        Write-Host "Make sure to include the 'C++ build tools' workload!" -ForegroundColor Red
                        
                        $continueManual = Read-Host "Continue installation anyway? (y/N)"
                        if ($continueManual -ne "y" -and $continueManual -ne "Y") {
                            exit 1
                        }
                    }
                    default {
                        Write-Host "Exiting installation. Please try again later." -ForegroundColor Yellow
                        exit 1
                    }
                }
            }
        }
        "2" {
            Write-Host "📦 Switching to pre-built binary installation..." -ForegroundColor Cyan
            $PreBuilt = $true
        }
        "3" {
            Write-Host "Please install Visual Studio Build Tools manually, then re-run this script." -ForegroundColor Yellow
            Write-Host "Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Gray
            Write-Host "Make sure to include the 'C++ build tools' workload!" -ForegroundColor Red
            exit 1
        }
        default {
            Write-Host "Invalid choice. Defaulting to pre-built binaries..." -ForegroundColor Yellow
            $PreBuilt = $true
        }
    }
}

# Check for Windows PE and ADK (Windows 11 24H2)
Write-Host "🔧 Checking for Windows PE and ADK..." -ForegroundColor Yellow

$adkPath = "${env:ProgramFiles(x86)}\Windows Kits\10"
$peAddonPath = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment"

$hasADK = Test-Path $adkPath
$hasPEAddon = Test-Path $peAddonPath

# Function to install via winget with fallback
function Install-ADKComponents {
    $hasWinget = Test-Command "winget"
    
    if ($hasWinget) {
        Write-Host "🎯 Using winget for installation (recommended method)..." -ForegroundColor Green
        
        try {
            if (-not $hasADK) {
                Write-Host "📦 Installing Windows ADK via winget..." -ForegroundColor Yellow
                winget install -e --id Microsoft.WindowsADK --silent --accept-package-agreements --accept-source-agreements
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "✅ Windows ADK installed successfully!" -ForegroundColor Green
                    $hasADK = $true
                } else {
                    throw "Winget installation failed"
                }
            }
            
            if (-not $hasPEAddon) {
                Write-Host "📦 Installing Windows PE add-on via winget..." -ForegroundColor Yellow
                winget install -e --id Microsoft.ADKPEAddon --silent --accept-package-agreements --accept-source-agreements
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "✅ Windows PE add-on installed successfully!" -ForegroundColor Green
                    $hasPEAddon = $true
                } else {
                    throw "Winget installation failed"
                }
            }
            
            return $true
        }
        catch {
            Write-Host "⚠️  Winget installation failed. Falling back to manual download..." -ForegroundColor Yellow
            return $false
        }
    } else {
        Write-Host "⚠️  Winget not available. Using manual download method..." -ForegroundColor Yellow
        return $false
    }
}

# Function for manual download fallback
function Install-ADKManual {
    Write-Host "📦 Manual installation method..." -ForegroundColor Yellow
    
    if (-not $hasADK) {
        Write-Host "   Opening ADK download: https://go.microsoft.com/fwlink/?linkid=2289980" -ForegroundColor Gray
        Start-Process "https://go.microsoft.com/fwlink/?linkid=2289980"
    }
    
    if (-not $hasPEAddon) {
        Write-Host "   Opening PE add-on download: https://go.microsoft.com/fwlink/?linkid=2289981" -ForegroundColor Gray
        Start-Process "https://go.microsoft.com/fwlink/?linkid=2289981"
    }
    
    Write-Host ""
    Write-Host "🔧 Installation Instructions:" -ForegroundColor Cyan
    Write-Host "   1. Install Windows ADK first (if needed)" -ForegroundColor Gray
    Write-Host "   2. Then install Windows PE add-on (if needed)" -ForegroundColor Gray
    Write-Host "   3. Both installers are now downloading to your Downloads folder" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   Press Enter after installing required components..." -ForegroundColor Yellow
    Read-Host
}

if ($hasADK -and $hasPEAddon) {
    Write-Host "✅ Windows ADK and PE add-on are already installed!" -ForegroundColor Green
} elseif ($hasADK -and -not $hasPEAddon) {
    Write-Host "✅ Windows ADK found at: $adkPath" -ForegroundColor Green
    Write-Host "⚠️  Windows PE add-on not found!" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  1. Auto-install Windows PE add-on (Recommended)" -ForegroundColor Green
    Write-Host "  2. Skip ADK setup (can install later)" -ForegroundColor Gray
    Write-Host ""
    
    $peChoice = Read-Host "Choose option (1-2)"
    if ($peChoice -eq "1") {
        $success = Install-ADKComponents
        if (-not $success) {
            Install-ADKManual
        }
    }
} else {
    Write-Host "⚠️  Windows ADK not found!" -ForegroundColor Yellow
    Write-Host "   For Windows 11 24H2 deployment, you'll need both ADK and PE add-on." -ForegroundColor Gray
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  1. Auto-install Windows ADK and PE add-on (Recommended)" -ForegroundColor Green
    Write-Host "  2. Skip ADK setup (can install later)" -ForegroundColor Gray
    Write-Host ""
    
    $adkChoice = Read-Host "Choose option (1-2)"
    if ($adkChoice -eq "1") {
        $success = Install-ADKComponents
        if (-not $success) {
            Install-ADKManual
        }
    } else {
        Write-Host "⏭️  Skipping ADK setup. You can install later using:" -ForegroundColor Yellow
        Write-Host "   winget install -e --id Microsoft.WindowsADK" -ForegroundColor Gray
        Write-Host "   winget install -e --id Microsoft.ADKPEAddon" -ForegroundColor Gray
        Write-Host "   Or download manually:" -ForegroundColor Gray
        Write-Host "   ADK: https://go.microsoft.com/fwlink/?linkid=2289980" -ForegroundColor Gray
        Write-Host "   PE add-on: https://go.microsoft.com/fwlink/?linkid=2289981" -ForegroundColor Gray
    }
}

# Final verification
if ($adkChoice -eq "1" -or $peChoice -eq "1") {
    Write-Host ""
    Write-Host "🔍 Verifying installation..." -ForegroundColor Yellow
    
    # Refresh paths
    $hasADK = Test-Path $adkPath
    $hasPEAddon = Test-Path $peAddonPath
    
    if ($hasADK) {
        Write-Host "✅ Windows ADK verified!" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Windows ADK not detected. Please ensure it's installed correctly." -ForegroundColor Yellow
    }
    
    if ($hasPEAddon) {
        Write-Host "✅ Windows PE add-on verified!" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Windows PE add-on not detected. Please ensure it's installed correctly." -ForegroundColor Yellow
    }
}

# Install Rust if not present
if (-not $SkipRust) {
    Write-Host "🔧 Checking for Rust installation..." -ForegroundColor Yellow
    
    if (Test-Command "cargo") {
        Write-Host "✅ Rust is already installed!" -ForegroundColor Green
        cargo --version
    } else {
        Write-Host "📦 Installing Rust..." -ForegroundColor Yellow
        
        # Download and run rustup-init
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupPath = "$env:TEMP\rustup-init.exe"
        
        Write-Host "   Downloading rustup-init.exe..." -ForegroundColor Gray
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        
        Write-Host "   Running Rust installer (this may take a few minutes)..." -ForegroundColor Gray
        & $rustupPath -y --default-toolchain stable --default-host x86_64-pc-windows-msvc
        
        # Refresh environment
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        
        if (Test-Command "cargo") {
            Write-Host "✅ Rust installed successfully!" -ForegroundColor Green
        } else {
            Write-Host "❌ Rust installation failed. Please install manually from https://rustup.rs/" -ForegroundColor Red
            exit 1
        }
    }
}

# Create installation directory
Write-Host "📁 Creating installation directory: $InstallPath" -ForegroundColor Yellow
New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null

if ($PreBuilt) {
    Write-Host "⬇️  Downloading pre-built GhostWin binaries..." -ForegroundColor Yellow
    
    try {
        # Download the latest release
        $releasesUrl = "https://api.github.com/repos/CK-Technology/ghostwin/releases/latest"
        $releaseInfo = Invoke-RestMethod -Uri $releasesUrl -UseBasicParsing
        
        $windowsAsset = $releaseInfo.assets | Where-Object { $_.name -like "*windows*" -or $_.name -like "*win64*" -or $_.name -like "*.exe" } | Select-Object -First 1
        
        if ($windowsAsset) {
            Write-Host "   Downloading $($windowsAsset.name)..." -ForegroundColor Gray
            $assetPath = Join-Path $InstallPath $windowsAsset.name
            Invoke-WebRequest -Uri $windowsAsset.browser_download_url -OutFile $assetPath -UseBasicParsing
            
            # If it's a zip file, extract it
            if ($windowsAsset.name -like "*.zip") {
                Expand-Archive -Path $assetPath -DestinationPath $InstallPath -Force
                Remove-Item $assetPath -Force
            }
            
            Write-Host "✅ Pre-built binaries downloaded successfully!" -ForegroundColor Green
        } else {
            Write-Host "⚠️  No pre-built Windows binaries found in latest release." -ForegroundColor Yellow
            Write-Host "   Falling back to source compilation..." -ForegroundColor Gray
            $PreBuilt = $false
        }
    } catch {
        Write-Host "⚠️  Failed to download pre-built binaries: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "   Falling back to source compilation..." -ForegroundColor Gray
        $PreBuilt = $false
    }
}

if (-not $PreBuilt) {
    # Clone or download GhostWin source
    Write-Host "⬇️  Downloading GhostWin source..." -ForegroundColor Yellow

# Check if git is available
if (Test-Command "git") {
    Write-Host "   Using git to clone repository..." -ForegroundColor Gray
    $parentPath = Split-Path $InstallPath -Parent
    git clone https://github.com/CK-Technology/ghostwin.git $InstallPath
} else {
    Write-Host "   Git not found - downloading ZIP archive (this is normal)..." -ForegroundColor Gray
    $zipUrl = "https://github.com/CK-Technology/ghostwin/archive/main.zip"
    $zipPath = "$env:TEMP\ghostwin.zip"
    
    try {
        Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $env:TEMP -Force
        Move-Item "$env:TEMP\ghostwin-main\*" $InstallPath -Force
        Remove-Item "$env:TEMP\ghostwin-main" -Recurse -Force
        Remove-Item $zipPath -Force
        Write-Host "   ✅ Download completed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "   ❌ Download failed: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "   Please check your internet connection and try again." -ForegroundColor Yellow
        exit 1
    }
}

# Build GhostWin (if not using pre-built and not skipping build)
if (-not $PreBuilt -and -not $SkipBuild) {
    Write-Host "🔨 Building GhostWin..." -ForegroundColor Yellow
    Push-Location $InstallPath

    try {
        Write-Host "   Running cargo build --release (this may take several minutes)..." -ForegroundColor Gray
        $buildResult = cargo build --release 2>&1
        
        if ($LASTEXITCODE -eq 0 -and (Test-Path "target\release\ghostwin.exe")) {
            Write-Host "✅ GhostWin built successfully!" -ForegroundColor Green
        } else {
            Write-Host "❌ Build failed!" -ForegroundColor Red
            Write-Host "Build output:" -ForegroundColor Yellow
            Write-Host $buildResult -ForegroundColor Gray
            exit 1
        }
    } catch {
        Write-Host "❌ Build process failed: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    } finally {
        Pop-Location
    }
} elseif ($SkipBuild) {
    Write-Host "⏭️  Skipping build process as requested." -ForegroundColor Yellow
} else {
    Write-Host "✅ Using pre-built binaries, skipping compilation." -ForegroundColor Green
}

# Find the executable path
$executablePath = ""
if (Test-Path "$InstallPath\target\release\ghostwin.exe") {
    $executablePath = "$InstallPath\target\release\ghostwin.exe"
    $executableDir = "$InstallPath\target\release"
} elseif (Test-Path "$InstallPath\ghostwin.exe") {
    $executablePath = "$InstallPath\ghostwin.exe"
    $executableDir = $InstallPath
} else {
    # Look for any .exe file in the install directory
    $exeFiles = Get-ChildItem -Path $InstallPath -Filter "*.exe" -Recurse | Where-Object { $_.Name -like "*ghostwin*" } | Select-Object -First 1
    if ($exeFiles) {
        $executablePath = $exeFiles.FullName
        $executableDir = $exeFiles.Directory.FullName
    }
}

if (-not $executablePath -or -not (Test-Path $executablePath)) {
    Write-Host "⚠️  GhostWin executable not found after installation!" -ForegroundColor Yellow
    Write-Host "   Please check the installation manually." -ForegroundColor Gray
} else {
    # Add to PATH (optional)
    $addToPath = Read-Host "Add GhostWin to PATH? (y/N)"
    if ($addToPath -eq "y" -or $addToPath -eq "Y") {
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($currentPath -notlike "*$executableDir*") {
            $newPath = $currentPath + ";" + $executableDir
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            Write-Host "✅ Added to PATH. Restart your terminal to use 'ghostwin' command." -ForegroundColor Green
        } else {
            Write-Host "✅ Already in PATH." -ForegroundColor Green
        }
    }

    # Validate installation
    Write-Host "🔍 Validating installation..." -ForegroundColor Yellow
    try {
        & $executablePath validate
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Installation validation passed!" -ForegroundColor Green
        } else {
            Write-Host "⚠️ Installation validation had warnings" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "⚠️ Could not validate installation: $($_.Exception.Message)" -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "🎉 GhostWin Installation Complete!" -ForegroundColor Green
    Write-Host "=================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Location: $InstallPath" -ForegroundColor Cyan
    Write-Host "Executable: $executablePath" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Quick Start:" -ForegroundColor Yellow
    Write-Host "  cd `"$InstallPath`"" -ForegroundColor Gray
    Write-Host "  `"$executablePath`" gui" -ForegroundColor Gray
    Write-Host ""
    Write-Host "For help: `"$executablePath`" --help" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Ready to deploy! 🚀" -ForegroundColor Green
}
