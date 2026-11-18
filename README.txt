========================================
  Trae Installer Creator
  Installer Builder Tool v1.0
========================================

DESCRIPTION
-----------
Trae Installer Creator is a powerful Windows installer builder tool
that supports both GUI and command-line compilation, compatible with
Inno Setup script format.

SYSTEM REQUIREMENTS
-------------------
- Windows 7 or later
- Python 3.8 or later
- pywin32 module

QUICK START
-----------

Step 1: Install Dependencies
  Double-click "install_deps.bat" to install required Python packages.

Step 2: Launch Application
  Option 1 (Recommended): Double-click "TraeInstallerCreator.vbs" 
                          to launch GUI (no console window)
  Option 2: Double-click "start.bat" to launch GUI (with console log)
  Option 3: Use "cli.bat" for command-line compilation

USAGE
-----

GUI Mode:
  1. Fill in application name, version, etc. in "Setup" tab
  2. Add files to package in "Files" tab
  3. Add shortcuts in "Icons" tab
  4. Click "Build Installer" to generate setup.exe

Command Line:
  cli.bat script_file.iss [options]

  Examples:
    cli.bat examples\MyProgram.iss
    cli.bat myscript.iss -O "d:\output" -F "MySetup"

SCRIPT FORMAT
-------------
Script uses .iss format, compatible with Inno Setup:

[Setup]
AppName=My Program
AppVersion=1.0
DefaultDirName={autopf}\My Program
Compression=zip
OutputDir=output

[Files]
Source: "MyProg.exe"; DestDir: "{app}"
Source: "Readme.txt"; DestDir: "{app}"; Flags: isreadme

[Icons]
Name: "{group}\My Program"; Filename: "{app}\MyProg.exe"
Name: "{autodesktop}\My Program"; Filename: "{app}\MyProg.exe"

DIRECTORY STRUCTURE
-------------------
  TraeInstallerCreator/
  ├── TraeInstallerCreator.vbs   GUI launcher (no console)
  ├── start.bat                  GUI launcher (with console)
  ├── cli.bat                    Command line compiler
  ├── install_deps.bat           Dependency installer
  ├── README.txt                 This file
  ├── src/                       Source code
  │   ├── core/                  Core modules
  │   ├── gui/                   GUI code
  │   ├── cli/                   CLI code
  │   ├── launcher.py            Startup helper
  │   └── traecc_entry.py        CLI entry point
  └── examples/                  Example scripts

TROUBLESHOOTING
---------------

Q: Nothing happens when I double-click?
A: Run "install_deps.bat" first to install dependencies.
   Ensure Python 3.8+ is installed.

Q: Where is the output installer?
A: Default output is in the OutputDir specified in script.

Q: What compression methods are supported?
A: zip, zlib, bzip2, lzma, lzma2. 
   Recommended: zip (best compatibility)

VERSION HISTORY
---------------
v1.0 - Initial Release
    - Basic installer creation
    - Inno Setup script compatibility
    - GUI and CLI modes
    - File compression and shortcut creation

========================================
  Copyright (C) 2026 Trae
========================================
