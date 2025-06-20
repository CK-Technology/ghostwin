# GhostWin One-Line Installer for Windows
# Usage: iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

param(
    [switch]$SkipRust,
    [string]$InstallPath = "C:\ProgramData\CKTech\GhostWin",
    [switch]$Help
)

if ($Help) {
    Write-Host @"
GhostWin Installer

Usage:
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex                # Full install
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -SkipRust      # Skip Rust install
  iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -InstallPath "C:\ProgramData\CKTech\GhostWin"

Options:
  -SkipRust      Skip Rust installation (if already installed)
  -InstallPath   Custom installation directory
  -Help          Show this help
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

# Clone or download GhostWin
Write-Host "⬇️  Downloading GhostWin..." -ForegroundColor Yellow

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

# Build GhostWin
Write-Host "🔨 Building GhostWin..." -ForegroundColor Yellow
Push-Location $InstallPath

try {
    Write-Host "   Running cargo build --release (this may take several minutes)..." -ForegroundColor Gray
    cargo build --release
    
    if (Test-Path "target\release\ghostwin.exe") {
        Write-Host "✅ GhostWin built successfully!" -ForegroundColor Green
    } else {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        exit 1
    }
} finally {
    Pop-Location
}

# Add to PATH (optional)
$addToPath = Read-Host "Add GhostWin to PATH? (y/N)"
if ($addToPath -eq "y" -or $addToPath -eq "Y") {
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $newPath = $currentPath + ";" + "$InstallPath\target\release"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "✅ Added to PATH. Restart your terminal to use 'ghostwin' command." -ForegroundColor Green
}

# Validate installation
Write-Host "🔍 Validating installation..." -ForegroundColor Yellow
& "$InstallPath\target\release\ghostwin.exe" validate

Write-Host ""
Write-Host "🎉 GhostWin Installation Complete!" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green
Write-Host ""
Write-Host "Location: $InstallPath" -ForegroundColor Cyan
Write-Host "Executable: $InstallPath\target\release\ghostwin.exe" -ForegroundColor Cyan
Write-Host ""
Write-Host "Quick Start:" -ForegroundColor Yellow
Write-Host "  cd `"$InstallPath`"" -ForegroundColor Gray
Write-Host "  .\target\release\ghostwin.exe gui" -ForegroundColor Gray
Write-Host ""
Write-Host "For help: .\target\release\ghostwin.exe --help" -ForegroundColor Gray
Write-Host ""
Write-Host "Ready to deploy! 🚀" -ForegroundColor Green
