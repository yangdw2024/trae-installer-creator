use std::ffi::c_void;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HWND, MAX_PATH};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, GetDC, GetDeviceCaps, ReleaseDC, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_PITCH, FF_DONTCARE, FW_BOLD, FW_NORMAL, HFONT, LOGPIXELSY,
    OUT_DEFAULT_PRECIS,
};
use windows_sys::Win32::System::Com::{
    CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_APARTMENTTHREADED,
};
use windows_sys::Win32::UI::Controls::{
    LVCFMT_LEFT, LVCF_FMT, LVCF_TEXT, LVCF_WIDTH, LVCOLUMNW, LVIF_TEXT, LVITEMW,
    LVM_DELETEALLITEMS, LVM_DELETEITEM, LVM_GETNEXTITEM, LVM_INSERTCOLUMNW, LVM_INSERTITEMW,
    LVM_SETITEMTEXTW, LVNI_SELECTED,
};
use windows_sys::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_FILEMUSTEXIST, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
    OPENFILENAMEW,
};
use windows_sys::Win32::UI::Shell::{
    SHBrowseForFolderW, SHGetPathFromIDListW, BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, MessageBoxW, SendMessageW, SetWindowTextW,
};

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

pub fn set_text(hwnd: HWND, text: &str) {
    let w = to_wide(text);
    unsafe {
        SetWindowTextW(hwnd, w.as_ptr());
    }
}

pub fn get_text(hwnd: HWND) -> String {
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
    let wt = to_wide(title);
    let wb = to_wide(text);
    unsafe { MessageBoxW(hwnd, wb.as_ptr(), wt.as_ptr(), flags) }
}

pub fn create_ui_font(pt_size: i32, bold: bool, face_name: &str) -> HFONT {
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
        let face = to_wide(face_name);
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

pub fn open_file_dialog(hwnd: HWND, title: &str, filter_pairs: &[(&str, &str)]) -> Option<String> {
    let mut filter_wide = Vec::new();
    for (desc, pat) in filter_pairs {
        filter_wide.extend(desc.encode_utf16());
        filter_wide.push(0);
        filter_wide.extend(pat.encode_utf16());
        filter_wide.push(0);
    }
    filter_wide.push(0);

    let mut file_buf = vec![0u16; 1024];
    let w_title = to_wide(title);

    unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = hwnd;
        ofn.lpstrFilter = filter_wide.as_ptr();
        ofn.lpstrFile = file_buf.as_mut_ptr();
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.lpstrTitle = w_title.as_ptr();
        ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;
        if GetOpenFileNameW(&mut ofn) != 0 {
            let res = from_wide(&file_buf);
            if !res.is_empty() {
                return Some(res);
            }
        }
    }
    None
}

pub fn save_file_dialog(
    hwnd: HWND,
    title: &str,
    default_ext: &str,
    filter_pairs: &[(&str, &str)],
) -> Option<String> {
    let mut filter_wide = Vec::new();
    for (desc, pat) in filter_pairs {
        filter_wide.extend(desc.encode_utf16());
        filter_wide.push(0);
        filter_wide.extend(pat.encode_utf16());
        filter_wide.push(0);
    }
    filter_wide.push(0);

    let mut file_buf = vec![0u16; 1024];
    let w_title = to_wide(title);
    let w_ext = to_wide(default_ext);

    unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = hwnd;
        ofn.lpstrFilter = filter_wide.as_ptr();
        ofn.lpstrFile = file_buf.as_mut_ptr();
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.lpstrTitle = w_title.as_ptr();
        ofn.lpstrDefExt = w_ext.as_ptr();
        ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;
        if GetSaveFileNameW(&mut ofn) != 0 {
            let res = from_wide(&file_buf);
            if !res.is_empty() {
                return Some(res);
            }
        }
    }
    None
}

pub unsafe fn lv_insert_column(lv: HWND, index: i32, text: &str, width: i32) {
    let mut w_txt = to_wide(text);
    let mut col: LVCOLUMNW = std::mem::zeroed();
    col.mask = LVCF_FMT | LVCF_WIDTH | LVCF_TEXT;
    col.fmt = LVCFMT_LEFT;
    col.cx = width;
    col.pszText = w_txt.as_mut_ptr();
    SendMessageW(lv, LVM_INSERTCOLUMNW, index as usize, &col as *const _ as isize);
}

pub unsafe fn lv_add_row(lv: HWND, row_idx: i32, cols: &[&str]) {
    if cols.is_empty() {
        return;
    }
    let mut first = to_wide(cols[0]);
    let mut item: LVITEMW = std::mem::zeroed();
    item.mask = LVIF_TEXT;
    item.iItem = row_idx;
    item.iSubItem = 0;
    item.pszText = first.as_mut_ptr();
    let inserted = SendMessageW(lv, LVM_INSERTITEMW, 0, &item as *const _ as isize) as i32;
    if inserted < 0 {
        return;
    }
    for (sub_idx, val) in cols.iter().enumerate().skip(1) {
        let mut w_sub = to_wide(val);
        let mut sub_item: LVITEMW = std::mem::zeroed();
        sub_item.iSubItem = sub_idx as i32;
        sub_item.pszText = w_sub.as_mut_ptr();
        SendMessageW(
            lv,
            LVM_SETITEMTEXTW,
            inserted as usize,
            &sub_item as *const _ as isize,
        );
    }
}

pub unsafe fn lv_get_selected_index(lv: HWND) -> Option<usize> {
    let idx = SendMessageW(lv, LVM_GETNEXTITEM, usize::MAX, LVNI_SELECTED as isize) as i32;
    if idx >= 0 {
        Some(idx as usize)
    } else {
        None
    }
}

pub unsafe fn lv_delete_row(lv: HWND, index: usize) {
    SendMessageW(lv, LVM_DELETEITEM, index, 0);
}

pub unsafe fn lv_clear(lv: HWND) {
    SendMessageW(lv, LVM_DELETEALLITEMS, 0, 0);
}
