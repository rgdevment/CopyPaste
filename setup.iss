#define MyAppName "CopyPaste"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#define MyAppPublisher "RGDevment"
#define MyAppExeName "CopyPaste.exe"
#define MyAppIcon "CopyPaste.UI\Assets\CopyPasteLogoSimple.ico"
#ifndef PublishDir
  #define PublishDir "CopyPaste.UI\bin\Release\net10.0-windows10.0.22621.0\win-x64\publish"
#endif
#ifndef MyArch
  #define MyArch "x64"
#endif

[Setup]
AppId={{AE2A10DA-F6FA-417B-8C06-99EBA788AFFE}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\{#MyAppName}
ArchitecturesAllowed={#MyArch}
ArchitecturesInstallIn64BitMode={#MyArch}
LicenseFile={#RepoRoot}\LICENSE
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=CopyPaste_Setup_{#MyArch}
Compression=lzma
SolidCompression=yes
SetupIconFile={#MyAppIcon}
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} Installer
VersionInfoTextVersion={#MyAppVersion}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
WizardStyle=modern
DisableWelcomePage=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#PublishDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs; Excludes: "*.pdb,*.xml,createdump.exe"
Source: "{#RepoRoot}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon; IconFilename: "{app}\{#MyAppExeName}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

; Inno Setup only removes files it installed - user data is preserved automatically

[Code]
const
  UninstallRegKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{{AE2A10DA-F6FA-417B-8C06-99EBA788AFFE}}_is1';

function GetUninstallExePath(): String;
var
  InstallPath: String;
begin
  Result := '';
  // Try to get install location from registry
  if RegQueryStringValue(HKCU, UninstallRegKey, 'InstallLocation', InstallPath) then
    Result := InstallPath + 'unins000.exe'
  else if RegQueryStringValue(HKLM, UninstallRegKey, 'InstallLocation', InstallPath) then
    Result := InstallPath + 'unins000.exe';
    
  // Fallback: check default location
  if (Result = '') or (not FileExists(Result)) then
    Result := ExpandConstant('{localappdata}\{#MyAppName}\unins000.exe');
end;

function IsAppInstalled(): Boolean;
var
  UninstallExe: String;
begin
  UninstallExe := GetUninstallExePath();
  Result := FileExists(UninstallExe);
end;

function UninstallPreviousVersion(): Boolean;
var
  UninstallExe: String;
  ResultCode: Integer;
begin
  Result := True;
  UninstallExe := GetUninstallExePath();
  
  if FileExists(UninstallExe) then
  begin
    // Run the native Inno Setup uninstaller
    // The uninstaller handles closing the app and removing files automatically
    Exec(UninstallExe, '/VERYSILENT /NORESTART /SUPPRESSMSGBOXES', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    
    // Wait for uninstaller to fully complete
    Sleep(2000);
    
    // Verify uninstallation completed
    if FileExists(UninstallExe) then
    begin
      Log('Warning: Uninstaller still exists after execution');
    end;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  NeedsRestart := False;
  
  if IsAppInstalled() then
  begin
    WizardForm.PreparingLabel.Caption := 'Removing previous version...';
    UninstallPreviousVersion();
  end;
end;
