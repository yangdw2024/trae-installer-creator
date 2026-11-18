#![windows_subsystem = "windows"]

mod installer;
mod uninstaller;
mod win32_util;
mod wizard_ui;

use anyhow::Result;
use std::fs::File;
use std::path::PathBuf;
use std::ptr::null_mut;
use trae_core::{expand_constants, read_installer_payload, PackageFooter};
use win32_util::msg_box;
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONINFORMATION, MB_OK};

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    if let Err(err) = run() {
        msg_box(
            null_mut(),
            "Trae Installer 运行错误",
            &format!("{:#}", err),
            MB_OK | MB_ICONERROR,
        );
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Mode 1: Uninstaller Stage 2 (spawned from %TEMP%)
    if args.len() >= 4 && args[1] == "--unins-stage2" {
        let install_dir = PathBuf::from(&args[2]);
        let orig_unins = PathBuf::from(&args[3]);
        let silent = args.iter().any(|a| is_silent_flag(a));
        return uninstaller::run_stage2(&install_dir, &orig_unins, silent);
    }

    let silent = args.iter().any(|a| is_silent_flag(a));
    let dir_override = parse_dir_override(&args);
    let force_uninstall = args.iter().any(|a| a.eq_ignore_ascii_case("--uninstall"));

    let self_exe = std::env::current_exe()?;
    let mut file = File::open(&self_exe)?;
    let footer_opt = PackageFooter::read_from_file(&mut file)?;
    drop(file);

    let Some(footer) = footer_opt else {
        if !silent {
            msg_box(
                null_mut(),
                "Trae Native Setup Stub v3.0",
                "这是一个纯净的 Trae 原生安装包运行时底座（Native Stub）。\n请使用 TraeCC 或 Trae Studio 将其打包为单文件安装程序。",
                MB_OK | MB_ICONINFORMATION,
            );
        }
        return Ok(());
    };

    // Mode 2: Uninstaller Stage 1
    if footer.is_uninstaller() || force_uninstall {
        return uninstaller::run_stage1(&self_exe, silent);
    }

    // Mode 3: Installer
    let (footer, config) = read_installer_payload(&self_exe)?;
    if silent {
        let install_dir = dir_override.unwrap_or_else(|| {
            PathBuf::from(expand_constants(
                &config.setup.default_dir_name,
                None,
                &config.setup.app_name,
            ))
        });
        installer::perform_installation(
            &self_exe,
            &footer,
            &config,
            &install_dir,
            true,
            |_, _, _| {},
        )?;
        return Ok(());
    }

    wizard_ui::run_wizard(self_exe, footer, config)
}

fn is_silent_flag(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("/SILENT")
        || arg.eq_ignore_ascii_case("/VERYSILENT")
        || arg.eq_ignore_ascii_case("--silent")
}

fn parse_dir_override(args: &[String]) -> Option<PathBuf> {
    for (i, a) in args.iter().enumerate() {
        let lower = a.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("/dir=") {
            let _ = rest;
            let orig_val = &args[i][5..];
            let trimmed = orig_val.trim_matches('"');
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        } else if lower == "/dir" && i + 1 < args.len() {
            let trimmed = args[i + 1].trim_matches('"');
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }
    None
}
