[Setup]
AppName=Test App
AppVersion=1.0
DefaultDirName=C:\TestApp
Compression=zip
OutputDir=..\test_output
LicenseText=END USER LICENSE AGREEMENT\n\nThis is a test license.\nPlease accept to continue.

[Files]
Source: "hello.txt"; DestDir: "{app}"

[Icons]
Name: "{group}\Test App"; Filename: "{app}\hello.txt"
Name: "{autodesktop}\Test App"; Filename: "{app}\hello.txt"
