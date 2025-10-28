; GhostWin InnoSetup Installer Script
; Modern Windows deployment toolkit installer
; Version: 0.3.3

#define MyAppName "GhostWin"
#define MyAppVersion "0.3.3"
#define MyAppPublisher "Resolve Technology"
#define MyAppURL "https://github.com/CK-Technology/ghostwin"
#define MyAppExeName "ghostwin.exe"

[Setup]
; Application Information
AppId={{8B9C5E7A-3F2D-4A1B-9E6F-7D8C4B3A2E1D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=LICENSE
OutputDir=dist
OutputBaseFilename=GhostWin-Setup-v{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
DisableDirPage=no
DisableProgramGroupPage=no
SetupIconFile=assets\icons\ghostwin.ico
UninstallDisplayIcon={app}\{#MyAppExeName}

; Visual Settings
WizardImageFile=assets\wizard-image.bmp
WizardSmallImageFile=assets\wizard-small.bmp

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath"; Description: "Add {#MyAppName} to system PATH"; GroupDescription: "System Integration:"; Flags: checkedonce
Name: "associateiso"; Description: "Associate .iso files with {#MyAppName} (optional)"; GroupDescription: "File Associations:"; Flags: unchecked

[Files]
; Main Executable
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

; Configuration Files
Source: "ghostwin.toml"; DestDir: "{app}"; Flags: ignoreversion confirmoverwrite
Source: "config\*"; DestDir: "{app}\config"; Flags: ignoreversion recursesubdirs createallsubdirs

; Tools Directory
Source: "tools\*"; DestDir: "{app}\tools"; Flags: ignoreversion recursesubdirs createallsubdirs

; Scripts Directory
Source: "scripts\*"; DestDir: "{app}\scripts"; Flags: ignoreversion recursesubdirs createallsubdirs

; PE AutoRun Directory
Source: "pe_autorun\*"; DestDir: "{app}\pe_autorun"; Flags: ignoreversion recursesubdirs createallsubdirs

; Resources (Icons, Fonts, etc.)
Source: "resources\*"; DestDir: "{app}\resources"; Flags: ignoreversion recursesubdirs createallsubdirs; Tasks: ; Languages:

; Documentation
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "DOCS.md"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "COMMANDS.md"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "GUNPOWDER.md"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

; Visual C++ Redistributables (bundled)
Source: "dependencies\vcredist_x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall

; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
; Start Menu Icons
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{#MyAppName} GUI"; Filename: "{app}\{#MyAppExeName}"; Parameters: "gui"; IconFilename: "{app}\resources\icons\gui.ico"
Name: "{group}\{#MyAppName} Validate"; Filename: "{app}\{#MyAppExeName}"; Parameters: "validate"; IconFilename: "{app}\resources\icons\validate.ico"
Name: "{group}\Documentation"; Filename: "{app}\docs"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"

; Desktop Icon
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

; Quick Launch Icon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: quicklaunchicon

[Run]
; Install Visual C++ Redistributables
Filename: "{tmp}\vcredist_x64.exe"; Parameters: "/quiet /norestart"; StatusMsg: "Installing Visual C++ Redistributables 2022..."; Check: VCRedistNeedsInstall; Flags: waituntilterminated

; Optional: Run GhostWin validation after installation
Filename: "{app}\{#MyAppExeName}"; Parameters: "validate"; Description: "Run system validation check"; Flags: postinstall shellexec skipifsilent nowait

; Optional: Open documentation
Filename: "{app}\README.md"; Description: "View README documentation"; Flags: postinstall shellexec skipifsilent nowait unchecked

[UninstallRun]
; Clean up any created configurations (optional)
Filename: "{cmd}"; Parameters: "/c rd /s /q ""{userappdata}\GhostWin"""; Flags: runhidden; RunOnceId: "CleanUserData"

[Code]
function VCRedistNeedsInstall: Boolean;
var
  Version: String;
begin
  // Check if VC++ 2015-2022 Redistributable is installed
  Result := not RegQueryStringValue(HKLM64, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Version', Version);
  if not Result then
  begin
    // Also accept newer versions
    Result := not RegQueryStringValue(HKLM64, 'SOFTWARE\Microsoft\DevDiv\vc\Servicing\14.0\RuntimeMinimum', 'Version', Version);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Path: string;
  AppPath: string;
begin
  if CurStep = ssPostInstall then
  begin
    // Add to PATH if requested
    if IsTaskSelected('addtopath') then
    begin
      AppPath := ExpandConstant('{app}');
      if RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Path) then
      begin
        if Pos(AppPath, Path) = 0 then
        begin
          Path := Path + ';' + AppPath;
          RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Path);

          // Notify system of environment change
          Log('Added to PATH: ' + AppPath);
        end
        else
        begin
          Log('Already in PATH: ' + AppPath);
        end;
      end;
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Path: string;
  AppPath: string;
  NewPath: string;
  P: Integer;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // Remove from PATH
    AppPath := ExpandConstant('{app}');
    if RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Path) then
    begin
      P := Pos(';' + AppPath, Path);
      if P > 0 then
      begin
        NewPath := Copy(Path, 1, P - 1) + Copy(Path, P + Length(';' + AppPath), Length(Path));
        RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', NewPath);
        Log('Removed from PATH: ' + AppPath);
      end
      else
      begin
        P := Pos(AppPath + ';', Path);
        if P > 0 then
        begin
          NewPath := Copy(Path, 1, P - 1) + Copy(Path, P + Length(AppPath + ';'), Length(Path));
          RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', NewPath);
          Log('Removed from PATH: ' + AppPath);
        end;
      end;
    end;
  end;
end;

function InitializeSetup(): Boolean;
begin
  Result := True;

  // Check if Windows 10/11
  if (GetWindowsVersion < $0A000000) then // Windows 10 is version 10.0
  begin
    MsgBox('{#MyAppName} requires Windows 10 or later.', mbError, MB_OK);
    Result := False;
  end;

  // Check if running as administrator
  if not IsAdmin then
  begin
    MsgBox('{#MyAppName} installation requires administrator privileges.' + #13#10 +
           'Please run the installer as administrator.', mbError, MB_OK);
    Result := False;
  end;
end;

function GetUninstallString: String;
var
  sUnInstPath: String;
  sUnInstallString: String;
begin
  sUnInstPath := ExpandConstant('Software\Microsoft\Windows\CurrentVersion\Uninstall\{#emit SetupSetting("AppId")}_is1');
  sUnInstallString := '';
  if not RegQueryStringValue(HKLM, sUnInstPath, 'UninstallString', sUnInstallString) then
    RegQueryStringValue(HKCU, sUnInstPath, 'UninstallString', sUnInstallString);
  Result := sUnInstallString;
end;

function IsUpgrade: Boolean;
begin
  Result := (GetUninstallString() <> '');
end;

function UnInstallOldVersion(): Integer;
var
  sUnInstallString: String;
  iResultCode: Integer;
begin
  Result := 0;
  sUnInstallString := GetUninstallString();
  if sUnInstallString <> '' then begin
    sUnInstallString := RemoveQuotes(sUnInstallString);
    if Exec(sUnInstallString, '/SILENT /NORESTART /SUPPRESSMSGBOXES','', SW_HIDE, ewWaitUntilTerminated, iResultCode) then
      Result := 3
    else
      Result := 2;
  end else
    Result := 1;
end;

procedure InitializeWizard;
var
  ResultCode: Integer;
begin
  if IsUpgrade() then
  begin
    if MsgBox('An older version of {#MyAppName} is installed. Do you want to uninstall it first?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      ResultCode := UnInstallOldVersion();
      if ResultCode = 0 then
        MsgBox('Failed to uninstall old version.', mbError, MB_OK);
    end;
  end;
end;

[Registry]
; File Association for .iso files (optional)
Root: HKCR; Subkey: ".iso\OpenWithProgids"; ValueType: string; ValueName: "GhostWin.IsoFile"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateiso
Root: HKCR; Subkey: "GhostWin.IsoFile"; ValueType: string; ValueName: ""; ValueData: "ISO Disk Image"; Flags: uninsdeletekey; Tasks: associateiso
Root: HKCR; Subkey: "GhostWin.IsoFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: associateiso
Root: HKCR; Subkey: "GhostWin.IsoFile\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" build --source-iso ""%1"""; Tasks: associateiso
