pub mod archive;
pub mod builder;
pub mod constants;
pub mod footer;
pub mod model;
pub mod parser;

pub use archive::{extract_solid_archive, pack_solid_archive};
pub use builder::{
    read_installer_payload, read_uninstaller_manifest, write_uninstaller_exe, BuildReport,
    InstallerBuilder,
};
pub use constants::{expand_constants, resolve_desktop_dir, resolve_start_menu_group_dir};
pub use footer::{
    extract_pure_stub_bytes, PackageFooter, FOOTER_SIZE, MAGIC_INSTALLER, MAGIC_UNINSTALLER,
};
pub use model::{
    default_license_text, ArchiveFileMeta, FileEntry, IconEntry, InstallerPayloadConfig,
    RegistryEntry, RunEntry, SetupScript, SetupSection, UninstallManifest,
};
pub use parser::IssParser;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "trae_core_test_{}_{}_{}",
            label,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_parser_round_trip_and_semicolon_safety() {
        let iss = r#"
; Header comment
[Setup]
AppName=My;SemiApp
AppVersion=2.1.0
AppPublisher=Trae Corp
DefaultDirName={autopf}\MyApp
Compression=zlib
OutputDir=out
LicenseText=Line 1; with semicolon\nLine 2

[License]
LicenseText=Override line 1; semi\nOverride line 2

[Files]
Source: "app\main.exe"; DestDir: "{app}"
Source: "data\cfg;v2.ini"; DestDir: "{app}\config"; DestName: "cfg.ini"

[Icons]
Name: "{group}\MyApp"; Filename: "{app}\main.exe"; Parameters: "--mode=fast"
"#;

        let parsed = IssParser::parse_str(iss).expect("parse should succeed");
        assert_eq!(parsed.setup.app_name, "My;SemiApp");
        assert_eq!(parsed.setup.app_version, "2.1.0");
        assert_eq!(
            parsed.setup.license_text,
            "Override line 1; semi\nOverride line 2"
        );
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.files[1].source, "data\\cfg;v2.ini");
        assert_eq!(parsed.files[1].dest_dir, "{app}\\config");
        assert_eq!(parsed.files[1].dest_name, "cfg.ini");
        assert_eq!(parsed.icons.len(), 1);
        assert_eq!(parsed.icons[0].parameters, "--mode=fast");

        let regenerated = parsed.to_iss_string();
        let reparsed = IssParser::parse_str(&regenerated).expect("reparse should succeed");
        assert_eq!(reparsed.setup.app_name, parsed.setup.app_name);
        assert_eq!(reparsed.setup.license_text, parsed.setup.license_text);
        assert_eq!(reparsed.files, parsed.files);
        assert_eq!(reparsed.icons, parsed.icons);
    }

    #[test]
    fn test_footer_round_trip_and_pure_stub_extraction() {
        let dir = make_temp_dir("footer");
        let stub_path = dir.join("fake_stub.exe");
        let stub_bytes = b"MZ_FAKE_PE_HEADER_AND_SECTIONS";
        let config_bytes = br#"{"hello":"world"}"#;
        let archive_bytes = b"SOLID_STREAM";

        let footer = PackageFooter::new(
            stub_bytes.len() as u64,
            config_bytes.len() as u64,
            archive_bytes.len() as u64,
            MAGIC_INSTALLER,
        );

        let pkg_path = dir.join("pkg.exe");
        {
            let mut f = File::create(&pkg_path).unwrap();
            f.write_all(stub_bytes).unwrap();
            f.write_all(config_bytes).unwrap();
            f.write_all(archive_bytes).unwrap();
            f.write_all(&footer.to_bytes()).unwrap();
        }
        fs::write(&stub_path, stub_bytes).unwrap();

        let mut f = File::open(&pkg_path).unwrap();
        let read_footer = PackageFooter::read_from_file(&mut f)
            .unwrap()
            .expect("footer should be found");
        assert_eq!(read_footer, footer);
        assert!(read_footer.is_installer());

        let extracted_stub = extract_pure_stub_bytes(&pkg_path).unwrap();
        assert_eq!(extracted_stub, stub_bytes);

        let clean_stub = extract_pure_stub_bytes(&stub_path).unwrap();
        assert_eq!(clean_stub, stub_bytes);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_solid_archive_pack_and_extract_with_crc32() {
        let dir = make_temp_dir("archive");
        let src_dir = dir.join("src_app");
        let sub_dir = src_dir.join("assets");
        fs::create_dir_all(&sub_dir).unwrap();

        fs::write(src_dir.join("app.exe"), b"BINARY_PAYLOAD_12345").unwrap();
        fs::write(sub_dir.join("readme.txt"), b"Hello Trae Installer v3").unwrap();
        fs::write(src_dir.join("empty.dat"), b"").unwrap();

        let entries = vec![
            FileEntry {
                source: "src_app".to_string(),
                dest_dir: "{app}".to_string(),
                ..Default::default()
            },
            FileEntry {
                source: "src_app\\app.exe".to_string(),
                dest_dir: r"{app}\backup".to_string(),
                dest_name: "app_bak.exe".to_string(),
                ..Default::default()
            },
        ];

        let mut compressed = Vec::new();
        let (metas, total_size) = pack_solid_archive(&entries, &dir, &mut compressed).unwrap();
        assert_eq!(metas.len(), 4);
        assert_eq!(total_size, 20 + 23 + 0 + 20);

        let extract_dir = dir.join("installed");
        let mut progress_calls = 0;
        let installed = extract_solid_archive(
            Cursor::new(&compressed),
            &metas,
            &extract_dir,
            |_curr, _tot, _name| {
                progress_calls += 1;
            },
        )
        .unwrap();

        assert_eq!(progress_calls, 4);
        assert_eq!(installed.len(), 4);
        assert_eq!(
            fs::read(extract_dir.join("app.exe")).unwrap(),
            b"BINARY_PAYLOAD_12345"
        );
        assert_eq!(
            fs::read(extract_dir.join("assets").join("readme.txt")).unwrap(),
            b"Hello Trae Installer v3"
        );
        assert_eq!(fs::read(extract_dir.join("empty.dat")).unwrap(), b"");
        assert_eq!(
            fs::read(extract_dir.join("backup").join("app_bak.exe")).unwrap(),
            b"BINARY_PAYLOAD_12345"
        );

        // Corrupt CRC32 expectation and verify detection
        let mut bad_metas = metas.clone();
        bad_metas[0].crc32 ^= 0xFFFF_FFFF;
        let corrupt_dir = dir.join("corrupt");
        let err = extract_solid_archive(
            Cursor::new(&compressed),
            &bad_metas,
            &corrupt_dir,
            |_, _, _| {},
        )
        .unwrap_err();
        assert!(err.to_string().contains("CRC32 mismatch"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_builder_single_file_and_uninstaller() {
        let dir = make_temp_dir("builder");
        let fake_stub = dir.join("trae-stub.exe");
        fs::write(&fake_stub, b"FAKE_NATIVE_WIN32_STUB_PE").unwrap();

        let src_file = dir.join("hello.txt");
        fs::write(&src_file, b"Single-file installer test payload!").unwrap();

        let mut script = SetupScript::default();
        script.setup.app_name = "UnitApp".to_string();
        script.setup.app_version = "3.0".to_string();
        script.files.push(FileEntry {
            source: "hello.txt".to_string(),
            dest_dir: "{app}".to_string(),
            ..Default::default()
        });

        let out_exe = dir.join("UnitAppSetup.exe");
        let report =
            InstallerBuilder::build(&script, &dir, Some(&out_exe), Some(&fake_stub)).unwrap();
        assert!(report.output_path.is_file());
        assert_eq!(report.file_count, 1);

        let (footer, cfg) = read_installer_payload(&out_exe).unwrap();
        assert_eq!(cfg.setup.app_name, "UnitApp");
        assert_eq!(cfg.files_meta.len(), 1);

        let mut f = File::open(&out_exe).unwrap();
        f.seek(SeekFrom::Start(footer.archive_offset())).unwrap();
        let limited = f.take(footer.archive_size);
        let install_dir = dir.join("installed");
        let installed =
            extract_solid_archive(limited, &cfg.files_meta, &install_dir, |_, _, _| {}).unwrap();
        assert_eq!(
            fs::read(install_dir.join("hello.txt")).unwrap(),
            b"Single-file installer test payload!"
        );

        let unins_exe = install_dir.join("unins000.exe");
        let manifest = UninstallManifest {
            app_name: "UnitApp".to_string(),
            app_version: "3.0".to_string(),
            publisher: "Trae".to_string(),
            install_dir: install_dir.to_string_lossy().into_owned(),
            installed_files: installed,
            shortcuts: vec![],
            registry_subkey: r"Software\Microsoft\Windows\CurrentVersion\Uninstall\UnitApp"
                .to_string(),
        };
        write_uninstaller_exe(&out_exe, &unins_exe, &manifest).unwrap();

        let (u_footer, u_manifest) = read_uninstaller_manifest(&unins_exe).unwrap();
        assert!(u_footer.is_uninstaller());
        assert_eq!(u_manifest.app_name, "UnitApp");
        assert_eq!(extract_pure_stub_bytes(&unins_exe).unwrap(), b"FAKE_NATIVE_WIN32_STUB_PE");

        let _ = fs::remove_dir_all(&dir);
    }
}
