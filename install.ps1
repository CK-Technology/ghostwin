# GhostWin Windows Installer
# One-line usage:
#   irm https://win.cktech.sh | iex
# Fallback raw URL:
#   irm https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

param(
    [switch]$SkipRust,
    [switch]$SkipBuild,
    [switch]$SkipADK,
    [switch]$SkipBuildTools,
    [switch]$AddToPath,
    [switch]$NonInteractive,
    [string]$InstallPath = "C:\ProgramData\CKTech\GhostWin",
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$AdkPackageId = "Microsoft.WindowsADK"
$AdkPackageVersion = "10.1.26100.2454"
$WinPeAddonPackageId = "Microsoft.WindowsADK.WinPEAddon"
$AdkDownloadUrl = "https://go.microsoft.com/fwlink/?linkid=2289980"
$WinPeAddonDownloadUrl = "https://go.microsoft.com/fwlink/?linkid=2289981"
$AdkInstallRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\Assessment and Deployment Kit"
$DeploymentToolsRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\Assessment and Deployment Kit\Deployment Tools"
$WinPeAmd64Root = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment\amd64"

function Pause-IfInteractive([string]$Message = "Press Enter to exit") {
    if (-not $NonInteractive) {
        [void](Read-Host $Message)
    }
}

function Write-Step($Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Write-Info($Message) {
    Write-Host "  $Message" -ForegroundColor Gray
}

function Write-Warn($Message) {
    Write-Host "  WARNING: $Message" -ForegroundColor Yellow
}

function Write-Fail($Message) {
    Write-Host "  ERROR: $Message" -ForegroundColor Red
}

function Test-Admin {
    return ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator
    )
}

function Test-WindowsHost {
    return $env:OS -eq "Windows_NT"
}

function Test-CommandExists([string]$Name) {
    return [bool](Get-Command -Name $Name -ErrorAction SilentlyContinue)
}

function Refresh-Path {
    $machinePath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = "$machinePath;$userPath"
}

function Confirm-Choice {
    param(
        [string]$Prompt,
        [bool]$Default = $false
    )

    if ($NonInteractive) {
        return $Default
    }

    $suffix = if ($Default) { "[Y/n]" } else { "[y/N]" }
    $response = Read-Host "$Prompt $suffix"
    if ([string]::IsNullOrWhiteSpace($response)) {
        return $Default
    }

    return $response -match '^(y|yes)$'
}

function Invoke-DownloadFile {
    param(
        [Parameter(Mandatory = $true)][string]$Url,
        [Parameter(Mandatory = $true)][string]$Destination
    )

    Write-Info "Downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
}

function Get-AdkOscdimgPath {
    return Join-Path $DeploymentToolsRoot "amd64\Oscdimg\oscdimg.exe"
}

function Get-WinPeWimPath {
    return Join-Path $WinPeAmd64Root "en-us\winpe.wim"
}

function Test-AdkDeploymentToolsInstalled {
    return (Test-Path (Get-AdkOscdimgPath))
}

function Test-WinPeAddonInstalled {
    return (Test-Path (Get-WinPeWimPath))
}

function Invoke-InstallerProcess {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string]$ArgumentList,
        [Parameter(Mandatory = $true)][string]$DisplayName
    )

    Write-Info "Installing $DisplayName"
    $process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -Wait -PassThru
    if (@(0, 3010) -notcontains $process.ExitCode) {
        throw "$DisplayName installer exited with code $($process.ExitCode)"
    }

    if ($process.ExitCode -eq 3010) {
        Write-Warn "$DisplayName installed, but Windows reported that a reboot may be required"
    }
}

function Get-SourceArchiveUrl {
    return "https://github.com/CK-Technology/ghostwin/archive/refs/heads/main.zip"
}

function Get-VsWherePath {
    return Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
}

function Test-BuildToolsInstalled {
    $vsWhere = Get-VsWherePath
    if (-not (Test-Path $vsWhere)) {
        return $false
    }

    try {
        $installations = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json 2>$null | ConvertFrom-Json
        return ($installations -and $installations.Count -gt 0)
    } catch {
        Write-Warn "Failed to inspect Visual Studio Build Tools: $($_.Exception.Message)"
        return $false
    }
}

function Install-BuildTools {
    Write-Step "Installing Visual Studio Build Tools"

    $installerPath = Join-Path $env:TEMP "vs_buildtools.exe"
    Invoke-DownloadFile -Url "https://aka.ms/vs/17/release/vs_buildtools.exe" -Destination $installerPath

    $arguments = @(
        "--quiet"
        "--wait"
        "--add", "Microsoft.VisualStudio.Workload.VCTools"
        "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
        "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22621"
    )

    $process = Start-Process -FilePath $installerPath -ArgumentList $arguments -Wait -PassThru
    Remove-Item $installerPath -Force -ErrorAction SilentlyContinue

    if (@(0, 3010) -notcontains $process.ExitCode) {
        throw "Build Tools installer exited with code $($process.ExitCode)"
    }

    if ($process.ExitCode -eq 3010) {
        Write-Warn "Build Tools installed, but Windows reported that a reboot may be required"
    }
}

function Ensure-BuildTools {
    if ($SkipBuildTools) {
        Write-Warn "Skipping Visual Studio Build Tools setup"
        return
    }

    Write-Step "Checking Visual Studio Build Tools"
    if (Test-BuildToolsInstalled) {
        Write-Info "Visual Studio Build Tools with C++ workload found"
        return
    }

    Write-Warn "Visual Studio Build Tools with C++ workload not found"
    if (-not (Confirm-Choice -Prompt "Install Build Tools now?" -Default (-not $NonInteractive))) {
        throw "Build Tools are required for source installation. Install Build Tools manually or re-run with -SkipBuild."
    }

    Install-BuildTools
    if (-not (Test-BuildToolsInstalled)) {
        throw "Build Tools were not detected after installation"
    }
}

function Reset-InstallerState {
    Write-Info "Preparing Windows Installer service..."

    # Kill any hung msiexec or previous ADK installer processes
    Get-Process -Name msiexec -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Get-Process -Name adksetup -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Get-Process -Name adkwinpesetup -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    # Restart Windows Installer service
    Stop-Service msiserver -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
    Set-Service msiserver -StartupType Manual -ErrorAction SilentlyContinue
    Start-Service msiserver -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    # Clean up temp installer files from previous attempts
    Remove-Item "$env:TEMP\adksetup*" -Force -ErrorAction SilentlyContinue
    Remove-Item "$env:TEMP\adkwinpe*" -Force -ErrorAction SilentlyContinue
    Remove-Item "$env:TEMP\adk*.log" -Force -ErrorAction SilentlyContinue
}

function Remove-ExistingADK {
    param(
        [string]$AdkInstallerPath,
        [string]$WinPeInstallerPath
    )

    Write-Info "Removing any existing ADK/WinPE installations..."

    # Use the installers themselves to uninstall - they know how to find existing installations
    # This is more reliable than registry detection

    if (Test-Path $WinPeInstallerPath) {
        Write-Info "Running WinPE add-on uninstaller..."
        $peProc = Start-Process -FilePath $WinPeInstallerPath -ArgumentList "/uninstall /quiet /norestart" -Wait -PassThru
        Write-Info "WinPE uninstall exit code: $($peProc.ExitCode)"
        Start-Sleep -Seconds 5
    }

    if (Test-Path $AdkInstallerPath) {
        Write-Info "Running ADK uninstaller..."
        $adkProc = Start-Process -FilePath $AdkInstallerPath -ArgumentList "/uninstall /quiet /norestart" -Wait -PassThru
        Write-Info "ADK uninstall exit code: $($adkProc.ExitCode)"
        Start-Sleep -Seconds 5
    }

    # Also try to clean up the install directories if they exist
    $adkPath = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\Assessment and Deployment Kit"
    if (Test-Path $adkPath) {
        Write-Info "Cleaning up ADK directory..."
        Remove-Item $adkPath -Recurse -Force -ErrorAction SilentlyContinue
    }

    Reset-InstallerState
    Write-Info "Cleanup complete"
}

function Install-ADKDirect {
    Write-Step "Installing Windows ADK 25H2 components"

    # Reset installer state to avoid error 1000
    Reset-InstallerState

    $adkInstaller = Join-Path $env:TEMP "adksetup.exe"
    $winPeInstaller = Join-Path $env:TEMP "adkwinpesetup.exe"
    $adkLog = Join-Path $env:TEMP "adk_install.log"
    $winPeLog = Join-Path $env:TEMP "adkwinpe_install.log"

    try {
        # Download installers first
        Write-Info "Downloading ADK installer..."
        Invoke-DownloadFile -Url $AdkDownloadUrl -Destination $adkInstaller

        if (-not (Test-Path $adkInstaller) -or (Get-Item $adkInstaller).Length -lt 1MB) {
            throw "ADK installer download failed or is corrupt"
        }

        Write-Info "Downloading WinPE add-on installer..."
        Invoke-DownloadFile -Url $WinPeAddonDownloadUrl -Destination $winPeInstaller

        if (-not (Test-Path $winPeInstaller) -or (Get-Item $winPeInstaller).Length -lt 1MB) {
            throw "WinPE add-on installer download failed or is corrupt"
        }

        # Remove any existing/partial ADK installations (fixes error 2001)
        Remove-ExistingADK -AdkInstallerPath $adkInstaller -WinPeInstallerPath $winPeInstaller

        # Install ADK base - use default path, add logging, disable CEIP
        Write-Info "Installing Windows ADK (this may take several minutes)..."
        $adkArgs = "/quiet /norestart /ceip off /log `"$adkLog`" /features OptionId.DeploymentTools"
        $adkProcess = Start-Process -FilePath $adkInstaller -ArgumentList $adkArgs -Wait -PassThru

        if ($adkProcess.ExitCode -notin @(0, 3010)) {
            $logTail = if (Test-Path $adkLog) { Get-Content $adkLog -Tail 30 -ErrorAction SilentlyContinue | Out-String } else { "No log file" }
            throw "ADK installer exited with code $($adkProcess.ExitCode)`nLog:`n$logTail"
        }

        if ($adkProcess.ExitCode -eq 3010) {
            Write-Warn "ADK installed but a reboot may be required"
        }

        # Verify ADK actually installed before continuing
        $retryCount = 0
        while (-not (Test-AdkDeploymentToolsInstalled) -and $retryCount -lt 15) {
            Start-Sleep -Seconds 2
            $retryCount++
        }

        if (-not (Test-AdkDeploymentToolsInstalled)) {
            throw "Windows ADK Deployment Tools were not detected after installation"
        }

        Write-Info "ADK base installed successfully"

        # Wait for registry and file system to settle before WinPE install
        Write-Info "Waiting for ADK registration to complete..."
        Start-Sleep -Seconds 10
        Reset-InstallerState

        # Install WinPE add-on
        Write-Info "Installing Windows PE add-on (this may take several minutes)..."
        $winPeArgs = "/quiet /norestart /ceip off /log `"$winPeLog`" /features OptionId.WindowsPreinstallationEnvironment"
        $winPeProcess = Start-Process -FilePath $winPeInstaller -ArgumentList $winPeArgs -Wait -PassThru

        if ($winPeProcess.ExitCode -notin @(0, 3010)) {
            $logTail = if (Test-Path $winPeLog) { Get-Content $winPeLog -Tail 30 -ErrorAction SilentlyContinue | Out-String } else { "No log file" }
            throw "WinPE add-on installer exited with code $($winPeProcess.ExitCode)`nLog:`n$logTail"
        }

        if ($winPeProcess.ExitCode -eq 3010) {
            Write-Warn "WinPE add-on installed but a reboot may be required"
        }

        Write-Info "Windows ADK and WinPE add-on installed successfully"

    } finally {
        Remove-Item $adkInstaller -Force -ErrorAction SilentlyContinue
        Remove-Item $winPeInstaller -Force -ErrorAction SilentlyContinue
    }
}

function Ensure-ADK {
    if ($SkipADK) {
        Write-Warn "Skipping ADK and WinPE checks"
        return
    }

    Write-Step "Checking Windows ADK and WinPE add-on"
    $hasAdk = Test-AdkDeploymentToolsInstalled
    $hasWinPe = Test-WinPeAddonInstalled

    if ($hasAdk -and $hasWinPe) {
        Write-Info "Windows ADK and WinPE add-on already installed"
        return
    }

    Write-Warn "Windows ADK and/or WinPE add-on not detected"
    if (-not (Confirm-Choice -Prompt "Install ADK and WinPE add-on now?" -Default $false)) {
        Write-Warn "Continuing without ADK. GhostWin validate/build will still require it later."
        return
    }

    try {
        Install-ADKDirect
    } catch {
        Write-Warn "ADK/WinPE install failed"
        Write-Fail $_.Exception.Message
        Write-Host "  Install manually if needed:" -ForegroundColor Yellow
        Write-Host "    ADK: $AdkDownloadUrl" -ForegroundColor Gray
        Write-Host "    WinPE Add-on: $WinPeAddonDownloadUrl" -ForegroundColor Gray
        if ($NonInteractive) {
            throw "Failed to install Windows ADK and WinPE add-on"
        }

        if (Confirm-Choice -Prompt "Open Microsoft download pages in your browser?" -Default $true) {
            Start-Process $AdkDownloadUrl
            Start-Process $WinPeAddonDownloadUrl
        }
        throw "Failed to install Windows ADK and WinPE add-on"
    }

    $hasAdk = Test-AdkDeploymentToolsInstalled
    $hasWinPe = Test-WinPeAddonInstalled
    if (-not ($hasAdk -and $hasWinPe)) {
        Write-Host "  Manual install links:" -ForegroundColor Yellow
        Write-Host "    ADK: $AdkDownloadUrl" -ForegroundColor Gray
        Write-Host "    WinPE Add-on: $WinPeAddonDownloadUrl" -ForegroundColor Gray
        throw "Windows ADK or WinPE add-on was not detected after installation"
    }

    Write-Info "Windows ADK install command completed"
}

function Ensure-Rust {
    if ($SkipRust) {
        Write-Warn "Skipping Rust installation"
        return
    }

    Write-Step "Checking Rust toolchain"
    Refresh-Path
    if (Test-CommandExists "cargo") {
        Write-Info "Rust already installed: $(cargo --version)"
        return
    }

    $rustupPath = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-DownloadFile -Url "https://win.rustup.rs/x86_64" -Destination $rustupPath

    Write-Info "Installing Rust toolchain"
    & $rustupPath -y --default-toolchain stable --default-host x86_64-pc-windows-msvc
    Remove-Item $rustupPath -Force -ErrorAction SilentlyContinue
    Refresh-Path

    if (-not (Test-CommandExists "cargo")) {
        throw "Rust installation completed but cargo was not found in PATH"
    }

    Write-Info "Rust installed: $(cargo --version)"
}

function Reset-InstallDirectory {
    param([string]$Path)

    Write-Step "Preparing installation directory"
    if (Test-Path $Path) {
        Write-Info "Cleaning existing directory $Path"
        Get-ChildItem -Path $Path -Force -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
    } else {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Install-SourceTree {
    param([string]$Path)

    Write-Step "Downloading GhostWin source"
    $zipPath = Join-Path $env:TEMP "ghostwin-main.zip"
    $extractPath = Join-Path $env:TEMP "ghostwin-main"

    Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
    Remove-Item $extractPath -Recurse -Force -ErrorAction SilentlyContinue

    Invoke-DownloadFile -Url (Get-SourceArchiveUrl) -Destination $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $env:TEMP -Force
    Move-Item (Join-Path $extractPath "*") $Path -Force

    Remove-Item $extractPath -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
}

function Build-FromSource {
    param([string]$Path)

    if ($SkipBuild) {
        Write-Warn "Skipping build as requested"
        return
    }

    Write-Step "Building GhostWin from source"
    Push-Location $Path
    try {
        $output = & cargo build --release 2>&1
        if ($LASTEXITCODE -ne 0 -or -not (Test-Path ".\target\release\ghostwin.exe")) {
            $tail = ($output | Select-Object -Last 30) -join [Environment]::NewLine
            throw "cargo build --release failed`n$tail"
        }
    } finally {
        Pop-Location
    }
}

function Get-InstalledExecutable {
    param([string]$Path)

    $candidates = @(
        (Join-Path $Path "target\release\ghostwin.exe")
        (Join-Path $Path "ghostwin.exe")
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            return $candidate
        }
    }

    $discovered = Get-ChildItem -Path $Path -Filter "ghostwin*.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($discovered) {
        return $discovered.FullName
    }

    return $null
}

function Add-ExecutableToPath {
    param([string]$ExecutablePath)

    $directory = Split-Path -Parent $ExecutablePath
    $currentUserPath = [Environment]::GetEnvironmentVariable("PATH", "User")

    if ($currentUserPath -like "*$directory*") {
        Write-Info "GhostWin directory already present in user PATH"
        return
    }

    $newPath = if ([string]::IsNullOrWhiteSpace($currentUserPath)) {
        $directory
    } else {
        "$currentUserPath;$directory"
    }

    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Info "Added $directory to user PATH"
}

function Validate-Installation {
    param([string]$ExecutablePath)

    Write-Step "Validating GhostWin installation"
    & $ExecutablePath --version
    if ($LASTEXITCODE -ne 0) {
        throw "Installed GhostWin executable failed version check"
    }

    try {
        & $ExecutablePath validate
        if ($LASTEXITCODE -ne 0) {
            Write-Warn "ghostwin validate completed with warnings"
        }
    } catch {
        Write-Warn "ghostwin validate could not complete: $($_.Exception.Message)"
    }
}

if ($Help) {
    Write-Host @"
GhostWin Windows Installer

Recommended one-liners:
  irm https://win.cktech.sh | iex

Fallback GitHub raw URL:
  irm https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

Options:
  -SkipRust        Do not install Rust
  -SkipBuild       Download source but do not run cargo build
  -SkipADK         Do not prompt for ADK / WinPE installation
  -SkipBuildTools  Do not prompt for Visual Studio Build Tools installation
  -AddToPath       Add installed GhostWin executable directory to the user PATH
  -NonInteractive  Avoid prompts and use safe defaults
  -InstallPath     Installation directory (default: C:\ProgramData\CKTech\GhostWin)
  -Help            Show this help
"@
    exit 0
}

Write-Host "GhostWin Windows Installer" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan

if (-not (Test-WindowsHost)) {
    throw "install.ps1 only supports Windows hosts"
}

if (-not (Test-Admin)) {
    Write-Warn "Installer is not running elevated. GhostWin can still install, but dependency setup may be limited."
}

try {
    Reset-InstallDirectory -Path $InstallPath

    if (-not $SkipBuild) {
        Ensure-BuildTools
        Ensure-Rust
    }

    Ensure-ADK

    Install-SourceTree -Path $InstallPath
    if (-not $SkipBuild) {
        Build-FromSource -Path $InstallPath
    }

    $executablePath = Get-InstalledExecutable -Path $InstallPath
    if (-not $executablePath) {
        throw "GhostWin executable was not found after installation"
    }

    if ($AddToPath -or (Confirm-Choice -Prompt "Add GhostWin to your user PATH?" -Default $false)) {
        Add-ExecutableToPath -ExecutablePath $executablePath
    }

    Validate-Installation -ExecutablePath $executablePath

    Write-Host "" 
    Write-Host "GhostWin installation complete." -ForegroundColor Green
    Write-Host "  Install path: $InstallPath" -ForegroundColor Cyan
    Write-Host "  Executable:   $executablePath" -ForegroundColor Cyan
    Write-Host "" 
    Write-Host "Quick start:" -ForegroundColor Yellow
    Write-Host "  `"$executablePath`" gui" -ForegroundColor Gray
    Write-Host "  `"$executablePath`" validate" -ForegroundColor Gray
    Write-Host "  `"$executablePath`" build --source-iso C:\path\to\Windows.iso --output-dir C:\ghostwin-build --output-iso C:\ghostwin-build\ghostwin.iso --verify" -ForegroundColor Gray
} catch {
    Write-Fail $_.Exception.Message
    Pause-IfInteractive
    return
}
