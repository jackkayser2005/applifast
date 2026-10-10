; Compile the validated six-file payload produced by applifast-preview.ps1.
#ifndef PayloadDir
  #error PayloadDir is required
#endif
#ifndef OutputDir
  #error OutputDir is required
#endif
#ifndef OutputName
  #error OutputName is required
#endif
#ifndef Version
  #error Version is required
#endif
#ifndef SourceRevision
  #error SourceRevision is required
#endif

; A QA build uses an isolated uninstall identity and never launches the app.
#ifdef TestIdentity
  #define AppIdentity TestIdentity
  #define AppName "Applifast installer QA " + TestIdentity
#else
  ; Keep stable across Applifast upgrades. Distinct from upstream Spotifast.
  #define AppIdentity "Applifast.Windows"
  #define AppName "Applifast"
#endif

[Setup]
AppId={#AppIdentity}
AppName={#AppName}
AppVersion={#Version}
AppVerName={#AppName} development preview ({#SourceRevision})
AppPublisher=Applifast contributors
AppSupportURL=https://github.com/jackkayser2005/applifast/issues
DefaultDirName={localappdata}\Programs\Applifast
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
LicenseFile=..\..\LICENSE
OutputDir={#OutputDir}
OutputBaseFilename={#OutputName}-setup
SetupIconFile=spotifast.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=no
RestartApplications=no
UninstallDisplayIcon={app}\Applifast.exe
VersionInfoVersion={#Version}

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#PayloadDir}\Applifast.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\BUILD.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\README.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\TESTING.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\developer-token.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\Applifast.exe"; AppUserModelID: "applifast"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\Applifast.exe"; Tasks: desktopicon; AppUserModelID: "applifast"

#ifndef TestIdentity
[Run]
Filename: "{app}\Applifast.exe"; Description: "Launch Applifast"; Flags: nowait postinstall skipifsilent
#endif
