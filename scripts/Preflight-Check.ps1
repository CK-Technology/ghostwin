# GhostWin Pre-flight Check
# Validates system is ready to build ISO

param(
    [switch]$Detailed
)

$ErrorActionPreference = "Continue"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "GhostWin Pre-flight Check" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$issues = @()
$warnings = @()

# Check 1: Administrator privileges
Write-Host "[1/8] Checking administrator privileges..." -NoNewline
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host " FAIL" -ForegroundColor Red
    $issues += "Administrator privileges required for DISM operations"
} else {
    Write-Host " OK" -ForegroundColor Green
}

# Check 2: Windows ADK
Write-Host "[2/8] Checking Windows ADK..." -NoNewline
$adkPaths = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit",
    "${env:ProgramFiles}\Windows Kits\10\Assessment and Deployment Kit"
)
$adkFound = $adkPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($adkFound) {
    Write-Host " OK" -ForegroundColor Green
    if ($Detailed) { Write-Host "   Found at: $adkFound" -ForegroundColor Gray }
} else {
    Write-Host " FAIL" -ForegroundColor Red
    $issues += "Windows ADK not found. Download from https://docs.microsoft.com/en-us/windows-hardware/get-started/adk-install"
}

# Check 3: WinPE Add-on
Write-Host "[3/8] Checking WinPE Add-on..." -NoNewline
$winpePaths = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment",
    "${env:ProgramFiles}\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment"
)
$winpeFound = $winpePaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($winpeFound) {
    Write-Host " OK" -ForegroundColor Green
} else {
    Write-Host " FAIL" -ForegroundColor Red
    $issues += "WinPE Add-on not found. Install alongside Windows ADK"
}

# Check 4: oscdimg.exe
Write-Host "[4/8] Checking oscdimg.exe..." -NoNewline
$oscdimgPaths = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe",
    "${env:ProgramFiles}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
)
$oscdimgFound = $oscdimgPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($oscdimgFound) {
    Write-Host " OK" -ForegroundColor Green
} else {
    Write-Host " FAIL" -ForegroundColor Red
    $issues += "oscdimg.exe not found (part of Windows ADK)"
}

# Check 5: 7-Zip
Write-Host "[5/8] Checking 7-Zip..." -NoNewline
$sevenZip = Get-Command "7z.exe" -ErrorAction SilentlyContinue
if ($sevenZip) {
    Write-Host " OK" -ForegroundColor Green
    if ($Detailed) { Write-Host "   Found at: $($sevenZip.Source)" -ForegroundColor Gray }
} else {
    Write-Host " FAIL" -ForegroundColor Red
    $issues += "7-Zip not found. Download from https://www.7-zip.org/"
}

# Check 6: Disk space
Write-Host "[6/8] Checking disk space..." -NoNewline
$drive = Get-PSDrive C
$freeGB = [math]::Round($drive.Free / 1GB, 2)
if ($freeGB -ge 20) {
    Write-Host " OK ($freeGB GB free)" -ForegroundColor Green
} elseif ($freeGB -ge 10) {
    Write-Host " WARNING ($freeGB GB free)" -ForegroundColor Yellow
    $warnings += "Low disk space: ${freeGB}GB free (20GB+ recommended)"
} else {
    Write-Host " FAIL ($freeGB GB free)" -ForegroundColor Red
    $issues += "Insufficient disk space: ${freeGB}GB free (minimum 10GB required)"
}

# Check 7: Critical drivers
Write-Host "[7/8] Checking for storage drivers..." -NoNewline
$driverDirs = @("pe_autorun\drivers", "tools\drivers", "drivers")
$criticalDrivers = @("*iastor*.inf", "*vmd*.inf", "*nvme*.inf")
$foundDrivers = @()

foreach ($dir in $driverDirs) {
    if (Test-Path $dir) {
        foreach ($pattern in $criticalDrivers) {
            $found = Get-ChildItem -Path $dir -Filter $pattern -Recurse -ErrorAction SilentlyContinue
            if ($found) {
                $foundDrivers += $found
            }
        }
    }
}

if ($foundDrivers.Count -gt 0) {
    Write-Host " OK ($($foundDrivers.Count) driver INF files found)" -ForegroundColor Green
    if ($Detailed) {
        foreach ($driver in $foundDrivers | Select-Object -First 5) {
            Write-Host "   - $($driver.Name)" -ForegroundColor Gray
        }
    }
} else {
    Write-Host " WARNING" -ForegroundColor Yellow
    $warnings += "No storage drivers found. NVMe drives may not be visible without drivers or VMD BIOS bypass"
}

# Check 8: GhostWin binary
Write-Host "[8/8] Checking GhostWin binary..." -NoNewline
$binaryPaths = @("target\release\ghostwin.exe", "target\debug\ghostwin.exe", "ghostwin.exe")
$binaryFound = $binaryPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($binaryFound) {
    Write-Host " OK" -ForegroundColor Green
    if ($Detailed) { Write-Host "   Found at: $binaryFound" -ForegroundColor Gray }
} else {
    Write-Host " WARNING" -ForegroundColor Yellow
    $warnings += "GhostWin binary not built. Run: cargo build --release"
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Pre-flight Check Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if ($issues.Count -eq 0 -and $warnings.Count -eq 0) {
    Write-Host ""
    Write-Host "✅ ALL CHECKS PASSED!" -ForegroundColor Green
    Write-Host "System is ready to build GhostWin ISO" -ForegroundColor Green
    Write-Host ""
    exit 0
} else {
    if ($issues.Count -gt 0) {
        Write-Host ""
        Write-Host "❌ CRITICAL ISSUES ($($issues.Count)):" -ForegroundColor Red
        foreach ($issue in $issues) {
            Write-Host "  - $issue" -ForegroundColor Red
        }
    }

    if ($warnings.Count -gt 0) {
        Write-Host ""
        Write-Host "⚠️  WARNINGS ($($warnings.Count)):" -ForegroundColor Yellow
        foreach ($warning in $warnings) {
            Write-Host "  - $warning" -ForegroundColor Yellow
        }
    }

    Write-Host ""
    if ($issues.Count -gt 0) {
        Write-Host "Fix critical issues before building ISO" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "System ready with warnings - proceed with caution" -ForegroundColor Yellow
        exit 0
    }
}
