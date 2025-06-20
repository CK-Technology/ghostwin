# GhostWin Simple Installer for Windows
# Usage: iwr -useb <url>/install-simple.ps1 | iex

param(
    [switch]$PreBuilt,
    [string]$InstallPath = "C:\ProgramData\CKTech\GhostWin",
    [switch]$Help
)

if ($Help) {
    Write-Host "GhostWin Simple Installer"
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  iwr -useb <url>/install-simple.ps1 | iex"
    Write-Host "  iwr -useb <url>/install-simple.ps1 | iex -PreBuilt"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -PreBuilt     Download pre-built binaries instead of compiling"
    Write-Host "  -InstallPath  Custom installation directory"
    Write-Host "  -Help         Show this help"
    exit 0
}

$ErrorActionPreference = "Stop"

Write-Host "=== GhostWin Simple Installer ===" -ForegroundColor Cyan
Write-Host ""

# Simple function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Check for Visual Studio Build Tools
Write-Host "Checking Visual Studio Build Tools..." -ForegroundColor Yellow
$vsBuildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasVSBuildTools = $false

if (Test-Path $vsBuildToolsPath) {
    try {
        $vsInstallations = & $vsBuildToolsPath -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json 2>$null | ConvertFrom-Json
        if ($vsInstallations -and $vsInstallations.Count -gt 0) {
            $hasVSBuildTools = $true
            Write-Host "SUCCESS: Visual Studio Build Tools found!" -ForegroundColor Green
        }
    } catch {
        Write-Host "Could not verify Build Tools" -ForegroundColor Yellow
    }
}

if (-not $hasVSBuildTools) {
    Write-Host "WARNING: Visual Studio Build Tools not found!" -ForegroundColor Yellow
    Write-Host "You may need to install them manually for compilation." -ForegroundColor Gray
}

# Check for Rust
if (-not $PreBuilt) {
    Write-Host "Checking Rust installation..." -ForegroundColor Yellow
    
    if (Test-Command "cargo") {
        Write-Host "SUCCESS: Rust is installed!" -ForegroundColor Green
        cargo --version
    } else {
        Write-Host "Rust not found. Installing..." -ForegroundColor Yellow
        
        # Download and run rustup-init
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupPath = "$env:TEMP\rustup-init.exe"
        
        Write-Host "Downloading Rust installer..." -ForegroundColor Gray
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        
        Write-Host "Running Rust installer..." -ForegroundColor Gray
        & $rustupPath -y --default-toolchain stable --default-host x86_64-pc-windows-msvc
        
        # Refresh environment
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        
        if (Test-Command "cargo") {
            Write-Host "SUCCESS: Rust installed!" -ForegroundColor Green
        } else {
            Write-Host "ERROR: Rust installation failed" -ForegroundColor Red
            Write-Host "Switching to pre-built binaries..." -ForegroundColor Yellow
            $PreBuilt = $true
        }
    }
}

# Create installation directory
Write-Host "Creating installation directory..." -ForegroundColor Yellow
if (Test-Path $InstallPath) {
    Remove-Item "$InstallPath\*" -Recurse -Force -ErrorAction SilentlyContinue
} else {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

if ($PreBuilt) {
    Write-Host "Downloading pre-built binaries..." -ForegroundColor Yellow
    
    try {
        $releasesUrl = "https://api.github.com/repos/CK-Technology/ghostwin/releases/latest"
        $releaseInfo = Invoke-RestMethod -Uri $releasesUrl -UseBasicParsing
        
        $windowsAsset = $releaseInfo.assets | Where-Object { $_.name -like "*windows*" -or $_.name -like "*win64*" -or $_.name -like "*.exe" } | Select-Object -First 1
        
        if ($windowsAsset) {
            Write-Host "Downloading $($windowsAsset.name)..." -ForegroundColor Gray
            $assetPath = Join-Path $InstallPath $windowsAsset.name
            Invoke-WebRequest -Uri $windowsAsset.browser_download_url -OutFile $assetPath -UseBasicParsing
            
            if ($windowsAsset.name -like "*.zip") {
                Expand-Archive -Path $assetPath -DestinationPath $InstallPath -Force
                Remove-Item $assetPath -Force
            }
            
            Write-Host "SUCCESS: Pre-built binaries downloaded!" -ForegroundColor Green
        } else {
            Write-Host "No pre-built Windows binaries found" -ForegroundColor Yellow
            $PreBuilt = $false
        }
    } catch {
        Write-Host "Failed to download pre-built binaries: $($_.Exception.Message)" -ForegroundColor Yellow
        $PreBuilt = $false
    }
}

if (-not $PreBuilt) {
    # Download source and build
    Write-Host "Downloading source code..." -ForegroundColor Yellow
    
    $zipUrl = "https://github.com/CK-Technology/ghostwin/archive/main.zip"
    $zipPath = "$env:TEMP\ghostwin.zip"
    
    try {
        if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
        if (Test-Path "$env:TEMP\ghostwin-main") { Remove-Item "$env:TEMP\ghostwin-main" -Recurse -Force }
        
        Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $env:TEMP -Force
        Move-Item "$env:TEMP\ghostwin-main\*" $InstallPath -Force
        Remove-Item "$env:TEMP\ghostwin-main" -Recurse -Force
        Remove-Item $zipPath -Force
        
        Write-Host "Source downloaded successfully!" -ForegroundColor Green
    } catch {
        Write-Host "ERROR: Source download failed: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
    
    # Build
    Write-Host "Building GhostWin..." -ForegroundColor Yellow
    Push-Location $InstallPath
    
    try {
        Write-Host "Running: cargo build --release" -ForegroundColor Gray
        $buildResult = & cargo build --release 2>&1
        
        if ($LASTEXITCODE -eq 0 -and (Test-Path "target\release\ghostwin.exe")) {
            Write-Host "SUCCESS: Build completed!" -ForegroundColor Green
        } else {
            Write-Host "ERROR: Build failed!" -ForegroundColor Red
            Write-Host "Build output:" -ForegroundColor Yellow
            $buildResult | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
            exit 1
        }
    } catch {
        Write-Host "ERROR: Build process failed: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    } finally {
        Pop-Location
    }
}

# Find executable
$executablePath = ""
if (Test-Path "$InstallPath\target\release\ghostwin.exe") {
    $executablePath = "$InstallPath\target\release\ghostwin.exe"
} elseif (Test-Path "$InstallPath\ghostwin.exe") {
    $executablePath = "$InstallPath\ghostwin.exe"
} else {
    $exeFiles = Get-ChildItem -Path $InstallPath -Filter "*.exe" -Recurse | Where-Object { $_.Name -like "*ghostwin*" } | Select-Object -First 1
    if ($exeFiles) {
        $executablePath = $exeFiles.FullName
    }
}

if ($executablePath -and (Test-Path $executablePath)) {
    Write-Host ""
    Write-Host "=== Installation Complete! ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "Installation Details:" -ForegroundColor Cyan
    Write-Host "  Location: $InstallPath" -ForegroundColor Gray
    Write-Host "  Executable: $executablePath" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Quick Start Commands:" -ForegroundColor Yellow
    Write-Host "  Launch GUI:      `"$executablePath`" gui" -ForegroundColor White
    Write-Host "  Show Help:       `"$executablePath`" --help" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "WARNING: GhostWin executable not found!" -ForegroundColor Yellow
    Write-Host "Installation may have failed." -ForegroundColor Gray
}
