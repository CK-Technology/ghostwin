# GhostWin Packaging & Distribution Plan

## Goal
Create a standalone Windows installer that doesn't require end-users to install Rust, run PowerShell scripts, or have development tools.

## Recommended Approach: Hybrid Solution

### Phase 1: Immediate (1-2 days)
1. **Create InnoSetup installer** for quick distribution
   - Bundle pre-compiled `ghostwin.exe`
   - Include all tools and scripts
   - Check for and install Visual C++ Redistributables
   - Add to PATH
   - Create desktop/start menu shortcuts

### Phase 2: Short-term (1 week)
1. **Setup GitHub Actions CI/CD**
   ```yaml
   name: Release
   on:
     push:
       tags:
         - 'v*'
   jobs:
     build-windows:
       runs-on: windows-latest
       steps:
         - uses: actions/checkout@v4
         - name: Setup Rust
           uses: dtolnay/rust-toolchain@stable
         - name: Build Release
           run: cargo build --release
         - name: Package with InnoSetup
           run: iscc installer.iss
         - name: Upload Artifacts
           uses: actions/upload-artifact@v4
   ```

2. **Create portable ZIP distribution**
   - Single folder with all dependencies
   - No installation required
   - Can run from USB drive

### Phase 3: Long-term (2-4 weeks)
1. **Professional MSI using WiX v4**
   - Enterprise-friendly deployment
   - Group Policy support
   - Silent installation options
   - Proper uninstaller

## InnoSetup Script Template

```iss
[Setup]
AppName=GhostWin
AppVersion=0.3.3
AppPublisher=Resolve Technology
AppPublisherURL=https://github.com/CK-Technology/ghostwin
DefaultDirName={commonpf}\GhostWin
DefaultGroupName=GhostWin
OutputBaseFilename=GhostWin-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\ghostwin.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "tools\*"; DestDir: "{app}\tools"; Flags: ignoreversion recursesubdirs
Source: "scripts\*"; DestDir: "{app}\scripts"; Flags: ignoreversion recursesubdirs
Source: "pe_autorun\*"; DestDir: "{app}\pe_autorun"; Flags: ignoreversion recursesubdirs
Source: "resources\*"; DestDir: "{app}\resources"; Flags: ignoreversion recursesubdirs
Source: "ghostwin.toml"; DestDir: "{app}"; Flags: ignoreversion
Source: "vcredist_x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall

[Icons]
Name: "{group}\GhostWin"; Filename: "{app}\ghostwin.exe"
Name: "{group}\GhostWin GUI"; Filename: "{app}\ghostwin.exe"; Parameters: "gui"
Name: "{group}\Uninstall GhostWin"; Filename: "{uninstallexe}"
Name: "{commondesktop}\GhostWin"; Filename: "{app}\ghostwin.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop icon"; GroupDescription: "Additional icons:"
Name: "addtopath"; Description: "Add GhostWin to PATH"; GroupDescription: "System integration:"

[Run]
Filename: "{tmp}\vcredist_x64.exe"; Parameters: "/quiet /norestart"; StatusMsg: "Installing Visual C++ Redistributables..."; Check: VCRedistNeedsInstall

[Code]
function VCRedistNeedsInstall: Boolean;
begin
  Result := not RegKeyExists(HKLM, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64');
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Path: string;
begin
  if CurStep = ssPostInstall then
  begin
    // Add to PATH if requested
    if IsTaskSelected('addtopath') then
    begin
      RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Path);
      if Pos(ExpandConstant('{app}'), Path) = 0 then
      begin
        Path := Path + ';' + ExpandConstant('{app}');
        RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Path);
      end;
    end;
  end;
end;
```

## Required Prerequisites (Bundled or Downloaded)

1. **Visual C++ Redistributables 2022** (always bundle)
2. **Windows ADK** (optional, prompt to download if needed for ISO building)
3. **Windows PE Add-on** (optional, prompt if ADK is installed)

## Build Automation Script

```powershell
# build-installer.ps1
param(
    [string]$Version = "0.3.3"
)

Write-Host "Building GhostWin Installer v$Version" -ForegroundColor Cyan

# Build the Rust binary
Write-Host "Building release binary..." -ForegroundColor Yellow
cargo build --release --target x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) {
    throw "Build failed"
}

# Download VC++ Redistributables if not present
$vcRedistUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
$vcRedistPath = "vcredist_x64.exe"
if (-not (Test-Path $vcRedistPath)) {
    Write-Host "Downloading Visual C++ Redistributables..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $vcRedistUrl -OutFile $vcRedistPath
}

# Update version in InnoSetup script
$issContent = Get-Content "installer.iss" -Raw
$issContent = $issContent -replace 'AppVersion=.*', "AppVersion=$Version"
Set-Content "installer.iss" -Value $issContent

# Compile installer
Write-Host "Compiling installer..." -ForegroundColor Yellow
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer.iss

Write-Host "✅ Installer created: Output\GhostWin-Setup.exe" -ForegroundColor Green
```

## Distribution Channels

1. **GitHub Releases** (primary)
   - Automated via GitHub Actions
   - Include both installer and portable ZIP

2. **Project Website**
   - Direct download links
   - Installation instructions

3. **winget** (future)
   - Submit manifest to Windows Package Manager
   - Enables: `winget install ghostwin`

## File Size Optimization

- Compiled binary: ~15-20 MB
- Tools & scripts: ~50 MB
- Total installer size: ~35-40 MB (compressed)
- Portable ZIP: ~60-70 MB

## Testing Requirements

Before each release:
1. Test on clean Windows 10/11 VMs
2. Verify PATH integration
3. Test uninstaller
4. Validate all tools load correctly
5. Test GUI launches in WinPE environment

## Success Metrics

- Installation time: < 30 seconds
- No manual prerequisites
- Single-click installation
- Works on Windows 10 1809+ and Windows 11
- No admin rights required for running (only for installation)