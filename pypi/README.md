# ForgePack v3.0

A fast, native Rust Windows single-file installer builder compatible with Inno Setup (`.iss`) scripts.

## Features

- **Ultra-Small Single-File Stub (`forge-stub.exe`)**: ~460 KB native Win32 runtime with zero external dependencies, custom icon, PerMonitorV2 high-DPI 5-step wizard, `/SILENT` + `/DIR="..."` automation, and self-deleting uninstaller (`unins000.exe`).
- **Native CLI Compiler (`forgecc.exe`)**: Millisecond `.iss` compilation with solid zlib streaming & CRC32 verification.
- **Native GUI IDE (`forge-studio.exe`)**: 5-tab Win32 studio with smart folder import and bidirectional `.iss` script synchronization.

## Quick Start

```bash
# Compile an .iss script into a single-file Setup.exe
forgecc examples/MyProgram.iss -O output -F MySetup

# Silent installation
MySetup.exe /SILENT /DIR="C:\MyApp"
```
