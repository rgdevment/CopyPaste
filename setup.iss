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

[InstallDelete]
; Clean old app files before installing new version (preserves user data in AppData)
Type: filesandordirs; Name: "{app}\*.dll"
Type: filesandordirs; Name: "{app}\*.exe"
Type: filesandordirs; Name: "{app}\*.pri"
Type: filesandordirs; Name: "{app}\*.json"
Type: filesandordirs; Name: "{app}\Assets"
Type: filesandordirs; Name: "{app}\Microsoft.UI.Xaml"
Type: filesandordirs; Name: "{app}\NpuDetect"
Type: filesandordirs; Name: "{app}\en-us"

[UninstallDelete]
; Clean all app files on uninstall (user data in AppData\Local\CopyPaste is preserved)
Type: filesandordirs; Name: "{app}\*"

[Code]
const
  UninstallRegKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{{AE2A10DA-F6FA-417B-8C06-99EBA788AFFE}}_is1';

function GetUninstallString(): String;
var
  UninstallString: String;
begin
  Result := '';
  if RegQueryStringValue(HKCU, UninstallRegKey, 'UninstallString', UninstallString) then
    Result := UninstallString
  else if RegQueryStringValue(HKLM, UninstallRegKey, 'UninstallString', UninstallString) then
    Result := UninstallString;
end;

function IsUpgrade(): Boolean;
begin
  Result := GetUninstallString() <> '';
end;

procedure CloseRunningApp();
var
  ResultCode: Integer;
begin
  // First try graceful close, then force kill with process tree
  Exec('taskkill', '/IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Sleep(300);
  // Force kill including child processes
  Exec('taskkill', '/F /T /IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Sleep(500);
end;

function UninstallPreviousVersion(): Boolean;
var
  UninstallString: String;
  ResultCode: Integer;
begin
  Result := True;
  UninstallString := GetUninstallString();
  
  if UninstallString <> '' then
  begin
    CloseRunningApp();
    
    UninstallString := RemoveQuotes(UninstallString);
    Exec(UninstallString, '/VERYSILENT /NORESTART /SUPPRESSMSGBOXES', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    Sleep(1000);
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  NeedsRestart := False;
  
  if IsUpgrade() then
  begin
    WizardForm.PreparingLabel.Caption := 'Removing previous version...';
    UninstallPreviousVersion();
  end;
end;

function InitializeUninstall(): Boolean;
begin
  Result := True;
  CloseRunningApp();
end;
