use crate::win32_util::msg_box;
use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::null_mut;
use std::thread;
use std::time::Duration;
use trae_core::{read_uninstaller_manifest, resolve_start_menu_group_dir};
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO,
};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

pub fn run_stage1(self_exe: &Path, silent: bool) -> Result<()> {
    let (_footer, manifest) = read_uninstaller_manifest(self_exe)?;

    if !silent {
        let title = format!("卸载 {}", manifest.app_name);
        let prompt = format!(
            "确定要彻底卸载 {} (版本 {}) 吗？\n\n安装位置：{}",
            manifest.app_name, manifest.app_version, manifest.install_dir
        );
        let choice = msg_box(null_mut(), &title, &prompt, MB_YESNO | MB_ICONQUESTION);
        if choice != IDYES {
            return Ok(());
        }
    }

    let temp_unins = std::env::temp_dir().join(format!(
        "trae_unins_{}.exe",
        std::process::id()
    ));
    fs::copy(self_exe, &temp_unins)?;

    let mut cmd = Command::new(&temp_unins);
    cmd.arg("--unins-stage2")
        .arg(&manifest.install_dir)
        .arg(self_exe);
    if silent {
        cmd.arg("--silent");
    }
    cmd.spawn()?;
    Ok(())
}

pub fn run_stage2(install_dir: &Path, orig_unins_exe: &Path, silent: bool) -> Result<()> {
    // Wait up to 4 seconds until Stage 1 exits and unlocks orig_unins_exe
    for _ in 0..80 {
        if !orig_unins_exe.exists() {
            break;
        }
        if OpenOptions::new().write(true).open(orig_unins_exe).is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let manifest_opt = if orig_unins_exe.is_file() {
        read_uninstaller_manifest(orig_unins_exe).ok().map(|(_, m)| m)
    } else {
        None
    };

    let app_name = manifest_opt
        .as_ref()
        .map(|m| m.app_name.clone())
        .unwrap_or_else(|| "程序".to_string());

    if let Some(ref manifest) = manifest_opt {
        for rel_file in &manifest.installed_files {
            let p = install_dir.join(rel_file);
            if p.is_file() {
                let _ = fs::remove_file(&p);
            }
        }
        for sc in &manifest.shortcuts {
            let p = Path::new(sc);
            if p.is_file() {
                let _ = fs::remove_file(p);
            }
        }
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(&manifest.registry_subkey);
    }

    if orig_unins_exe.is_file() {
        for _ in 0..20 {
            if fs::remove_file(orig_unins_exe).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    // Remove Start Menu group folder if empty
    let group_dir = PathBuf::from(resolve_start_menu_group_dir(&app_name));
    if group_dir.is_dir() {
        let _ = fs::remove_dir(&group_dir);
    }

    // Clean empty subdirectories bottom-up and install_dir itself
    remove_empty_dirs_bottom_up(install_dir);
    let _ = fs::remove_dir(install_dir);

    if !silent {
        let title = format!("卸载 {}", app_name);
        let done_msg = format!("{} 已从您的计算机成功卸载。", app_name);
        msg_box(null_mut(), &title, &done_msg, MB_OK | MB_ICONINFORMATION);
    }

    // Schedule detached self-deletion of the temporary uninstaller executable
    if let Ok(self_temp) = std::env::current_exe() {
        let cmd_str = format!(
            "ping 127.0.0.1 -n 2 > NUL & del /F /Q \"{}\"",
            self_temp.display()
        );
        let _ = Command::new("cmd.exe")
            .args(["/C", &cmd_str])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    Ok(())
}

fn remove_empty_dirs_bottom_up(dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                remove_empty_dirs_bottom_up(&path);
                let _ = fs::remove_dir(&path);
            }
        }
    }
}
