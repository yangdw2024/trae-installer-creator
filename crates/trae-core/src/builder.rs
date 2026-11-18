use crate::archive::pack_solid_archive;
use crate::footer::{
    extract_pure_stub_bytes, PackageFooter, MAGIC_INSTALLER, MAGIC_UNINSTALLER,
};
use crate::model::{InstallerPayloadConfig, SetupScript, UninstallManifest};
use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct BuildReport {
    pub output_path: PathBuf,
    pub stub_path: PathBuf,
    pub file_count: usize,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub total_exe_size: u64,
}

pub struct InstallerBuilder;

impl InstallerBuilder {
    pub fn find_stub_path(explicit: Option<&Path>) -> Result<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(p) = explicit {
            candidates.push(p.to_path_buf());
        }
        for stub_name in ["forge-stub.exe", "trae-stub.exe"] {
            if let Ok(curr_exe) = std::env::current_exe() {
                if let Some(exe_dir) = curr_exe.parent() {
                    candidates.push(exe_dir.join(stub_name));
                    candidates.push(exe_dir.join("runtime").join(stub_name));
                    candidates.push(exe_dir.join("release").join(stub_name));
                }
            }
            if let Ok(cwd) = std::env::current_dir() {
                candidates.push(cwd.join(stub_name));
                candidates.push(cwd.join("target").join("release").join(stub_name));
                candidates.push(cwd.join("target").join("debug").join(stub_name));
                candidates.push(cwd.join("release").join(stub_name));
            }
            let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..");
            candidates.push(manifest_root.join("target").join("release").join(stub_name));
            candidates.push(manifest_root.join("target").join("debug").join(stub_name));
            candidates.push(manifest_root.join("release").join(stub_name));
        }

        for c in &candidates {
            if c.is_file() {
                return Ok(c.clone());
            }
        }
        bail!(
            "Could not locate trae-stub.exe. Searched: {:?}",
            candidates
        );
    }

    pub fn resolve_output_path(
        script: &SetupScript,
        base_dir: &Path,
        explicit_output_path: Option<&Path>,
    ) -> PathBuf {
        if let Some(p) = explicit_output_path {
            return p.to_path_buf();
        }
        let out_dir_raw = if script.setup.output_dir.trim().is_empty() {
            "."
        } else {
            script.setup.output_dir.trim()
        };
        let out_dir_path = Path::new(out_dir_raw);
        let resolved_dir = if out_dir_path.is_absolute() {
            out_dir_path.to_path_buf()
        } else {
            base_dir.join(out_dir_path)
        };
        let base_name = if !script.setup.output_base_filename.trim().is_empty() {
            script.setup.output_base_filename.trim().to_string()
        } else {
            format!("{} Setup {}", script.setup.app_name, script.setup.app_version)
        };
        let file_name = if base_name.to_ascii_lowercase().ends_with(".exe") {
            base_name
        } else {
            format!("{}.exe", base_name)
        };
        resolved_dir.join(file_name)
    }

    pub fn build(
        script: &SetupScript,
        base_dir: &Path,
        explicit_output_path: Option<&Path>,
        explicit_stub_path: Option<&Path>,
    ) -> Result<BuildReport> {
        let stub_path = Self::find_stub_path(explicit_stub_path)?;
        let pure_stub = extract_pure_stub_bytes(&stub_path)
            .with_context(|| format!("Failed to read stub from {:?}", stub_path))?;

        let output_path = Self::resolve_output_path(script, base_dir, explicit_output_path);
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let unique_tag = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_archive_path = std::env::temp_dir().join(format!(
            "trae_pkg_{}_{}.zlib",
            std::process::id(),
            unique_tag
        ));

        let build_res = (|| -> Result<BuildReport> {
            let (files_meta, total_uncompressed_size) = {
                let temp_file = File::create(&temp_archive_path)?;
                pack_solid_archive(&script.files, base_dir, temp_file)?
            };

            let archive_size = fs::metadata(&temp_archive_path)?.len();
            let payload_config = InstallerPayloadConfig {
                setup: script.setup.clone(),
                icons: script.icons.clone(),
                registry: script.registry.clone(),
                run: script.run.clone(),
                files_meta: files_meta.clone(),
                total_uncompressed_size,
            };
            let config_json = serde_json::to_vec(&payload_config)?;

            let config_offset = pure_stub.len() as u64;
            let config_size = config_json.len() as u64;
            let footer =
                PackageFooter::new(config_offset, config_size, archive_size, MAGIC_INSTALLER);

            {
                let mut out_file = BufWriter::new(File::create(&output_path)?);
                out_file.write_all(&pure_stub)?;
                out_file.write_all(&config_json)?;

                let mut temp_read = File::open(&temp_archive_path)?;
                io::copy(&mut temp_read, &mut out_file)?;
                out_file.write_all(&footer.to_bytes())?;
                out_file.flush()?;
            }

            let total_exe_size = fs::metadata(&output_path)?.len();
            Ok(BuildReport {
                output_path,
                stub_path,
                file_count: files_meta.len(),
                uncompressed_size: total_uncompressed_size,
                compressed_size: archive_size,
                total_exe_size,
            })
        })();

        let _ = fs::remove_file(&temp_archive_path);
        build_res
    }
}

pub fn read_installer_payload(exe_path: &Path) -> Result<(PackageFooter, InstallerPayloadConfig)> {
    let mut file = File::open(exe_path)?;
    let Some(footer) = PackageFooter::read_from_file(&mut file)? else {
        bail!("Executable {:?} does not contain a Trae package footer", exe_path);
    };
    if !footer.is_installer() {
        bail!("Executable {:?} is not an installer package", exe_path);
    }

    file.seek(SeekFrom::Start(footer.config_offset))?;
    let mut config_bytes = vec![0u8; usize::try_from(footer.config_size)?];
    file.read_exact(&mut config_bytes)?;
    let config: InstallerPayloadConfig = serde_json::from_slice(&config_bytes)?;
    Ok((footer, config))
}

pub fn read_uninstaller_manifest(exe_path: &Path) -> Result<(PackageFooter, UninstallManifest)> {
    let mut file = File::open(exe_path)?;
    let Some(footer) = PackageFooter::read_from_file(&mut file)? else {
        bail!("Executable {:?} does not contain a Trae uninstaller footer", exe_path);
    };
    if !footer.is_uninstaller() {
        bail!("Executable {:?} is not an uninstaller package", exe_path);
    }

    file.seek(SeekFrom::Start(footer.config_offset))?;
    let mut config_bytes = vec![0u8; usize::try_from(footer.config_size)?];
    file.read_exact(&mut config_bytes)?;
    let manifest: UninstallManifest = serde_json::from_slice(&config_bytes)?;
    Ok((footer, manifest))
}

pub fn write_uninstaller_exe(
    current_exe: &Path,
    target_unins_path: &Path,
    manifest: &UninstallManifest,
) -> Result<()> {
    let pure_stub = extract_pure_stub_bytes(current_exe)?;
    let manifest_json = serde_json::to_vec(manifest)?;

    if let Some(parent) = target_unins_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let footer = PackageFooter::new(
        pure_stub.len() as u64,
        manifest_json.len() as u64,
        0,
        MAGIC_UNINSTALLER,
    );

    let mut out = BufWriter::new(File::create(target_unins_path)?);
    out.write_all(&pure_stub)?;
    out.write_all(&manifest_json)?;
    out.write_all(&footer.to_bytes())?;
    out.flush()?;
    Ok(())
}
