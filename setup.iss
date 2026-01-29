#define MyAppName "CopyPaste"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#ifndef MyAppVersionNumeric
  #define MyAppVersionNumeric "1.0.0.0"
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
VersionInfoVersion={#MyAppVersionNumeric}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} Installer
VersionInfoTextVersion={#MyAppVersion}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersionNumeric}
WizardStyle=modern
DisableWelcomePage=no
; Native app closure using Windows Restart Manager
; Close both the native launcher and the .NET app
CloseApplications=yes
CloseApplicationsFilter=CopyPaste.exe,CopyPaste.App.exe
RestartApplications=no

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
