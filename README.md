# Trae Installer Creator v3.0

A fast, native Rust Windows single-file installer builder compatible with Inno Setup (`.iss`) scripts.

## Architecture (Rust Workspace)

- `crates/trae-core`: `.iss` AST parser/generator, path constant expander, 32-byte EOF footer (`TRAESET3` / `TRAEUNI3`), solid zlib stream packer/extractor with CRC32 verification, and single-file `.exe` builder.
- `crates/trae-stub`: Ultra-small (~454 KB) native Win32 single-file installer & self-deleting uninstaller (`unins000.exe`), supporting 5-step high-DPI GUI wizard and `/SILENT` + `/DIR="..."` automation.
- `crates/traecc`: Native CLI compiler (`traecc.exe`).
- `crates/trae-studio`: Native Win32 GUI IDE (`trae-studio.exe`) with bidirectional `.iss` script sync.

## Quick Start

```bash
# Compile an .iss script into a single-file Setup.exe
traecc examples/MyProgram.iss -O output -F MySetup

# Silent installation
MySetup.exe /SILENT /DIR="C:\MyApp"
```
