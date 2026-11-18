use crate::win32_util::create_shortcut;
use anyhow::Result;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use trae_core::{
    expand_constants, extract_solid_archive, resolve_desktop_dir, resolve_start_menu_group_dir,
    write_uninstaller_exe, IconEntry, InstallerPayloadConfig, PackageFooter, UninstallManifest,
};
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
use winreg::RegKey;

#[derive(Debug, Clone)]
pub struct InstallOutcome {
    pub install_dir: PathBuf,
    pub installed_files: Vec<String>,
    pub shortcuts: Vec<String>,
    pub uninstaller_path: PathBuf,
    pub registry_subkey: String,
    pub primary_exe: Option<PathBuf>,
}

pub fn perform_installation<F>(
    self_exe: &Path,
    footer: &PackageFooter,
    config: &InstallerPayloadConfig,
    install_dir: &Path,
    create_desktop_immediately: bool,
    mut on_progress: F,
) -> Result<InstallOutcome>
where
    F: FnMut(usize, usize, &str),
{
    let mut exe_file = File::open(self_exe)?;
    exe_file.seek(SeekFrom::Start(footer.archive_offset()))?;
    let archive_reader = exe_file.take(footer.archive_size);

    let installed_files =
        extract_solid_archive(archive_reader, &config.files_meta, install_dir, |c, t, n| {
            on_progress(c, t, n);
        })?;

    let install_dir_str = install_dir.to_string_lossy().into_owned();
    let app_name = &config.setup.app_name;

    let mut shortcuts = create_non_desktop_icons(
        &config.icons,
        &install_dir_str,
        app_name,
    );

    let primary_exe = find_primary_exe(config, install_dir, &installed_files);

    if create_desktop_immediately {
        let desk = create_desktop_icons(
            &config.icons,
            &install_dir_str,
            app_name,
            primary_exe.as_deref(),
        );
        shortcuts.extend(desk);
    }

    let registry_subkey = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}",
        app_name
    );
    let uninstaller_path = install_dir.join("unins000.exe");

    let manifest = build_manifest(
        config,
        &install_dir_str,
        &installed_files,
        &shortcuts,
        &registry_subkey,
    );

    write_uninstaller_exe(self_exe, &uninstaller_path, &manifest)?;
    register_uninstaller_in_registry(
        config,
        &install_dir_str,
        &uninstaller_path,
        &registry_subkey,
    )?;
    apply_custom_registry_entries(config, &install_dir_str);

    Ok(InstallOutcome {
        install_dir: install_dir.to_path_buf(),
        installed_files,
        shortcuts,
        uninstaller_path,
        registry_subkey,
        primary_exe,
    })
}

pub fn update_desktop_shortcuts_and_manifest(
    self_exe: &Path,
    config: &InstallerPayloadConfig,
    outcome: &mut InstallOutcome,
) -> Result<()> {
    let install_dir_str = outcome.install_dir.to_string_lossy().into_owned();
    let app_name = &config.setup.app_name;
    let desk = create_desktop_icons(
        &config.icons,
        &install_dir_str,
        app_name,
        outcome.primary_exe.as_deref(),
    );
    if !desk.is_empty() {
        outcome.shortcuts.extend(desk);
        let manifest = build_manifest(
            config,
            &install_dir_str,
            &outcome.installed_files,
            &outcome.shortcuts,
            &outcome.registry_subkey,
        );
        write_uninstaller_exe(self_exe, &outcome.uninstaller_path, &manifest)?;
    }
    Ok(())
}

fn build_manifest(
    config: &InstallerPayloadConfig,
    install_dir_str: &str,
    installed_files: &[String],
    shortcuts: &[String],
    registry_subkey: &str,
) -> UninstallManifest {
    UninstallManifest {
        app_name: config.setup.app_name.clone(),
        app_version: config.setup.app_version.clone(),
        publisher: config.setup.app_publisher.clone(),
        install_dir: install_dir_str.to_string(),
        installed_files: installed_files.to_vec(),
        shortcuts: shortcuts.to_vec(),
        registry_subkey: registry_subkey.to_string(),
    }
}

fn create_non_desktop_icons(
    icons: &[IconEntry],
    install_dir: &str,
    app_name: &str,
) -> Vec<String> {
    let mut created = Vec::new();
    let group_dir = resolve_start_menu_group_dir(app_name);

    for icon in icons {
        let lower_name = icon.name.to_ascii_lowercase();
        if lower_name.contains("{autodesktop}") || lower_name.contains("{userdesktop}") {
            continue;
        }
        let target = expand_constants(&icon.filename, Some(install_dir), app_name);
        if target.trim().is_empty() {
            continue;
        }
        let args = expand_constants(&icon.parameters, Some(install_dir), app_name);
        let wd = expand_constants(&icon.working_dir, Some(install_dir), app_name);
        let ico = expand_constants(&icon.icon_file, Some(install_dir), app_name);

        let lnk_path = if lower_name.contains("{group}") {
            let expanded = expand_constants(&icon.name, Some(install_dir), app_name);
            let with_ext = if expanded.to_ascii_lowercase().ends_with(".lnk") {
                expanded
            } else {
                format!("{}.lnk", expanded)
            };
            PathBuf::from(with_ext)
        } else {
            let base = Path::new(&icon.name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(app_name);
            PathBuf::from(&group_dir).join(format!("{}.lnk", base))
        };

        if create_shortcut(&lnk_path, &target, &args, &wd, &ico).is_ok() {
            created.push(lnk_path.to_string_lossy().into_owned());
        }
    }

    created
}

fn create_desktop_icons(
    icons: &[IconEntry],
    install_dir: &str,
    app_name: &str,
    primary_exe: Option<&Path>,
) -> Vec<String> {
    let mut created = Vec::new();
    let desktop_dir = resolve_desktop_dir();

    let desktop_entries: Vec<&IconEntry> = icons
        .iter()
        .filter(|i| {
            let l = i.name.to_ascii_lowercase();
            l.contains("{autodesktop}") || l.contains("{userdesktop}")
        })
        .collect();

    if !desktop_entries.is_empty() {
        for icon in desktop_entries {
            let target = expand_constants(&icon.filename, Some(install_dir), app_name);
            if target.trim().is_empty() {
                continue;
            }
            let args = expand_constants(&icon.parameters, Some(install_dir), app_name);
            let wd = expand_constants(&icon.working_dir, Some(install_dir), app_name);
            let ico = expand_constants(&icon.icon_file, Some(install_dir), app_name);

            let expanded = expand_constants(&icon.name, Some(install_dir), app_name);
            let lnk_str = if expanded.to_ascii_lowercase().ends_with(".lnk") {
                expanded
            } else {
                format!("{}.lnk", expanded)
            };
            let lnk_path = PathBuf::from(lnk_str);
            if create_shortcut(&lnk_path, &target, &args, &wd, &ico).is_ok() {
                created.push(lnk_path.to_string_lossy().into_owned());
            }
        }
    } else if let Some(exe_path) = primary_exe {
        let lnk_path = PathBuf::from(&desktop_dir).join(format!("{}.lnk", app_name));
        let target = exe_path.to_string_lossy().into_owned();
        if create_shortcut(&lnk_path, &target, "", "", "").is_ok() {
            created.push(lnk_path.to_string_lossy().into_owned());
        }
    }

    created
}

pub fn find_primary_exe(
    config: &InstallerPayloadConfig,
    install_dir: &Path,
    installed_files: &[String],
) -> Option<PathBuf> {
    for rel in installed_files {
        let lower = rel.to_ascii_lowercase();
        let file_name = Path::new(rel)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if lower.ends_with(".exe") && !file_name.starts_with("unins") {
            let candidate = install_dir.join(rel);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let _ = config;
    None
}

fn register_uninstaller_in_registry(
    config: &InstallerPayloadConfig,
    install_dir: &str,
    uninstaller_path: &Path,
    registry_subkey: &str,
) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey_with_flags(registry_subkey, KEY_WRITE)?;
    key.set_value("DisplayName", &config.setup.app_name)?;
    key.set_value("DisplayVersion", &config.setup.app_version)?;
    key.set_value("Publisher", &config.setup.app_publisher)?;
    key.set_value("InstallLocation", &install_dir)?;
    let unins_cmd = format!("\"{}\"", uninstaller_path.display());
    key.set_value("UninstallString", &unins_cmd)?;
    key.set_value("NoModify", &1u32)?;
    key.set_value("NoRepair", &1u32)?;

    if !config.setup.uninstall_display_icon.trim().is_empty() {
        let icon_expanded = expand_constants(
            &config.setup.uninstall_display_icon,
            Some(install_dir),
            &config.setup.app_name,
        );
        let _ = key.set_value("DisplayIcon", &icon_expanded);
    }
    Ok(())
}

fn apply_custom_registry_entries(config: &InstallerPayloadConfig, install_dir: &str) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for entry in &config.registry {
        if !entry.root.eq_ignore_ascii_case("HKCU") && !entry.root.eq_ignore_ascii_case("HKA") {
            continue;
        }
        let subkey = expand_constants(&entry.subkey, Some(install_dir), &config.setup.app_name);
        if let Ok((key, _)) = hkcu.create_subkey_with_flags(&subkey, KEY_WRITE) {
            let name =
                expand_constants(&entry.value_name, Some(install_dir), &config.setup.app_name);
            let data =
                expand_constants(&entry.value_data, Some(install_dir), &config.setup.app_name);
            if entry.value_type.eq_ignore_ascii_case("dword") {
                if let Ok(num) = data.parse::<u32>() {
                    let _ = key.set_value(&name, &num);
                }
            } else {
                let _ = key.set_value(&name, &data);
            }
        }
    }
}
