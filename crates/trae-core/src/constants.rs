use std::path::PathBuf;

/// Expand Inno Setup style path constants case-insensitively.
pub fn expand_constants(input: &str, app_dir: Option<&str>, app_name: &str) -> String {
    if input.is_empty() || !input.contains('{') {
        return input.to_string();
    }

    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
    let pf32 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());
    let localappdata = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
    let userappdata = std::env::var("APPDATA").unwrap_or_else(|_| localappdata.clone());
    let commonappdata =
        std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());
    let tmp = std::env::temp_dir().to_string_lossy().into_owned();
    let tmp_trimmed = tmp.trim_end_matches(['\\', '/']).to_string();

    let start_menu_programs = PathBuf::from(&userappdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join(app_name)
        .to_string_lossy()
        .into_owned();

    let desktop = resolve_desktop_dir();

    let mut out = input.to_string();
    if let Some(app) = app_dir {
        out = replace_ci(&out, "{app}", app.trim_end_matches(['\\', '/']));
    }
    out = replace_ci(&out, "{autopf}", &pf);
    out = replace_ci(&out, "{pf}", &pf);
    out = replace_ci(&out, "{commonpf}", &pf);
    out = replace_ci(&out, "{pf32}", &pf32);
    out = replace_ci(&out, "{commonpf32}", &pf32);
    out = replace_ci(&out, "{localappdata}", &localappdata);
    out = replace_ci(&out, "{userappdata}", &userappdata);
    out = replace_ci(&out, "{commonappdata}", &commonappdata);
    out = replace_ci(&out, "{tmp}", &tmp_trimmed);
    out = replace_ci(&out, "{group}", &start_menu_programs);
    out = replace_ci(&out, "{autodesktop}", &desktop);
    out = replace_ci(&out, "{userdesktop}", &desktop);
    out
}

pub fn resolve_desktop_dir() -> String {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let primary = PathBuf::from(&profile).join("Desktop");
        if primary.exists() {
            return primary.to_string_lossy().into_owned();
        }
        let onedrive = PathBuf::from(&profile).join("OneDrive").join("Desktop");
        if onedrive.exists() {
            return onedrive.to_string_lossy().into_owned();
        }
        return primary.to_string_lossy().into_owned();
    }
    if let Ok(pub_dir) = std::env::var("PUBLIC") {
        return PathBuf::from(pub_dir)
            .join("Desktop")
            .to_string_lossy()
            .into_owned();
    }
    r"C:\Users\Public\Desktop".to_string()
}

pub fn resolve_start_menu_group_dir(app_name: &str) -> String {
    let appdata = std::env::var("APPDATA")
        .or_else(|_| std::env::var("LOCALAPPDATA"))
        .unwrap_or_else(|_| r"C:\Users\Default\AppData\Roaming".to_string());
    PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join(app_name)
        .to_string_lossy()
        .into_owned()
}

fn replace_ci(haystack: &str, needle: &str, replacement: &str) -> String {
    let lower_haystack = haystack.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    let mut result = String::with_capacity(haystack.len());
    let mut cursor = 0;

    while let Some(rel_idx) = lower_haystack[cursor..].find(&lower_needle) {
        let abs_idx = cursor + rel_idx;
        if abs_idx < haystack.len() && abs_idx + needle.len() <= haystack.len() {
            result.push_str(&haystack[cursor..abs_idx]);
            result.push_str(replacement);
            cursor = abs_idx + needle.len();
        } else {
            break;
        }
    }
    result.push_str(&haystack[cursor..]);
    result
}
