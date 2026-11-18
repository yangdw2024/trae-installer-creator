use anyhow::{bail, Result};
use std::ffi::c_void;
use std::fs;
use std::path::Path;
use std::ptr::{null, null_mut};
use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{HWND, MAX_PATH};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, GetDC, GetDeviceCaps, ReleaseDC, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_PITCH, FF_DONTCARE, FW_BOLD, FW_NORMAL, HFONT, LOGPIXELSY,
    OUT_DEFAULT_PRECIS,
};
use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows_sys::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows_sys::Win32::UI::Shell::{
    SHBrowseForFolderW, SHGetPathFromIDListW, ShellExecuteW, ShellLink, BIF_NEWDIALOGSTYLE,
    BIF_RETURNONLYFSDIRS, BROWSEINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, MessageBoxW, SetWindowTextW, SW_SHOWNORMAL,
};

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

pub fn set_window_text(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    unsafe {
        SetWindowTextW(hwnd, wide.as_ptr());
    }
}

pub fn get_window_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len as usize) + 2];
        let copied = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        from_wide(&buf[..copied.max(0) as usize])
    }
}

pub fn msg_box(hwnd: HWND, title: &str, text: &str, flags: u32) -> i32 {
    let w_title = to_wide(title);
    let w_text = to_wide(text);
    unsafe { MessageBoxW(hwnd, w_text.as_ptr(), w_title.as_ptr(), flags) }
}

pub fn create_ui_font(pt_size: i32, bold: bool) -> HFONT {
    unsafe {
        let hdc = GetDC(null_mut());
        let dpi = if hdc.is_null() {
            96
        } else {
            let y = GetDeviceCaps(hdc, LOGPIXELSY as i32);
            ReleaseDC(null_mut(), hdc);
            if y > 0 { y } else { 96 }
        };
        let height = -(pt_size * dpi / 72);
        let weight = if bold { FW_BOLD } else { FW_NORMAL };
        let face = to_wide("Microsoft YaHei UI");
        CreateFontW(
            height,
            0,
            0,
            0,
            weight as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            face.as_ptr(),
        )
    }
}

pub fn pick_folder(hwnd: HWND, title: &str) -> Option<String> {
    let w_title = to_wide(title);
    unsafe {
        let _ = CoInitializeEx(null(), COINIT_APARTMENTTHREADED as u32);
        let bi = BROWSEINFOW {
            hwndOwner: hwnd,
            pidlRoot: null_mut(),
            pszDisplayName: null_mut(),
            lpszTitle: w_title.as_ptr(),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            lpfn: None,
            lParam: 0,
            iImage: 0,
        };
        let pidl = SHBrowseForFolderW(&bi);
        if pidl.is_null() {
            CoUninitialize();
            return None;
        }
        let mut path_buf = vec![0u16; MAX_PATH as usize + 4];
        let ok = SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr());
        CoTaskMemFree(pidl as *const c_void);
        CoUninitialize();
        if ok != 0 {
            let p = from_wide(&path_buf);
            if !p.is_empty() {
                return Some(p);
            }
        }
        None
    }
}

pub fn query_free_disk_mb(path_str: &str) -> Option<u64> {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let root = if bytes.len() >= 2 && bytes[1] == b':' {
        format!("{}:\\", trimmed.chars().next()?)
    } else if trimmed.starts_with(r"\\") {
        let parts: Vec<&str> = trimmed.trim_start_matches('\\').split('\\').collect();
        if parts.len() >= 2 {
            format!(r"\\{}\{}", parts[0], parts[1])
        } else {
            return None;
        }
    } else {
        return None;
    };

    let w_root = to_wide(&root);
    let mut free_avail: u64 = 0;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            w_root.as_ptr(),
            &mut free_avail,
            null_mut(),
            null_mut(),
        )
    };
    if ok != 0 {
        Some(free_avail / (1024 * 1024))
    } else {
        None
    }
}

pub fn shell_execute_open(target: &Path, working_dir: &Path) {
    let w_op = to_wide("open");
    let w_file = to_wide(&target.to_string_lossy());
    let w_dir = to_wide(&working_dir.to_string_lossy());
    unsafe {
        ShellExecuteW(
            null_mut(),
            w_op.as_ptr(),
            w_file.as_ptr(),
            null(),
            w_dir.as_ptr(),
            SW_SHOWNORMAL,
        );
    }
}

const IID_ISHELL_LINK_W: GUID = GUID::from_u128(0x000214F9_0000_0000_C000_000000000046);
const IID_IPERSIST_FILE: GUID = GUID::from_u128(0x0000010B_0000_0000_C000_000000000046);

#[repr(C)]
struct IShellLinkWVtbl {
    query_interface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    _get_path: *const c_void,
    _get_id_list: *const c_void,
    _set_id_list: *const c_void,
    _get_description: *const c_void,
    _set_description: *const c_void,
    _get_working_directory: *const c_void,
    set_working_directory: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
    _get_arguments: *const c_void,
    set_arguments: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
    _get_hotkey: *const c_void,
    _set_hotkey: *const c_void,
    _get_show_cmd: *const c_void,
    _set_show_cmd: *const c_void,
    _get_icon_location: *const c_void,
    set_icon_location: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32,
    _set_relative_path: *const c_void,
    _resolve: *const c_void,
    set_path: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
}

#[repr(C)]
struct IPersistFileVtbl {
    query_interface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    _get_class_id: *const c_void,
    _is_dirty: *const c_void,
    _load: *const c_void,
    save: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32,
    _save_completed: *const c_void,
    _get_cur_file: *const c_void,
}

pub fn create_shortcut(
    lnk_path: &Path,
    target: &str,
    args: &str,
    working_dir: &str,
    icon_file: &str,
) -> Result<()> {
    if let Some(parent) = lnk_path.parent() {
        fs::create_dir_all(parent)?;
    }

    unsafe {
        let _ = CoInitializeEx(null(), COINIT_APARTMENTTHREADED as u32);

        let mut sl_ptr: *mut c_void = null_mut();
        let hr = CoCreateInstance(
            &ShellLink,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_ISHELL_LINK_W,
            &mut sl_ptr,
        );
        if hr < 0 || sl_ptr.is_null() {
            CoUninitialize();
            bail!("CoCreateInstance(ShellLink) failed with HRESULT {:#x}", hr);
        }

        let sl_vtbl = &**(sl_ptr as *const *const IShellLinkWVtbl);

        let w_target = to_wide(target);
        (sl_vtbl.set_path)(sl_ptr, w_target.as_ptr());

        if !args.trim().is_empty() {
            let w_args = to_wide(args);
            (sl_vtbl.set_arguments)(sl_ptr, w_args.as_ptr());
        }

        let resolved_wd = if !working_dir.trim().is_empty() {
            working_dir.to_string()
        } else {
            Path::new(target)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        if !resolved_wd.is_empty() {
            let w_wd = to_wide(&resolved_wd);
            (sl_vtbl.set_working_directory)(sl_ptr, w_wd.as_ptr());
        }

        if !icon_file.trim().is_empty() {
            let w_icon = to_wide(icon_file);
            (sl_vtbl.set_icon_location)(sl_ptr, w_icon.as_ptr(), 0);
        }

        let mut pf_ptr: *mut c_void = null_mut();
        let hr_qi = (sl_vtbl.query_interface)(sl_ptr, &IID_IPERSIST_FILE, &mut pf_ptr);
        if hr_qi < 0 || pf_ptr.is_null() {
            (sl_vtbl.release)(sl_ptr);
            CoUninitialize();
            bail!("QueryInterface(IPersistFile) failed with HRESULT {:#x}", hr_qi);
        }

        let pf_vtbl = &**(pf_ptr as *const *const IPersistFileVtbl);
        let w_lnk = to_wide(&lnk_path.to_string_lossy());
        let hr_save = (pf_vtbl.save)(pf_ptr, w_lnk.as_ptr(), 1);

        (pf_vtbl.release)(pf_ptr);
        (sl_vtbl.release)(sl_ptr);
        CoUninitialize();

        if hr_save < 0 {
            bail!("IPersistFile::Save failed with HRESULT {:#x}", hr_save);
        }
    }
    Ok(())
}
