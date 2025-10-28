# GhostWin Installer Build Script
# Automates the process of building the installer package
# Requires: Rust, InnoSetup 6

param(
    [string]$Version = "0.3.3",
    [string]$Configuration = "release",
    [switch]$SkipBuild,
    [switch]$SkipTests,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-Error { Write-Host $args -ForegroundColor Red }

Write-Info "======================================"
Write-Info "GhostWin Installer Build Script v$Version"
Write-Info "======================================"
Write-Info ""

# Check prerequisites
Write-Info "Checking prerequisites..."

# Check Rust installation
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "❌ Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
}
Write-Success "✅ Rust/Cargo found: $(cargo --version)"

# Check InnoSetup installation
$InnoSetupPaths = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
)

$InnoSetup = $null
foreach ($path in $InnoSetupPaths) {
    if (Test-Path $path) {
        $InnoSetup = $path
        break
    }
}

if (-not $InnoSetup) {
    Write-Error "❌ InnoSetup 6 not found. Please install from https://jrsoftware.org/isinfo.php"
    exit 1
}
Write-Success "✅ InnoSetup found: $InnoSetup"

# Create necessary directories
Write-Info ""
Write-Info "Creating necessary directories..."
$dirs = @("dist", "dependencies", "assets\icons")
foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Success "✅ Created directory: $dir"
    }
}

# Download Visual C++ Redistributables if not present
Write-Info ""
Write-Info "Checking for VC++ Redistributables..."
$vcRedistPath = "dependencies\vcredist_x64.exe"
if (-not (Test-Path $vcRedistPath)) {
    Write-Info "Downloading Visual C++ Redistributables 2022..."
    $vcRedistUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"

    try {
        Invoke-WebRequest -Uri $vcRedistUrl -OutFile $vcRedistPath -UseBasicParsing
        Write-Success "✅ Downloaded VC++ Redistributables"
    }
    catch {
        Write-Warning "⚠️ Failed to download VC++ Redistributables: $_"
        Write-Warning "The installer will require manual download of vc_redist.x64.exe"
    }
}
else {
    Write-Success "✅ VC++ Redistributables already present"
}

# Build the Rust project
if (-not $SkipBuild) {
    Write-Info ""
    Write-Info "Building Rust project ($Configuration)..."

    $buildArgs = @("build")
    if ($Configuration -eq "release") {
        $buildArgs += "--release"
    }
    if ($Verbose) {
        $buildArgs += "--verbose"
    }

    try {
        $buildOutput = & cargo @buildArgs 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "❌ Build failed!"
            Write-Host $buildOutput
            exit $LASTEXITCODE
        }
        Write-Success "✅ Build completed successfully"
    }
    catch {
        Write-Error "❌ Build failed: $_"
        exit 1
    }
}
else {
    Write-Warning "⚠️ Skipping build (--SkipBuild specified)"
}

# Run tests
if (-not $SkipTests) {
    Write-Info ""
    Write-Info "Running tests..."

    try {
        $testOutput = & cargo test 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "⚠️ Tests failed, but continuing..."
            Write-Host $testOutput
        }
        else {
            Write-Success "✅ All tests passed"
        }
    }
    catch {
        Write-Warning "⚠️ Test execution failed: $_"
    }
}
else {
    Write-Warning "⚠️ Skipping tests (--SkipTests specified)"
}

# Verify binary exists
Write-Info ""
Write-Info "Verifying binary..."
$binaryPath = "target\$Configuration\ghostwin.exe"
if (-not (Test-Path $binaryPath)) {
    Write-Error "❌ Binary not found at: $binaryPath"
    exit 1
}

$binarySize = (Get-Item $binaryPath).Length / 1MB
Write-Success "✅ Binary found: $binaryPath ($([math]::Round($binarySize, 2)) MB)"

# Update version in InnoSetup script
Write-Info ""
Write-Info "Updating version in installer script..."
$issPath = "installer.iss"
if (Test-Path $issPath) {
    $issContent = Get-Content $issPath -Raw
    $issContent = $issContent -replace '#define MyAppVersion ".*"', "#define MyAppVersion ""$Version"""
    Set-Content -Path $issPath -Value $issContent
    Write-Success "✅ Updated version to $Version"
}
else {
    Write-Warning "⚠️ installer.iss not found, skipping version update"
}

# Build the installer
Write-Info ""
Write-Info "Building installer with InnoSetup..."

try {
    $isccOutput = & $InnoSetup $issPath 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "❌ InnoSetup compilation failed!"
        Write-Host $isccOutput
        exit $LASTEXITCODE
    }

    Write-Success "✅ Installer compiled successfully"
}
catch {
    Write-Error "❌ InnoSetup compilation failed: $_"
    exit 1
}

# Verify installer was created
$installerPath = "dist\GhostWin-Setup-v$Version.exe"
if (Test-Path $installerPath) {
    $installerSize = (Get-Item $installerPath).Length / 1MB
    Write-Success ""
    Write-Success "======================================"
    Write-Success "✅ SUCCESS!"
    Write-Success "======================================"
    Write-Success "Installer: $installerPath"
    Write-Success "Size: $([math]::Round($installerSize, 2)) MB"
    Write-Success ""

    # Calculate SHA256 hash for verification
    Write-Info "Calculating SHA256 hash..."
    $hash = (Get-FileHash -Path $installerPath -Algorithm SHA256).Hash
    Write-Info "SHA256: $hash"

    # Save hash to file
    $hashFile = "$installerPath.sha256"
    Set-Content -Path $hashFile -Value $hash
    Write-Success "✅ Hash saved to: $hashFile"
}
else {
    Write-Error "❌ Installer not found at expected location: $installerPath"
    exit 1
}

Write-Info ""
Write-Info "Build complete! You can now distribute the installer."
Write-Info ""
Write-Info "Next steps:"
Write-Info "  1. Test the installer on a clean Windows VM"
Write-Info "  2. Upload to GitHub Releases"
Write-Info "  3. Update documentation with new version"
