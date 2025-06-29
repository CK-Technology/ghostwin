# GhostWin One-Line Installer for Windows
# Usage: iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

param(
    [switch]$SkipRust,
    [switch]$SkipBuild,
    [switch]$PreBuilt,
    [switch]$FixCargo,
    [switch]$SkipEnvConfig,
    [string]$InstallPath = "C:\ProgramData\CKTech\GhostWin",
    [switch]$Help
)

if ($Help) {
    Write-Host @"
GhostWin Installer

This installer automatically handles all dependencies including:
- Visual Studio Build Tools (for Windows compilation)
- Rust toolchain (if compiling from source)
- Windows ADK and PE add-on (via winget or manual download)

Usage:
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex                # Full install with dependency checks
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -PreBuilt      # Download pre-built binaries (faster)
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -SkipRust      # Skip Rust install
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -InstallPath "C:\Tools\GhostWin"

Options:
  -PreBuilt        Download pre-built binaries instead of compiling from source
  -SkipRust        Skip Rust installation (if already installed)
  -SkipBuild       Skip the build process (download source only)
  -FixCargo        Reset Cargo configuration to fix index/network issues
  -SkipEnvConfig   Skip automatic Rust/Cargo environment optimization
  -InstallPath     Custom installation directory
  -Help            Show this help

Dependencies handled automatically:
- Uses winget for Windows ADK/PE installation (with manual fallback)
- Checks for Visual Studio Build Tools
- Installs Rust if needed (for source compilation)
"@
    exit 0
}

$ErrorActionPreference = "Stop"

# Handle FixCargo option first
if ($FixCargo) {
    Fix-CargoIssues
    exit 0
}

Write-Host "*** GhostWin Installation Script ***" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

# Check if running as administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "WARNING: This script requires Administrator privileges for optimal setup." -ForegroundColor Yellow
    Write-Host "         Some features may not work without admin rights." -ForegroundColor Yellow
    Write-Host ""
}

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Function to fix common Cargo issues
function Fix-CargoIssues {
    Write-Host "FIXING: Attempting to fix common Cargo issues..." -ForegroundColor Yellow
    
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
    
    try {
        # 1. Clear potentially corrupted registry index
        $registryPath = "$cargoHome\registry"
        if (Test-Path $registryPath) {
            Write-Host "   Clearing Cargo registry cache..." -ForegroundColor Gray
            Remove-Item $registryPath -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        # 2. Create or update Cargo config with better network settings
        if (-not (Test-Path $cargoHome)) {
            New-Item -ItemType Directory -Path $cargoHome -Force | Out-Null
        }
        
        $configFile = "$cargoHome\config.toml"
        $configContent = @"
[net]
retry = 3
offline = false

[http]
timeout = 300
low-speed-limit = 10
multiplexing = false

[build]
jobs = 1  # Reduce parallel jobs to avoid overwhelming slow connections
"@
        
        Write-Host "   Creating optimized Cargo configuration..." -ForegroundColor Gray
        Set-Content -Path $configFile -Value $configContent -Force
        
        Write-Host "SUCCESS: Cargo configuration reset complete!" -ForegroundColor Green
        Write-Host "   Try running the installer again without -FixCargo" -ForegroundColor Gray
        
    } catch {
        Write-Host "WARNING: Some Cargo fixes failed: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "   Manual fix: Delete '$cargoHome' and reinstall Rust" -ForegroundColor Gray
    }
}

# Function to configure optimal Rust/Cargo environment for Windows
function Configure-RustEnvironment {
    Write-Host "CONFIGURING: Setting up optimal Rust/Cargo environment for Windows..." -ForegroundColor Yellow
    
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
    
    try {
        # 1. Ensure .cargo directory exists
        if (-not (Test-Path $cargoHome)) {
            New-Item -ItemType Directory -Path $cargoHome -Force | Out-Null
        }
        
        # 2. Create comprehensive Cargo configuration
        $configFile = "$cargoHome\config.toml"
        
        # Remove any existing config file first to ensure clean state
        if (Test-Path $configFile) {
            Write-Host "   Removing existing Cargo config to ensure clean state..." -ForegroundColor Gray
            Remove-Item $configFile -Force -ErrorAction SilentlyContinue
        }
        
        $configContent = @"
# Optimized Cargo configuration for Windows builds
[net]
retry = 5                    # Retry failed network operations up to 5 times
offline = false              # Allow network access

[http]
timeout = 600                # 10 minute timeout for large downloads
low-speed-limit = 1024       # Minimum 1KB/s (fail if slower)
multiplexing = false         # Disable HTTP/2 multiplexing (avoids some Windows issues)

[build]
jobs = 2                     # Limit parallel compilation jobs (reduces memory usage)
target-dir = "target"        # Use relative target directory
incremental = true           # Enable incremental compilation for faster rebuilds

[profile.release]
opt-level = 2                # Optimize for speed but not at expense of compile time
lto = "thin"                 # Thin link-time optimization (faster than "fat")
codegen-units = 4            # Balance between compile time and runtime performance
panic = "abort"              # Smaller binaries, faster startup

[profile.dev]
opt-level = 0                # No optimization for dev builds (faster compilation)
debug = true                 # Include debug info
split-debuginfo = "packed"   # Keep debug info in single file on Windows

[registries.crates-io]
protocol = "sparse"          # Use sparse registry protocol (faster, less bandwidth)

[term]
verbose = false              # Reduce noise in output
color = "auto"               # Auto-detect color support
"@
        
        Write-Host "   Creating optimized Cargo configuration..." -ForegroundColor Gray
        Set-Content -Path $configFile -Value $configContent -Force
        
        # 3. Set up environment variables for this session
        Write-Host "   Configuring environment variables..." -ForegroundColor Gray
        
        # Cargo environment variables
        $env:CARGO_NET_RETRY = "5"
        $env:CARGO_HTTP_TIMEOUT = "600"
        $env:CARGO_HTTP_LOW_SPEED_LIMIT = "1024"
        $env:CARGO_HTTP_MULTIPLEXING = "false"
        $env:CARGO_INCREMENTAL = "1"
        $env:CARGO_TARGET_DIR = "target"
        
        # Rust compilation variables
        $env:RUSTFLAGS = "-C target-cpu=native"              # Optimize for current CPU
        $env:RUST_BACKTRACE = "1"                            # Enable backtraces for debugging
        
        # Windows-specific variables
        $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "link.exe"  # Explicit linker
        
        # Memory and performance settings
        $logicalCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
        $optimalJobs = [Math]::Max(1, [Math]::Min($logicalCores, 4))  # Cap at 4 for stability
        $env:CARGO_BUILD_JOBS = $optimalJobs.ToString()
        
        Write-Host "   Configured for $optimalJobs parallel build jobs" -ForegroundColor Gray
        
        # 5. Configure Windows Defender exclusions (if running as admin)
        if (([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
            Write-Host "   Configuring Windows Defender exclusions..." -ForegroundColor Gray
            
            try {
                # Add Cargo and target directories to exclusions
                Add-MpPreference -ExclusionPath $cargoHome -ErrorAction SilentlyContinue
                Add-MpPreference -ExclusionPath "$env:USERPROFILE\.rustup" -ErrorAction SilentlyContinue
                Add-MpPreference -ExclusionProcess "cargo.exe" -ErrorAction SilentlyContinue
                Add-MpPreference -ExclusionProcess "rustc.exe" -ErrorAction SilentlyContinue
                Add-MpPreference -ExclusionProcess "link.exe" -ErrorAction SilentlyContinue
                
                Write-Host "   Windows Defender exclusions added for Rust tools" -ForegroundColor Gray
            } catch {
                Write-Host "   Could not configure Windows Defender exclusions (this is optional)" -ForegroundColor Gray
            }
        }
        
        # 6. Verify Rust toolchain and update if needed
        Write-Host "   Verifying Rust toolchain..." -ForegroundColor Gray
        
        if (Test-Command "rustup") {
            # Update to latest stable
            & rustup update stable 2>$null | Out-Null
            
            # Ensure MSVC target is installed
            & rustup target add x86_64-pc-windows-msvc 2>$null | Out-Null
            
            # Add useful components
            & rustup component add clippy 2>$null | Out-Null    # Linter
            & rustup component add rustfmt 2>$null | Out-Null   # Formatter
        }
        
        # 7. Pre-warm Cargo cache with common dependencies
        Write-Host "   Pre-warming Cargo cache with common dependencies..." -ForegroundColor Gray
        
        $tempProject = "$env:TEMP\cargo-warmup-$(Get-Random)"
        try {
            # Create a temporary project with common dependencies
            New-Item -ItemType Directory -Path $tempProject -Force | Out-Null
            Push-Location $tempProject
            
            $cargoToml = @"
[package]
name = "cargo-warmup"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }
"@
            Set-Content -Path "Cargo.toml" -Value $cargoToml
            
            # This will download and cache common dependencies
            & cargo check --quiet 2>$null | Out-Null
            
        } catch {
            # Ignore errors in pre-warming
        } finally {
            Pop-Location
            Remove-Item $tempProject -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        Write-Host "SUCCESS: Rust/Cargo environment configured for optimal Windows performance!" -ForegroundColor Green
        Write-Host "   Configuration includes:" -ForegroundColor Gray
        Write-Host "   - Network retry and timeout settings" -ForegroundColor Gray
        Write-Host "   - Optimized compilation profiles" -ForegroundColor Gray
        Write-Host "   - Windows Defender exclusions (if admin)" -ForegroundColor Gray
        Write-Host "   - Memory and CPU optimization" -ForegroundColor Gray
        
    } catch {
        Write-Host "WARNING: Some environment configuration failed: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "   Build should still work but may be slower" -ForegroundColor Gray
    }
}

# Check for Visual Studio Build Tools (required for Windows builds)
Write-Host "Checking for Visual Studio Build Tools..." -ForegroundColor Yellow

$vsBuildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasVSBuildTools = $false

if (Test-Path $vsBuildToolsPath) {
    try {
        $vsInstallations = & $vsBuildToolsPath -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json 2>$null | ConvertFrom-Json
        if ($vsInstallations -and $vsInstallations.Count -gt 0) {
            $hasVSBuildTools = $true
            Write-Host "SUCCESS: Visual Studio Build Tools with C++ support found!" -ForegroundColor Green
        }
    } catch {
        Write-Host "   Failed to query Visual Studio installations: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

if (-not $hasVSBuildTools) {
    Write-Host "WARNING: Visual Studio Build Tools with C++ support not found!" -ForegroundColor Red
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
            Write-Host "DOWNLOADING: Downloading Visual Studio Build Tools..." -ForegroundColor Yellow
            
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
                    Write-Host "SUCCESS: Visual Studio Build Tools installed successfully!" -ForegroundColor Green
                    Remove-Item $buildToolsPath -Force -ErrorAction SilentlyContinue
                } elseif ($process.ExitCode -eq 3010) {
                    Write-Host "SUCCESS: Visual Studio Build Tools installed successfully!" -ForegroundColor Green
                    Write-Host "   Note: A reboot may be required for full functionality." -ForegroundColor Yellow
                    Remove-Item $buildToolsPath -Force -ErrorAction SilentlyContinue
                } else {
                    Write-Host "WARNING:  Build Tools installation may have had issues (exit code: $($process.ExitCode))" -ForegroundColor Yellow
                    Write-Host "   Continuing anyway - Rust installation will verify if tools are working." -ForegroundColor Gray
                }
            } catch {
                Write-Host "ERROR: Failed to download/install Build Tools: $($_.Exception.Message)" -ForegroundColor Red
                Write-Host ""
                Write-Host "Options:" -ForegroundColor Yellow
                Write-Host "  1. Use pre-built GhostWin binaries (no compilation needed)" -ForegroundColor Green
                Write-Host "  2. Install Build Tools manually and continue with source build" -ForegroundColor Gray
                Write-Host "  3. Exit and try again later" -ForegroundColor Gray
                Write-Host ""
                
                $fallbackChoice = Read-Host "Choose option (1-3)"
                switch ($fallbackChoice) {
                    "1" {
                        Write-Host "DOWNLOADING: Switching to pre-built binary installation..." -ForegroundColor Cyan
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
            Write-Host "DOWNLOADING: Switching to pre-built binary installation..." -ForegroundColor Cyan
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
Write-Host "CHECKING: Checking for Windows PE and ADK..." -ForegroundColor Yellow

$adkPath = "${env:ProgramFiles(x86)}\Windows Kits\10"
$peAddonPath = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment"

$hasADK = Test-Path $adkPath
$hasPEAddon = Test-Path $peAddonPath

# Function to install via winget with fallback
function Install-ADKComponents {
    $hasWinget = Test-Command "winget"
    
    if ($hasWinget) {
        Write-Host "USING: Using winget for installation (recommended method)..." -ForegroundColor Green
        
        try {
            if (-not $hasADK) {
                Write-Host "DOWNLOADING: Installing Windows ADK via winget..." -ForegroundColor Yellow
                winget install -e --id Microsoft.WindowsADK --silent --accept-package-agreements --accept-source-agreements
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "SUCCESS: Windows ADK installed successfully!" -ForegroundColor Green
                    $hasADK = $true
                } else {
                    throw "Winget installation failed"
                }
            }
            
            if (-not $hasPEAddon) {
                Write-Host "DOWNLOADING: Installing Windows PE add-on via winget..." -ForegroundColor Yellow
                winget install -e --id Microsoft.ADKPEAddon --silent --accept-package-agreements --accept-source-agreements
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "SUCCESS: Windows PE add-on installed successfully!" -ForegroundColor Green
                    $hasPEAddon = $true
                } else {
                    throw "Winget installation failed"
                }
            }
            
            return $true
        }
        catch {
            Write-Host "WARNING:  Winget installation failed. Falling back to manual download..." -ForegroundColor Yellow
            return $false
        }
    } else {
        Write-Host "WARNING:  Winget not available. Using manual download method..." -ForegroundColor Yellow
        return $false
    }
}

# Function for manual download fallback
function Install-ADKManual {
    Write-Host "DOWNLOADING: Manual installation method..." -ForegroundColor Yellow
    
    if (-not $hasADK) {
        Write-Host "   Opening ADK download: https://go.microsoft.com/fwlink/?linkid=2289980" -ForegroundColor Gray
        Start-Process "https://go.microsoft.com/fwlink/?linkid=2289980"
    }
    
    if (-not $hasPEAddon) {
        Write-Host "   Opening PE add-on download: https://go.microsoft.com/fwlink/?linkid=2289981" -ForegroundColor Gray
        Start-Process "https://go.microsoft.com/fwlink/?linkid=2289981"
    }
    
    Write-Host ""
    Write-Host "CHECKING: Installation Instructions:" -ForegroundColor Cyan
    Write-Host "   1. Install Windows ADK first (if needed)" -ForegroundColor Gray
    Write-Host "   2. Then install Windows PE add-on (if needed)" -ForegroundColor Gray
    Write-Host "   3. Both installers are now downloading to your Downloads folder" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   Press Enter after installing required components..." -ForegroundColor Yellow
    Read-Host
}

if ($hasADK -and $hasPEAddon) {
    Write-Host "SUCCESS: Windows ADK and PE add-on are already installed!" -ForegroundColor Green
} elseif ($hasADK -and -not $hasPEAddon) {
    Write-Host "SUCCESS: Windows ADK found at: $adkPath" -ForegroundColor Green
    Write-Host "WARNING:  Windows PE add-on not found!" -ForegroundColor Yellow
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
    Write-Host "WARNING:  Windows ADK not found!" -ForegroundColor Yellow
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
        Write-Host "SKIPPING:  Skipping ADK setup. You can install later using:" -ForegroundColor Yellow
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
    Write-Host "VERIFYING: Verifying installation..." -ForegroundColor Yellow
    
    # Refresh paths
    $hasADK = Test-Path $adkPath
    $hasPEAddon = Test-Path $peAddonPath
    
    if ($hasADK) {
        Write-Host "SUCCESS: Windows ADK verified!" -ForegroundColor Green
    } else {
        Write-Host "WARNING:  Windows ADK not detected. Please ensure it's installed correctly." -ForegroundColor Yellow
    }
    
    if ($hasPEAddon) {
        Write-Host "SUCCESS: Windows PE add-on verified!" -ForegroundColor Green
    } else {
        Write-Host "WARNING:  Windows PE add-on not detected. Please ensure it's installed correctly." -ForegroundColor Yellow
    }
}

# Install Rust if not present
if (-not $SkipRust) {
    Write-Host "CHECKING: Checking for Rust installation..." -ForegroundColor Yellow
    
    if (Test-Command "cargo") {
        Write-Host "SUCCESS: Rust is already installed!" -ForegroundColor Green
        cargo --version
    } else {
        Write-Host "DOWNLOADING: Installing Rust..." -ForegroundColor Yellow
        
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
            Write-Host "SUCCESS: Rust installed successfully!" -ForegroundColor Green
        } else {
            Write-Host "ERROR: Rust installation failed. Please install manually from https://rustup.rs/" -ForegroundColor Red
            exit 1
        }
    }
    
    # Configure Rust environment for optimal Windows builds (unless skipped)
    if (-not $SkipEnvConfig) {
        Configure-RustEnvironment
    } else {
        Write-Host "SKIPPING: Rust environment configuration (as requested)" -ForegroundColor Yellow
    }
}

# Create installation directory
Write-Host "CREATING: Creating installation directory: $InstallPath" -ForegroundColor Yellow
if (Test-Path $InstallPath) {
    Write-Host "   Directory already exists, cleaning..." -ForegroundColor Gray
    Remove-Item "$InstallPath\*" -Recurse -Force -ErrorAction SilentlyContinue
} else {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

if ($PreBuilt) {
    Write-Host "DOWNLOADING: Downloading pre-built GhostWin binaries..." -ForegroundColor Yellow
    
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
            
            Write-Host "SUCCESS: Pre-built binaries downloaded successfully!" -ForegroundColor Green
        } else {
            Write-Host "WARNING:  No pre-built Windows binaries found in latest release." -ForegroundColor Yellow
            Write-Host "   Falling back to source compilation..." -ForegroundColor Gray
            $PreBuilt = $false
        }
    } catch {
        Write-Host "WARNING:  Failed to download pre-built binaries: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "   Falling back to source compilation..." -ForegroundColor Gray
        $PreBuilt = $false
    }
}

if (-not $PreBuilt) {
    # Clone or download GhostWin source
    Write-Host "Downloading GhostWin source..." -ForegroundColor Yellow

    # Download ZIP archive (most reliable method for Windows)
    Write-Host "   Downloading ZIP archive from GitHub..." -ForegroundColor Gray
    $zipUrl = "https://github.com/CK-Technology/ghostwin/archive/main.zip"
    $zipPath = "$env:TEMP\ghostwin.zip"
    
    try {
        # Clean up any existing temp files
        if (Test-Path $zipPath) {
            Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path "$env:TEMP\ghostwin-main") {
            Remove-Item "$env:TEMP\ghostwin-main" -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $env:TEMP -Force
        Move-Item "$env:TEMP\ghostwin-main\*" $InstallPath -Force
        Remove-Item "$env:TEMP\ghostwin-main" -Recurse -Force
        Remove-Item $zipPath -Force
        Write-Host "   SUCCESS: Download completed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "   ERROR: Download failed: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "   Please check your internet connection and try again." -ForegroundColor Yellow
        exit 1
    }
}

# Build GhostWin (if not using pre-built and not skipping build)
if (-not $PreBuilt -and -not $SkipBuild) {
    Write-Host "BUILDING: Building GhostWin with optimized environment..." -ForegroundColor Yellow
    Push-Location $InstallPath

    try {
        Write-Host "   Using pre-configured Rust/Cargo environment for optimal performance" -ForegroundColor Gray
        Write-Host "   Starting build process (this may take several minutes)..." -ForegroundColor Gray
        Write-Host "   Tip: Environment is optimized to prevent 'Updating crates.io index' hangs" -ForegroundColor Gray
        
        # Run the build with comprehensive error capture
        $buildAttempts = 0
        $maxAttempts = 3
        $buildSuccess = $false
        
        do {
            $buildAttempts++
            if ($buildAttempts -gt 1) {
                Write-Host "   Build attempt $buildAttempts of $maxAttempts..." -ForegroundColor Yellow
                # Clear any partial build artifacts on retry
                if (Test-Path "target") {
                    Write-Host "   Cleaning previous build artifacts..." -ForegroundColor Gray
                    & cargo clean 2>$null | Out-Null
                }
            }
            
            Write-Host "   Running: cargo build --release" -ForegroundColor Gray
            $buildResult = & cargo build --release 2>&1
            $buildOutput = $buildResult -join "`n"
            
            if ($LASTEXITCODE -eq 0 -and (Test-Path "target\release\ghostwin.exe")) {
                $buildSuccess = $true
                Write-Host "SUCCESS: GhostWin built successfully!" -ForegroundColor Green
                
                # Verify the executable
                $exeSize = (Get-Item "target\release\ghostwin.exe").Length
                Write-Host "   Executable size: $([math]::Round($exeSize/1MB, 2)) MB" -ForegroundColor Gray
                break
            } else {
                Write-Host "ERROR: Build attempt $buildAttempts failed!" -ForegroundColor Red
                
                # Analyze the error and provide specific guidance
                if ($buildOutput -match "Updating crates.io index") {
                    Write-Host "   Issue: Cargo index update hanging (despite optimizations)" -ForegroundColor Yellow
                    Write-Host "   This indicates a severe network or corporate firewall issue" -ForegroundColor Gray
                } elseif ($buildOutput -match "failed to get") {
                    Write-Host "   Issue: Network connectivity problem" -ForegroundColor Yellow
                } elseif ($buildOutput -match "linker.*not found") {
                    Write-Host "   Issue: Missing Visual Studio Build Tools or linker" -ForegroundColor Yellow
                } elseif ($buildOutput -match "permission denied" -or $buildOutput -match "access is denied") {
                    Write-Host "   Issue: Permission denied - antivirus interference" -ForegroundColor Yellow
                } elseif ($buildOutput -match "out of memory") {
                    Write-Host "   Issue: Insufficient memory for compilation" -ForegroundColor Yellow
                }
                
                if ($buildAttempts -lt $maxAttempts) {
                    Write-Host "   Waiting 10 seconds before retry..." -ForegroundColor Gray
                    Start-Sleep 10
                }
            }
        } while ($buildAttempts -lt $maxAttempts -and -not $buildSuccess)
        
        if (-not $buildSuccess) {
            Write-Host ""
            Write-Host "Build output (last 25 lines):" -ForegroundColor Yellow
            ($buildResult | Select-Object -Last 25) | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
            
            Write-Host ""
            Write-Host "SOLUTIONS: Try one of these approaches:" -ForegroundColor Yellow
            Write-Host "  1. Run: install.ps1 -PreBuilt    (download pre-built binaries instead)" -ForegroundColor White
            Write-Host "  2. Run: install.ps1 -FixCargo    (reset Cargo configuration and try again)" -ForegroundColor White
            Write-Host "  3. Check internet connection and disable antivirus temporarily" -ForegroundColor Gray
            Write-Host "  4. Ensure you have at least 4GB RAM and 3GB free disk space" -ForegroundColor Gray
            Write-Host "  5. Run PowerShell as Administrator" -ForegroundColor Gray
            Write-Host "  6. Try building manually: cd '$InstallPath' && cargo build --release" -ForegroundColor Gray
            Write-Host ""
            Write-Host "If problems persist, use the -PreBuilt option for a quick install:" -ForegroundColor Green
            Write-Host "  PowerShell: iwr -useb <script-url> | iex -PreBuilt" -ForegroundColor White
            Write-Host ""
            Write-Host "For detailed troubleshooting, see: https://github.com/CK-Technology/ghostwin/blob/main/TROUBLESHOOTING.md" -ForegroundColor Cyan
            
            exit 1
        }
    } catch {
        Write-Host "ERROR: Build process failed with exception: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host ""
        Write-Host "Try using pre-built binaries instead:" -ForegroundColor Green
        Write-Host "  PowerShell: install.ps1 -PreBuilt" -ForegroundColor White
        exit 1
    } finally {
        Pop-Location
    }
} elseif ($SkipBuild) {
    Write-Host "SKIPPING:  Skipping build process as requested." -ForegroundColor Yellow
} else {
    Write-Host "SUCCESS: Using pre-built binaries, skipping compilation." -ForegroundColor Green
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
    Write-Host "WARNING:  GhostWin executable not found after installation!" -ForegroundColor Yellow
    Write-Host "   Please check the installation manually." -ForegroundColor Gray
} else {
    # Add to PATH (optional)
    $addToPath = Read-Host "Add GhostWin to PATH? (y/N)"
    if ($addToPath -eq "y" -or $addToPath -eq "Y") {
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($currentPath -notlike "*$executableDir*") {
            $newPath = $currentPath + ";" + $executableDir
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            Write-Host "Added to PATH. Restart your terminal to use 'ghostwin' command." -ForegroundColor Green
        } else {
            Write-Host "Already in PATH." -ForegroundColor Green
        }
    }

    # Validate installation
    Write-Host "VERIFYING: Validating installation..." -ForegroundColor Yellow
    try {
        & $executablePath validate
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Installation validation passed!" -ForegroundColor Green
        } else {
            Write-Host "Installation validation had warnings" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "Could not validate installation: $($_.Exception.Message)" -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "*** GhostWin Installation Complete! ***" -ForegroundColor Green
    Write-Host "=======================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Installation Details:" -ForegroundColor Cyan
    Write-Host "  Location: $InstallPath" -ForegroundColor Gray
    Write-Host "  Executable: $executablePath" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Quick Start Commands:" -ForegroundColor Yellow
    Write-Host "  Launch GUI:      `"$executablePath`" gui" -ForegroundColor White
    Write-Host "  Build ISO:       `"$executablePath`" build --source-iso Windows11.iso" -ForegroundColor White
    Write-Host "  Show Help:       `"$executablePath`" --help" -ForegroundColor White
    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. Place your Windows ISO in the GhostWin directory" -ForegroundColor Gray
    Write-Host "  2. Launch the GUI to configure your deployment" -ForegroundColor Gray
    Write-Host "  3. Build your custom WinPE image" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Documentation:" -ForegroundColor Cyan
    Write-Host "  Setup Guide:     https://github.com/CK-Technology/ghostwin/blob/main/GUNPOWDER.md" -ForegroundColor Gray
    Write-Host "  Troubleshooting: https://github.com/CK-Technology/ghostwin/blob/main/TROUBLESHOOTING.md" -ForegroundColor Gray
    Write-Host "  Full Docs:       https://github.com/CK-Technology/ghostwin/blob/main/README.md" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Happy deploying! " -ForegroundColor Green
}
