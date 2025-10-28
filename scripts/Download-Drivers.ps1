# GhostWin Automatic Driver Downloader
# Downloads latest drivers for modern hardware (Intel 15th gen, Micron, Dell, Samsung)
# Saves to pe_autorun/drivers/ for ISO building

param(
    [switch]$IntelRST,
    [switch]$Samsung,
    [switch]$Micron,
    [switch]$Dell,
    [switch]$All,
    [string]$OutputPath = "pe_autorun\drivers",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# Color output functions
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-Error { Write-Host $args -ForegroundColor Red }

Write-Info "=========================================="
Write-Info "GhostWin Automatic Driver Downloader"
Write-Info "=========================================="
Write-Info ""

# Create output directory
if (-not (Test-Path $OutputPath)) {
    New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
    Write-Success "Created directory: $OutputPath"
}

# Driver download definitions
$DriverSources = @{
    IntelRST = @{
        Name = "Intel Rapid Storage Technology (15th Gen support)"
        Url = "https://downloadmirror.intel.com/823572/SetupRST.exe"
        FileName = "SetupRST.exe"
        OutputDir = "Intel_RST"
        Description = "Required for Intel VMD/RapidStorage (Dell Optiplex 15th gen)"
        CriticalFiles = @("iaStorAVC.inf", "iaStorAC.inf", "iaStorAV.inf")
    }

    SamsungNVMe = @{
        Name = "Samsung NVMe Driver"
        Url = "https://download.semiconductor.samsung.com/resources/software-resources/Samsung_NVM_Express_Driver_3.3.zip"
        FileName = "Samsung_NVMe_Driver.zip"
        OutputDir = "Samsung_NVMe"
        Description = "Samsung 980 PRO, 990 PRO, 970 EVO Plus support"
        CriticalFiles = @("*.inf")
    }

    # Note: Micron doesn't have a generic download URL, needs model-specific
    MicronGeneric = @{
        Name = "Micron Storage Executive (includes drivers)"
        Url = "https://www.micron.com/products/storage/ssd"
        FileName = "Micron_StorageExecutive.exe"
        OutputDir = "Micron_NVMe"
        Description = "Micron 2200/2300/3400 NVMe firmware and drivers"
        CriticalFiles = @("*.inf")
        RequiresManual = $true
    }
}

# Intel RST Driver Download
if ($IntelRST -or $All) {
    Write-Info ""
    Write-Info "=========================================="
    Write-Info "Downloading Intel RapidStorage Technology"
    Write-Info "=========================================="

    $driver = $DriverSources.IntelRST
    $driverPath = Join-Path $OutputPath $driver.OutputDir

    if ((Test-Path $driverPath) -and -not $Force) {
        Write-Warning "Intel RST drivers already exist at: $driverPath"
        Write-Warning "Use -Force to re-download"
    }
    else {
        Write-Info "Downloading: $($driver.Name)"
        Write-Info "URL: $($driver.Url)"

        try {
            $downloadPath = Join-Path $env:TEMP $driver.FileName

            Write-Info "Downloading to: $downloadPath"
            Invoke-WebRequest -Uri $driver.Url -OutFile $downloadPath -UseBasicParsing
            Write-Success "✅ Downloaded successfully"

            # Extract Intel RST installer
            Write-Info "Extracting Intel RST drivers..."

            # Run installer in silent mode to extract
            $extractPath = Join-Path $env:TEMP "IntelRST_Extract"
            if (Test-Path $extractPath) {
                Remove-Item -Path $extractPath -Recurse -Force
            }
            New-Item -ItemType Directory -Path $extractPath -Force | Out-Null

            # Try to extract using 7-Zip if available
            $sevenZip = Get-Command "7z.exe" -ErrorAction SilentlyContinue
            if ($sevenZip) {
                Write-Info "Extracting with 7-Zip..."
                & 7z.exe x $downloadPath -o"$extractPath" -y | Out-Null
            }
            else {
                Write-Warning "7-Zip not found, trying alternative extraction..."
                # Try running installer to temp location
                Start-Process $downloadPath -ArgumentList "/s /extractonly /targetdir=$extractPath" -Wait -NoNewWindow
            }

            # Find F6 driver folder
            $f6Folders = Get-ChildItem -Path $extractPath -Recurse -Directory | Where-Object {
                $_.Name -like "*F6*" -or $_.Name -like "*Driver*" -or $_.Name -like "*VMD*"
            }

            if ($f6Folders) {
                $sourceFolder = $f6Folders[0].FullName
                Write-Info "Found driver folder: $sourceFolder"

                # Copy to output directory
                if (Test-Path $driverPath) {
                    Remove-Item -Path $driverPath -Recurse -Force
                }

                Copy-Item -Path $sourceFolder -Destination $driverPath -Recurse -Force
                Write-Success "✅ Intel RST drivers extracted to: $driverPath"

                # Verify critical files
                foreach ($file in $driver.CriticalFiles) {
                    $found = Get-ChildItem -Path $driverPath -Filter $file -Recurse -ErrorAction SilentlyContinue
                    if ($found) {
                        Write-Success "  ✅ Found: $file"
                    }
                    else {
                        Write-Warning "  ⚠️ Missing: $file"
                    }
                }
            }
            else {
                Write-Warning "Could not locate driver files in extracted package"
                Write-Info "Please manually extract drivers from: $downloadPath"
            }

            # Cleanup
            Remove-Item -Path $downloadPath -Force -ErrorAction SilentlyContinue
            Remove-Item -Path $extractPath -Recurse -Force -ErrorAction SilentlyContinue
        }
        catch {
            Write-Error "❌ Failed to download Intel RST: $_"
        }
    }
}

# Samsung NVMe Driver Download
if ($Samsung -or $All) {
    Write-Info ""
    Write-Info "=========================================="
    Write-Info "Downloading Samsung NVMe Driver"
    Write-Info "=========================================="

    $driver = $DriverSources.SamsungNVMe
    $driverPath = Join-Path $OutputPath $driver.OutputDir

    if ((Test-Path $driverPath) -and -not $Force) {
        Write-Warning "Samsung drivers already exist at: $driverPath"
        Write-Warning "Use -Force to re-download"
    }
    else {
        Write-Info "Downloading: $($driver.Name)"
        Write-Info "Note: Samsung URL may change, checking..."

        try {
            $downloadPath = Join-Path $env:TEMP $driver.FileName

            Write-Info "Attempting download from Samsung..."
            Invoke-WebRequest -Uri $driver.Url -OutFile $downloadPath -UseBasicParsing
            Write-Success "✅ Downloaded successfully"

            # Extract ZIP
            Write-Info "Extracting Samsung drivers..."
            Expand-Archive -Path $downloadPath -DestinationPath $driverPath -Force
            Write-Success "✅ Samsung drivers extracted to: $driverPath"

            # Cleanup
            Remove-Item -Path $downloadPath -Force -ErrorAction SilentlyContinue
        }
        catch {
            Write-Warning "⚠️ Automatic download failed: $_"
            Write-Info "Please manually download from: https://semiconductor.samsung.com/consumer-storage/support/tools/"
            Write-Info "Extract to: $driverPath"
        }
    }
}

# Micron Driver Download (Manual)
if ($Micron -or $All) {
    Write-Info ""
    Write-Info "=========================================="
    Write-Info "Micron NVMe Drivers (Manual Download)"
    Write-Info "=========================================="

    $driver = $DriverSources.MicronGeneric
    $driverPath = Join-Path $OutputPath $driver.OutputDir

    Write-Warning "Micron drivers require manual download (model-specific)"
    Write-Info ""
    Write-Info "Steps to get Micron drivers:"
    Write-Info "1. Visit: https://www.micron.com/products/storage/ssd/client-ssd"
    Write-Info "2. Select your specific Micron model (2200/2300/3400/7450)"
    Write-Info "3. Download 'Firmware Update Tool' or 'Driver Package'"
    Write-Info "4. Extract files to: $driverPath"
    Write-Info ""
    Write-Info "Common Micron models:"
    Write-Info "  - Micron 2200 NVMe"
    Write-Info "  - Micron 2300 NVMe"
    Write-Info "  - Micron 3400 NVMe (latest for Dell 15th gen)"
    Write-Info "  - Micron 7450 PRO NVMe"
}

# Dell Driver Package Download
if ($Dell -or $All) {
    Write-Info ""
    Write-Info "=========================================="
    Write-Info "Dell Driver Package (Manual)"
    Write-Info "=========================================="

    Write-Warning "Dell drivers are system-specific and require your Service Tag"
    Write-Info ""
    Write-Info "To download Dell drivers:"
    Write-Info "1. Visit: https://www.dell.com/support/home/"
    Write-Info "2. Enter your Dell Service Tag or select your model"
    Write-Info "3. Download 'Chipset' and 'Storage Controller' drivers"
    Write-Info "4. Extract to: $OutputPath\Dell_Storage\"
    Write-Info ""
    Write-Info "For Dell Optiplex 15th Gen (2024+):"
    Write-Info "  - Look for 'Intel Chipset Driver'"
    Write-Info "  - Look for 'Intel Rapid Storage Technology'"
    Write-Info "  - May include specific NVMe firmware"
}

# Summary
Write-Info ""
Write-Info "=========================================="
Write-Info "Download Summary"
Write-Info "=========================================="

$downloadedCount = 0
$totalDrivers = 0

foreach ($key in $DriverSources.Keys) {
    $driver = $DriverSources[$key]
    $driverPath = Join-Path $OutputPath $driver.OutputDir
    $totalDrivers++

    if (Test-Path $driverPath) {
        $infCount = (Get-ChildItem -Path $driverPath -Filter "*.inf" -Recurse -ErrorAction SilentlyContinue).Count
        if ($infCount -gt 0) {
            Write-Success "✅ $($driver.Name): $infCount .inf files found"
            $downloadedCount++
        }
        else {
            Write-Warning "⚠️ $($driver.Name): Directory exists but no .inf files"
        }
    }
    else {
        Write-Warning "❌ $($driver.Name): Not downloaded"
    }
}

Write-Info ""
Write-Info "Status: $downloadedCount / $totalDrivers driver packages ready"
Write-Info "Output directory: $OutputPath"

# Next steps
Write-Info ""
Write-Info "=========================================="
Write-Info "Next Steps"
Write-Info "=========================================="
Write-Info "1. Verify drivers are extracted to $OutputPath"
Write-Info "2. For manual downloads (Micron/Dell), follow instructions above"
Write-Info "3. Build your ISO with: ghostwin build --source-iso Windows11.iso"
Write-Info "4. Drivers will be automatically loaded in WinPE!"
Write-Info ""
Write-Info "To verify driver files:"
Write-Info "  Get-ChildItem -Path '$OutputPath' -Filter '*.inf' -Recurse | Select-Object Name, Directory"
Write-Info ""
Write-Success "Driver download complete!"
