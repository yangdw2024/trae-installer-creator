; -- MyProgram.iss --
; Example script for Trae Installer Creator

[Setup]
AppName=My Program
AppVersion=1.0
WizardStyle=modern dynamic
DefaultDirName={autopf}\My Program
DefaultGroupName=My Program
UninstallDisplayIcon={app}\MyProg.exe
Compression=lzma2
SolidCompression=yes
OutputDir=output

[Files]
Source: "MyProg.exe"; DestDir: "{app}"
Source: "MyProg.chm"; DestDir: "{app}"
Source: "Readme.txt"; DestDir: "{app}"; Flags: isreadme

[Icons]
Name: "{group}\My Program"; Filename: "{app}\MyProg.exe"
Name: "{autodesktop}\My Program"; Filename: "{app}\MyProg.exe"