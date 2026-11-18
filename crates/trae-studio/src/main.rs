#![windows_subsystem = "windows"]

mod win32_dlg;

use std::fs;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::thread;
use trae_core::{
    default_license_text, BuildReport, FileEntry, IconEntry, InstallerBuilder, IssParser,
    SetupScript,
};
use win32_dlg::{
    create_ui_font, get_text, lv_add_row, lv_clear, lv_delete_row, lv_get_selected_index,
    lv_insert_column, msg_box, open_file_dialog, pick_folder, save_file_dialog, set_text,
    to_wide,
};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect, InvalidateRect,
    SelectObject, SetBkMode, SetTextColor, COLOR_BTNFACE, COLOR_WINDOW, HBRUSH, HDC, HFONT,
    PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_LISTVIEW_CLASSES, ICC_STANDARD_CLASSES, ICC_TAB_CLASSES,
    INITCOMMONCONTROLSEX, LVS_EX_FULLROWSELECT, LVS_EX_GRIDLINES, LVS_REPORT, LVS_SHOWSELALWAYS,
    LVS_SINGLESEL, LVM_SETEXTENDEDLISTVIEWSTYLE, NMHDR, TCIF_TEXT, TCITEMW, TCM_GETCURSEL,
    TCM_INSERTITEMW, TCN_SELCHANGE, WC_LISTVIEWW, WC_TABCONTROLW,
};
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, IsDialogMessageW, LoadCursorW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SendMessageW, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, BS_DEFPUSHBUTTON, BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW, ES_AUTOHSCROLL,
    ES_AUTOVSCROLL, ES_MULTILINE, GWLP_USERDATA, IDC_ARROW, IDYES, MB_ICONERROR,
    MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE,
    SW_SHOW, WM_APP, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORSTATIC, WM_DESTROY, WM_NOTIFY,
    WM_PAINT, WM_SETFONT, WNDCLASSEXW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_HSCROLL,
    WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};

const ID_BTN_QUICK_BUILD: usize = 2001;
const ID_BTN_SMART_IMPORT: usize = 2002;
const ID_BTN_OPEN_SCRIPT: usize = 2003;
const ID_BTN_SAVE_SCRIPT: usize = 2004;
const ID_BTN_TEMPLATE: usize = 2005;
const ID_TAB_MAIN: usize = 2010;

// Tab 0: Setup
const ID_BTN_BROWSE_OUT: usize = 2101;

// Tab 1: Files
const ID_BTN_ADD_FILE: usize = 2201;
const ID_BTN_ADD_DIR: usize = 2202;
const ID_BTN_REM_FILE: usize = 2203;
const ID_BTN_CLR_FILES: usize = 2204;

// Tab 2: License
const ID_BTN_LOAD_LIC: usize = 2301;
const ID_BTN_RESET_LIC: usize = 2302;

// Tab 3: Icons
const ID_BTN_AUTO_ICONS: usize = 2401;
const ID_BTN_ADD_ICON: usize = 2402;
const ID_BTN_REM_ICON: usize = 2403;
const ID_BTN_CLR_ICONS: usize = 2404;

const WM_APP_BUILD_OK: u32 = WM_APP + 10;
const WM_APP_BUILD_ERR: u32 = WM_APP + 11;

struct StudioState {
    script: SetupScript,
    current_script_path: Option<PathBuf>,
    current_tab: usize,
    is_building: bool,

    font_normal: HFONT,
    font_bold: HFONT,
    font_banner: HFONT,
    font_mono: HFONT,
    white_brush: HBRUSH,
    banner_brush: HBRUSH,
    toolbar_brush: HBRUSH,

    tab_ctrl: HWND,
    tab_pages: [Vec<HWND>; 5],
    lbl_status: HWND,
    btn_build: HWND,

    // Tab 0 controls
    edit_app_name: HWND,
    edit_app_ver: HWND,
    edit_publisher: HWND,
    edit_website: HWND,
    edit_default_dir: HWND,
    edit_default_group: HWND,
    edit_unins_icon: HWND,
    edit_out_dir: HWND,
    edit_out_name: HWND,

    // Tab 1 controls
    lv_files: HWND,

    // Tab 2 controls
    edit_license: HWND,

    // Tab 3 controls
    lv_icons: HWND,
    edit_icon_name: HWND,
    edit_icon_target: HWND,

    // Tab 4 controls
    edit_iss_source: HWND,
}

fn default_desktop_dir() -> String {
    std::env::var("USERPROFILE")
        .map(|p| PathBuf::from(p).join("Desktop").to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".to_string())
}

fn default_studio_script() -> SetupScript {
    let mut s = SetupScript::default();
    s.setup.output_dir = default_desktop_dir();
    s
}

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_TAB_CLASSES | ICC_LISTVIEW_CLASSES | ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&icc);

        let hinstance = GetModuleHandleW(null());
        let cls_name = to_wide("TraeStudioMainClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: cls_name.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);

        let win_w = 940;
        let win_h = 680;
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let x = ((sw - win_w) / 2).max(0);
        let y = ((sh - win_h) / 2).max(0);

        let state = Box::new(StudioState {
            script: default_studio_script(),
            current_script_path: None,
            current_tab: 0,
            is_building: false,
            font_normal: create_ui_font(9, false, "Microsoft YaHei UI"),
            font_bold: create_ui_font(10, true, "Microsoft YaHei UI"),
            font_banner: create_ui_font(13, true, "Microsoft YaHei UI"),
            font_mono: create_ui_font(9, false, "Consolas"),
            white_brush: CreateSolidBrush(0x00FFFFFF),
            banner_brush: CreateSolidBrush(0x00D77800),
            toolbar_brush: CreateSolidBrush(0x00F4F4F4),
            tab_ctrl: null_mut(),
            tab_pages: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            lbl_status: null_mut(),
            btn_build: null_mut(),
            edit_app_name: null_mut(),
            edit_app_ver: null_mut(),
            edit_publisher: null_mut(),
            edit_website: null_mut(),
            edit_default_dir: null_mut(),
            edit_default_group: null_mut(),
            edit_unins_icon: null_mut(),
            edit_out_dir: null_mut(),
            edit_out_name: null_mut(),
            lv_files: null_mut(),
            edit_license: null_mut(),
            lv_icons: null_mut(),
            edit_icon_name: null_mut(),
            edit_icon_target: null_mut(),
            edit_iss_source: null_mut(),
        });
        let state_ptr = Box::into_raw(state);

        let title = to_wide("Trae Studio v3.0 — 原生单文件安装包制作工具 (Rust)");
        let hwnd = CreateWindowExW(
            0,
            cls_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            x,
            y,
            win_w,
            win_h,
            null_mut(),
            null_mut(),
            hinstance,
            state_ptr as *const _,
        );
        if hwnd.is_null() {
            let _ = Box::from_raw(state_ptr);
            return;
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let cs = &*(lparam as *const windows_sys::Win32::UI::WindowsAndMessaging::CREATESTRUCTW);
        let state_ptr = cs.lpCreateParams as *mut StudioState;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
        build_ui(hwnd, &mut *state_ptr);
        sync_model_to_ui(&*state_ptr);
        switch_tab(hwnd, &mut *state_ptr, 0);
        return 0;
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut StudioState;
    if state_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let state = &mut *state_ptr;

    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            paint_chrome(hdc, state);
            EndPaint(hwnd, &ps);
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            state.white_brush as isize
        }
        WM_NOTIFY => {
            let nmhdr = &*(lparam as *const NMHDR);
            if nmhdr.idFrom == ID_TAB_MAIN && nmhdr.code == TCN_SELCHANGE {
                let new_tab = SendMessageW(state.tab_ctrl, TCM_GETCURSEL, 0, 0) as usize;
                on_tab_change(hwnd, state, new_tab);
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as usize;
            handle_command(hwnd, state, id);
            0
        }
        WM_APP_BUILD_OK => {
            let report = Box::from_raw(lparam as *mut BuildReport);
            state.is_building = false;
            EnableWindow(state.btn_build, 1);
            set_text(
                state.lbl_status,
                &format!("构建成功：{}", report.output_path.display()),
            );
            msg_box(
                hwnd,
                "生成成功",
                &format!(
                    "已生成独立单文件安装包！\n\n\
                     输出路径：{}\n\
                     打包文件数：{}\n\
                     原始体积：{:.2} MB\n\
                     压缩后载荷：{:.2} MB\n\
                     最终单文件 EXE 大小：{:.2} MB",
                    report.output_path.display(),
                    report.file_count,
                    report.uncompressed_size as f64 / (1024.0 * 1024.0),
                    report.compressed_size as f64 / (1024.0 * 1024.0),
                    report.total_exe_size as f64 / (1024.0 * 1024.0),
                ),
                MB_OK | MB_ICONINFORMATION,
            );
            0
        }
        WM_APP_BUILD_ERR => {
            let err_str = Box::from_raw(lparam as *mut String);
            state.is_building = false;
            EnableWindow(state.btn_build, 1);
            set_text(state.lbl_status, "构建失败");
            msg_box(hwnd, "生成失败", &err_str, MB_OK | MB_ICONERROR);
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            DeleteObject(state.font_normal as _);
            DeleteObject(state.font_bold as _);
            DeleteObject(state.font_banner as _);
            DeleteObject(state.font_mono as _);
            DeleteObject(state.white_brush as _);
            DeleteObject(state.banner_brush as _);
            DeleteObject(state.toolbar_brush as _);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(state_ptr));
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_chrome(hdc: HDC, state: &StudioState) {
    let banner = RECT {
        left: 0,
        top: 0,
        right: 940,
        bottom: 54,
    };
    FillRect(hdc, &banner, state.banner_brush);

    let toolbar = RECT {
        left: 0,
        top: 54,
        right: 940,
        bottom: 102,
    };
    FillRect(hdc, &toolbar, state.toolbar_brush);

    let status_bg = RECT {
        left: 0,
        top: 610,
        right: 940,
        bottom: 650,
    };
    FillRect(hdc, &status_bg, (COLOR_BTNFACE + 1) as HBRUSH);

    let old_font = SelectObject(hdc, state.font_banner as _);
    SetBkMode(hdc, TRANSPARENT as i32);
    SetTextColor(hdc, 0x00FFFFFF);

    let title = to_wide("Trae Studio v3.0 — 原生单文件安装包制作工具");
    let mut tr = RECT {
        left: 20,
        top: 14,
        right: 700,
        bottom: 48,
    };
    DrawTextW(hdc, title.as_ptr(), -1, &mut tr, 0);

    SelectObject(hdc, state.font_normal as _);
    let sub = to_wide("Rust 原生引擎 · 极小单文件 Stub · 流式高压缩");
    let mut sr = RECT {
        left: 610,
        top: 18,
        right: 920,
        bottom: 48,
    };
    DrawTextW(hdc, sub.as_ptr(), -1, &mut sr, 0);
    SelectObject(hdc, old_font);
}

unsafe fn ctrl(
    parent: HWND,
    class: &str,
    text: &str,
    style: u32,
    ex_style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: usize,
    font: HFONT,
) -> HWND {
    let wc = to_wide(class);
    let wt = to_wide(text);
    let h_ctrl = CreateWindowExW(
        ex_style,
        wc.as_ptr(),
        wt.as_ptr(),
        style,
        x,
        y,
        w,
        h,
        parent,
        id as _,
        GetModuleHandleW(null()),
        null(),
    );
    if !h_ctrl.is_null() && !font.is_null() {
        SendMessageW(h_ctrl, WM_SETFONT, font as usize, 1);
    }
    h_ctrl
}

unsafe fn build_ui(hwnd: HWND, state: &mut StudioState) {
    // Quick Toolbar
    state.btn_build = ctrl(
        hwnd,
        "BUTTON",
        "一键生成单文件安装包",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON as u32,
        0,
        16,
        64,
        175,
        30,
        ID_BTN_QUICK_BUILD,
        state.font_bold,
    );
    ctrl(
        hwnd,
        "BUTTON",
        "智能导入软件文件夹",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        204,
        64,
        160,
        30,
        ID_BTN_SMART_IMPORT,
        state.font_normal,
    );
    ctrl(
        hwnd,
        "BUTTON",
        "打开 .iss 脚本",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        378,
        64,
        118,
        30,
        ID_BTN_OPEN_SCRIPT,
        state.font_normal,
    );
    ctrl(
        hwnd,
        "BUTTON",
        "保存 .iss 脚本",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        508,
        64,
        118,
        30,
        ID_BTN_SAVE_SCRIPT,
        state.font_normal,
    );
    ctrl(
        hwnd,
        "BUTTON",
        "恢复默认模板",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        638,
        64,
        118,
        30,
        ID_BTN_TEMPLATE,
        state.font_normal,
    );

    // Tab Control
    state.tab_ctrl = CreateWindowExW(
        0,
        WC_TABCONTROLW,
        null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        14,
        112,
        896,
        490,
        hwnd,
        ID_TAB_MAIN as _,
        GetModuleHandleW(null()),
        null(),
    );
    SendMessageW(state.tab_ctrl, WM_SETFONT, state.font_normal as usize, 1);

    for (idx, title) in ["基本设置", "文件管理", "许可协议", "快捷方式", "ISS 脚本（双向同步）"]
        .iter()
        .enumerate()
    {
        let mut wt = to_wide(title);
        let mut tci: TCITEMW = std::mem::zeroed();
        tci.mask = TCIF_TEXT;
        tci.pszText = wt.as_mut_ptr();
        SendMessageW(state.tab_ctrl, TCM_INSERTITEMW, idx, &tci as *const _ as isize);
    }

    // Status label
    state.lbl_status = ctrl(
        hwnd,
        "STATIC",
        "就绪 — Rust 原生核心已加载",
        WS_CHILD | WS_VISIBLE,
        0,
        16,
        618,
        890,
        22,
        0,
        state.font_normal,
    );

    // --- Tab 0: Setup ---
    let mut row_y = 158;
    let add_field = |label: &str, y: i32, w: i32| -> (HWND, HWND) {
        let l = ctrl(
            hwnd,
            "STATIC",
            label,
            WS_CHILD,
            0,
            34,
            y + 4,
            135,
            22,
            0,
            state.font_normal,
        );
        let e = ctrl(
            hwnd,
            "EDIT",
            "",
            WS_CHILD | WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32,
            0,
            175,
            y,
            w,
            26,
            0,
            state.font_normal,
        );
        (l, e)
    };

    let (l1, e1) = add_field("软件名称：", row_y, 440);
    state.edit_app_name = e1;
    row_y += 40;
    let (l2, e2) = add_field("软件版本：", row_y, 180);
    state.edit_app_ver = e2;
    row_y += 40;
    let (l3, e3) = add_field("发布者：", row_y, 440);
    state.edit_publisher = e3;
    row_y += 40;
    let (l4, e4) = add_field("官方网站：", row_y, 440);
    state.edit_website = e4;
    row_y += 46;
    let (l5, e5) = add_field("默认安装目录：", row_y, 520);
    state.edit_default_dir = e5;
    row_y += 40;
    let (l6, e6) = add_field("开始菜单组：", row_y, 440);
    state.edit_default_group = e6;
    row_y += 40;
    let (l7, e7) = add_field("卸载程序图标：", row_y, 520);
    state.edit_unins_icon = e7;
    row_y += 46;
    let (l8, e8) = add_field("输出目录：", row_y, 520);
    state.edit_out_dir = e8;
    let b_out = ctrl(
        hwnd,
        "BUTTON",
        "浏览...",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        706,
        row_y - 1,
        92,
        28,
        ID_BTN_BROWSE_OUT,
        state.font_normal,
    );
    row_y += 40;
    let (l9, e9) = add_field("输出文件名：", row_y, 440);
    state.edit_out_name = e9;
    let l_hint = ctrl(
        hwnd,
        "STATIC",
        "(留空则按“软件名称 Setup 版本号.exe”自动命名)",
        WS_CHILD,
        0,
        626,
        row_y + 4,
        270,
        22,
        0,
        state.font_normal,
    );

    state.tab_pages[0] = vec![
        l1, e1, l2, e2, l3, e3, l4, e4, l5, e5, l6, e6, l7, e7, l8, e8, b_out, l9, e9, l_hint,
    ];

    // --- Tab 1: Files ---
    state.lv_files = CreateWindowExW(
        0,
        WC_LISTVIEWW,
        null(),
        WS_CHILD | WS_BORDER | WS_TABSTOP | LVS_REPORT | LVS_SHOWSELALWAYS | LVS_SINGLESEL,
        30,
        152,
        864,
        388,
        hwnd,
        null_mut(),
        GetModuleHandleW(null()),
        null(),
    );
    SendMessageW(state.lv_files, WM_SETFONT, state.font_normal as usize, 1);
    SendMessageW(
        state.lv_files,
        LVM_SETEXTENDEDLISTVIEWSTYLE,
        0,
        (LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES) as isize,
    );
    lv_insert_column(state.lv_files, 0, "源文件 / 文件夹", 450);
    lv_insert_column(state.lv_files, 1, "安装目标目录", 210);
    lv_insert_column(state.lv_files, 2, "重命名", 180);

    let b_af = ctrl(
        hwnd,
        "BUTTON",
        "添加文件",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        30,
        552,
        112,
        30,
        ID_BTN_ADD_FILE,
        state.font_normal,
    );
    let b_ad = ctrl(
        hwnd,
        "BUTTON",
        "添加文件夹",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        154,
        552,
        112,
        30,
        ID_BTN_ADD_DIR,
        state.font_normal,
    );
    let b_rf = ctrl(
        hwnd,
        "BUTTON",
        "删除选中",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        278,
        552,
        112,
        30,
        ID_BTN_REM_FILE,
        state.font_normal,
    );
    let b_cf = ctrl(
        hwnd,
        "BUTTON",
        "清空全部",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        402,
        552,
        112,
        30,
        ID_BTN_CLR_FILES,
        state.font_normal,
    );
    state.tab_pages[1] = vec![state.lv_files, b_af, b_ad, b_rf, b_cf];

    // --- Tab 2: License ---
    let b_ll = ctrl(
        hwnd,
        "BUTTON",
        "从文本文件加载协议...",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        30,
        152,
        165,
        28,
        ID_BTN_LOAD_LIC,
        state.font_normal,
    );
    let b_rl = ctrl(
        hwnd,
        "BUTTON",
        "恢复默认中文协议",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        208,
        152,
        145,
        28,
        ID_BTN_RESET_LIC,
        state.font_normal,
    );
    state.edit_license = ctrl(
        hwnd,
        "EDIT",
        "",
        WS_CHILD
            | WS_BORDER
            | WS_VSCROLL
            | WS_TABSTOP
            | ES_MULTILINE as u32
            | ES_AUTOVSCROLL as u32,
        0,
        30,
        190,
        864,
        396,
        0,
        state.font_mono,
    );
    state.tab_pages[2] = vec![b_ll, b_rl, state.edit_license];

    // --- Tab 3: Icons ---
    state.lv_icons = CreateWindowExW(
        0,
        WC_LISTVIEWW,
        null(),
        WS_CHILD | WS_BORDER | WS_TABSTOP | LVS_REPORT | LVS_SHOWSELALWAYS | LVS_SINGLESEL,
        30,
        152,
        864,
        340,
        hwnd,
        null_mut(),
        GetModuleHandleW(null()),
        null(),
    );
    SendMessageW(state.lv_icons, WM_SETFONT, state.font_normal as usize, 1);
    SendMessageW(
        state.lv_icons,
        LVM_SETEXTENDEDLISTVIEWSTYLE,
        0,
        (LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES) as isize,
    );
    lv_insert_column(state.lv_icons, 0, "快捷方式名称与位置", 330);
    lv_insert_column(state.lv_icons, 1, "目标程序", 330);
    lv_insert_column(state.lv_icons, 2, "参数", 180);

    let l_in = ctrl(
        hwnd,
        "STATIC",
        "名称：",
        WS_CHILD,
        0,
        30,
        508,
        48,
        22,
        0,
        state.font_normal,
    );
    state.edit_icon_name = ctrl(
        hwnd,
        "EDIT",
        r"{group}\我的程序",
        WS_CHILD | WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32,
        0,
        80,
        504,
        260,
        26,
        0,
        state.font_normal,
    );
    let l_it = ctrl(
        hwnd,
        "STATIC",
        "目标：",
        WS_CHILD,
        0,
        352,
        508,
        48,
        22,
        0,
        state.font_normal,
    );
    state.edit_icon_target = ctrl(
        hwnd,
        "EDIT",
        r"{app}\MyProg.exe",
        WS_CHILD | WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32,
        0,
        402,
        504,
        320,
        26,
        0,
        state.font_normal,
    );
    let b_addi = ctrl(
        hwnd,
        "BUTTON",
        "添加快捷方式",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        734,
        503,
        160,
        28,
        ID_BTN_ADD_ICON,
        state.font_normal,
    );

    let b_autoi = ctrl(
        hwnd,
        "BUTTON",
        "一键生成开始菜单+桌面快捷方式",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        30,
        552,
        245,
        30,
        ID_BTN_AUTO_ICONS,
        state.font_normal,
    );
    let b_remi = ctrl(
        hwnd,
        "BUTTON",
        "删除选中",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        288,
        552,
        112,
        30,
        ID_BTN_REM_ICON,
        state.font_normal,
    );
    let b_clri = ctrl(
        hwnd,
        "BUTTON",
        "清空全部",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        412,
        552,
        112,
        30,
        ID_BTN_CLR_ICONS,
        state.font_normal,
    );
    state.tab_pages[3] = vec![
        state.lv_icons,
        l_in,
        state.edit_icon_name,
        l_it,
        state.edit_icon_target,
        b_addi,
        b_autoi,
        b_remi,
        b_clri,
    ];

    // --- Tab 4: ISS Script ---
    let l_iss = ctrl(
        hwnd,
        "STATIC",
        "下方显示完整的 Inno Setup (.iss) 脚本源码。直接编辑后切回其他标签页或点击保存/生成，将自动同步到界面：",
        WS_CHILD,
        0,
        30,
        152,
        864,
        22,
        0,
        state.font_normal,
    );
    state.edit_iss_source = ctrl(
        hwnd,
        "EDIT",
        "",
        WS_CHILD
            | WS_BORDER
            | WS_VSCROLL
            | WS_HSCROLL
            | WS_TABSTOP
            | ES_MULTILINE as u32
            | ES_AUTOVSCROLL as u32
            | ES_AUTOHSCROLL as u32,
        0,
        30,
        180,
        864,
        406,
        0,
        state.font_mono,
    );
    state.tab_pages[4] = vec![l_iss, state.edit_iss_source];
}

unsafe fn switch_tab(hwnd: HWND, state: &mut StudioState, tab: usize) {
    state.current_tab = tab;
    for (idx, ctrls) in state.tab_pages.iter().enumerate() {
        let cmd = if idx == tab { SW_SHOW } else { SW_HIDE };
        for &c in ctrls {
            ShowWindow(c, cmd);
        }
    }
    InvalidateRect(hwnd, null(), 1);
}

unsafe fn on_tab_change(hwnd: HWND, state: &mut StudioState, new_tab: usize) {
    if state.current_tab == 4 && new_tab != 4 {
        if !sync_script_tab_to_model(hwnd, state) {
            return;
        }
        sync_model_to_ui(state);
    } else if state.current_tab != 4 && new_tab == 4 {
        sync_ui_to_model(state);
        set_text(state.edit_iss_source, &state.script.to_iss_string());
    }
    switch_tab(hwnd, state, new_tab);
}

unsafe fn sync_script_tab_to_model(hwnd: HWND, state: &mut StudioState) -> bool {
    let raw = get_text(state.edit_iss_source);
    if raw.trim().is_empty() {
        return true;
    }
    match IssParser::parse_str(&raw) {
        Ok(parsed) => {
            state.script = parsed;
            true
        }
        Err(err) => {
            msg_box(
                hwnd,
                "脚本解析错误",
                &format!("ISS 脚本语法错误：\n{:#}", err),
                MB_OK | MB_ICONERROR,
            );
            false
        }
    }
}

unsafe fn sync_ui_to_model(state: &mut StudioState) {
    state.script.setup.app_name = get_text(state.edit_app_name).trim().to_string();
    state.script.setup.app_version = get_text(state.edit_app_ver).trim().to_string();
    state.script.setup.app_publisher = get_text(state.edit_publisher).trim().to_string();
    state.script.setup.app_publisher_url = get_text(state.edit_website).trim().to_string();
    state.script.setup.default_dir_name = get_text(state.edit_default_dir).trim().to_string();
    state.script.setup.default_group_name = get_text(state.edit_default_group).trim().to_string();
    state.script.setup.uninstall_display_icon = get_text(state.edit_unins_icon).trim().to_string();
    state.script.setup.output_dir = get_text(state.edit_out_dir).trim().to_string();
    state.script.setup.output_base_filename = get_text(state.edit_out_name).trim().to_string();
    state.script.setup.license_text = get_text(state.edit_license)
        .replace("\r\n", "\n")
        .trim_end()
        .to_string();
}

unsafe fn sync_model_to_ui(state: &StudioState) {
    set_text(state.edit_app_name, &state.script.setup.app_name);
    set_text(state.edit_app_ver, &state.script.setup.app_version);
    set_text(state.edit_publisher, &state.script.setup.app_publisher);
    set_text(state.edit_website, &state.script.setup.app_publisher_url);
    set_text(state.edit_default_dir, &state.script.setup.default_dir_name);
    set_text(state.edit_default_group, &state.script.setup.default_group_name);
    set_text(state.edit_unins_icon, &state.script.setup.uninstall_display_icon);
    set_text(state.edit_out_dir, &state.script.setup.output_dir);
    set_text(state.edit_out_name, &state.script.setup.output_base_filename);

    let lic_crlf = state
        .script
        .setup
        .license_text
        .replace("\r\n", "\n")
        .replace('\n', "\r\n");
    set_text(state.edit_license, &lic_crlf);

    refresh_files_listview(state);
    refresh_icons_listview(state);
    set_text(state.edit_iss_source, &state.script.to_iss_string());
}

unsafe fn refresh_files_listview(state: &StudioState) {
    lv_clear(state.lv_files);
    for (idx, f) in state.script.files.iter().enumerate() {
        lv_add_row(state.lv_files, idx as i32, &[&f.source, &f.dest_dir, &f.dest_name]);
    }
}

unsafe fn refresh_icons_listview(state: &StudioState) {
    lv_clear(state.lv_icons);
    for (idx, ic) in state.script.icons.iter().enumerate() {
        lv_add_row(state.lv_icons, idx as i32, &[&ic.name, &ic.filename, &ic.parameters]);
    }
}

unsafe fn handle_command(hwnd: HWND, state: &mut StudioState, id: usize) {
    match id {
        ID_BTN_QUICK_BUILD => start_build(hwnd, state),
        ID_BTN_SMART_IMPORT => smart_import_folder(hwnd, state),
        ID_BTN_OPEN_SCRIPT => open_script(hwnd, state),
        ID_BTN_SAVE_SCRIPT => save_script(hwnd, state),
        ID_BTN_TEMPLATE => {
            state.script = default_studio_script();
            state.current_script_path = None;
            sync_model_to_ui(state);
            set_text(state.lbl_status, "已恢复默认模板");
        }
        ID_BTN_BROWSE_OUT => {
            if let Some(dir) = pick_folder(hwnd, "选择安装包输出目录") {
                set_text(state.edit_out_dir, &dir);
            }
        }
        ID_BTN_ADD_FILE => {
            if let Some(path) = open_file_dialog(hwnd, "选择要打包的文件", &[("所有文件", "*.*")]) {
                state.script.files.push(FileEntry {
                    source: path,
                    dest_dir: "{app}".to_string(),
                    ..Default::default()
                });
                refresh_files_listview(state);
            }
        }
        ID_BTN_ADD_DIR => {
            if let Some(dir) = pick_folder(hwnd, "选择要打包的文件夹") {
                state.script.files.push(FileEntry {
                    source: dir,
                    dest_dir: "{app}".to_string(),
                    ..Default::default()
                });
                refresh_files_listview(state);
            }
        }
        ID_BTN_REM_FILE => {
            if let Some(idx) = lv_get_selected_index(state.lv_files) {
                if idx < state.script.files.len() {
                    state.script.files.remove(idx);
                    lv_delete_row(state.lv_files, idx);
                }
            }
        }
        ID_BTN_CLR_FILES => {
            if msg_box(hwnd, "确认", "确定清空所有文件吗？", MB_YESNO | MB_ICONQUESTION) == IDYES {
                state.script.files.clear();
                lv_clear(state.lv_files);
            }
        }
        ID_BTN_LOAD_LIC => {
            if let Some(path) = open_file_dialog(
                hwnd,
                "选择许可协议文本文件",
                &[("文本文件", "*.txt"), ("所有文件", "*.*")],
            ) {
                if let Ok(content) = fs::read_to_string(&path) {
                    state.script.setup.license_text = content.clone();
                    set_text(
                        state.edit_license,
                        &content.replace("\r\n", "\n").replace('\n', "\r\n"),
                    );
                }
            }
        }
        ID_BTN_RESET_LIC => {
            let def = default_license_text();
            state.script.setup.license_text = def.clone();
            set_text(state.edit_license, &def.replace('\n', "\r\n"));
        }
        ID_BTN_ADD_ICON => {
            let name = get_text(state.edit_icon_name).trim().to_string();
            let target = get_text(state.edit_icon_target).trim().to_string();
            if !name.is_empty() && !target.is_empty() {
                state.script.icons.push(IconEntry {
                    name,
                    filename: target,
                    ..Default::default()
                });
                refresh_icons_listview(state);
            }
        }
        ID_BTN_AUTO_ICONS => {
            sync_ui_to_model(state);
            let app_name = state.script.setup.app_name.clone();
            let exe_target = detect_main_exe_target(&state.script.files)
                .unwrap_or_else(|| r"{app}\MyProg.exe".to_string());
            state.script.icons.push(IconEntry {
                name: format!(r"{{group}}\{}", app_name),
                filename: exe_target.clone(),
                ..Default::default()
            });
            state.script.icons.push(IconEntry {
                name: format!(r"{{autodesktop}}\{}", app_name),
                filename: exe_target,
                ..Default::default()
            });
            refresh_icons_listview(state);
        }
        ID_BTN_REM_ICON => {
            if let Some(idx) = lv_get_selected_index(state.lv_icons) {
                if idx < state.script.icons.len() {
                    state.script.icons.remove(idx);
                    lv_delete_row(state.lv_icons, idx);
                }
            }
        }
        ID_BTN_CLR_ICONS => {
            state.script.icons.clear();
            lv_clear(state.lv_icons);
        }
        _ => {}
    }
}

unsafe fn smart_import_folder(hwnd: HWND, state: &mut StudioState) {
    let Some(folder) = pick_folder(hwnd, "选择您的软件文件夹") else {
        return;
    };
    sync_ui_to_model(state);

    let folder_path = PathBuf::from(&folder);
    let folder_name = folder_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("我的程序")
        .to_string();

    state.script.files.clear();
    state.script.files.push(FileEntry {
        source: folder.clone(),
        dest_dir: "{app}".to_string(),
        ..Default::default()
    });

    if state.script.setup.app_name.trim().is_empty() || state.script.setup.app_name == "我的程序" {
        state.script.setup.app_name = folder_name.clone();
        state.script.setup.default_group_name = folder_name.clone();
        state.script.setup.default_dir_name = format!(r"{{autopf}}\{}", folder_name);
    }

    let mut main_exe: Option<String> = None;
    if let Ok(entries) = fs::read_dir(&folder_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    let lower = name.to_ascii_lowercase();
                    if lower.ends_with(".exe") && !lower.starts_with("unins") {
                        main_exe = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    if let Some(ref exe_name) = main_exe {
        let app_name = state.script.setup.app_name.clone();
        let target = format!(r"{{app}}\{}", exe_name);
        state.script.setup.uninstall_display_icon = target.clone();
        state.script.icons.clear();
        state.script.icons.push(IconEntry {
            name: format!(r"{{group}}\{}", app_name),
            filename: target.clone(),
            ..Default::default()
        });
        state.script.icons.push(IconEntry {
            name: format!(r"{{autodesktop}}\{}", app_name),
            filename: target,
            ..Default::default()
        });
    }

    sync_model_to_ui(state);
    set_text(
        state.lbl_status,
        &format!(
            "已导入软件文件夹：{}（主程序：{}）",
            folder,
            main_exe.as_deref().unwrap_or("未检测到")
        ),
    );
}

unsafe fn open_script(hwnd: HWND, state: &mut StudioState) {
    let Some(path) = open_file_dialog(
        hwnd,
        "打开安装脚本",
        &[("Inno 安装脚本", "*.iss"), ("所有文件", "*.*")],
    ) else {
        return;
    };
    let pb = PathBuf::from(&path);
    match IssParser::parse_file(&pb) {
        Ok(parsed) => {
            state.script = parsed;
            state.current_script_path = Some(pb);
            sync_model_to_ui(state);
            set_text(state.lbl_status, &format!("已加载脚本：{}", path));
        }
        Err(err) => {
            msg_box(
                hwnd,
                "打开失败",
                &format!("无法加载脚本：\n{:#}", err),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}

unsafe fn save_script(hwnd: HWND, state: &mut StudioState) {
    if state.current_tab == 4 {
        if !sync_script_tab_to_model(hwnd, state) {
            return;
        }
        sync_model_to_ui(state);
    } else {
        sync_ui_to_model(state);
    }

    let target_path = if let Some(ref p) = state.current_script_path {
        p.clone()
    } else if let Some(chosen) = save_file_dialog(
        hwnd,
        "保存安装脚本",
        "iss",
        &[("Inno 安装脚本", "*.iss"), ("所有文件", "*.*")],
    ) {
        PathBuf::from(chosen)
    } else {
        return;
    };

    let iss_text = state.script.to_iss_string();
    if let Err(err) = fs::write(&target_path, &iss_text) {
        msg_box(
            hwnd,
            "保存失败",
            &format!("写入脚本失败：\n{}", err),
            MB_OK | MB_ICONERROR,
        );
        return;
    }

    state.current_script_path = Some(target_path.clone());
    set_text(state.edit_iss_source, &iss_text);
    set_text(
        state.lbl_status,
        &format!("已保存脚本：{}", target_path.display()),
    );
}

unsafe fn start_build(hwnd: HWND, state: &mut StudioState) {
    if state.is_building {
        return;
    }

    if state.current_tab == 4 {
        if !sync_script_tab_to_model(hwnd, state) {
            return;
        }
        sync_model_to_ui(state);
    } else {
        sync_ui_to_model(state);
    }

    if state.script.setup.app_name.trim().is_empty() {
        msg_box(hwnd, "提示", "请先填写软件名称！", MB_OK | MB_ICONERROR);
        return;
    }
    if state.script.files.is_empty() {
        if msg_box(
            hwnd,
            "未添加文件",
            "尚未添加任何文件，是否现在选择一个软件文件夹？",
            MB_YESNO | MB_ICONQUESTION,
        ) == IDYES
        {
            smart_import_folder(hwnd, state);
        }
        return;
    }

    state.is_building = true;
    EnableWindow(state.btn_build, 0);
    set_text(state.lbl_status, "正在流式压缩并构建单文件安装包...");

    let script = state.script.clone();
    let base_dir = state
        .current_script_path
        .as_ref()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let hwnd_val = hwnd as isize;

    thread::spawn(move || {
        let target_hwnd = hwnd_val as HWND;
        match InstallerBuilder::build(&script, &base_dir, None, None) {
            Ok(report) => {
                PostMessageW(
                    target_hwnd,
                    WM_APP_BUILD_OK,
                    0,
                    Box::into_raw(Box::new(report)) as isize,
                );
            }
            Err(err) => {
                PostMessageW(
                    target_hwnd,
                    WM_APP_BUILD_ERR,
                    0,
                    Box::into_raw(Box::new(format!("{:#}", err))) as isize,
                );
            }
        }
    });
}

fn detect_main_exe_target(files: &[FileEntry]) -> Option<String> {
    for f in files {
        let p = Path::new(&f.source);
        if p.is_file() {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.to_ascii_lowercase().ends_with(".exe") {
                    return Some(format!(r"{{app}}\{}", name));
                }
            }
        } else if p.is_dir() {
            if let Ok(entries) = fs::read_dir(p) {
                for e in entries.flatten() {
                    let ep = e.path();
                    if ep.is_file() {
                        if let Some(name) = ep.file_name().and_then(|s| s.to_str()) {
                            let lower = name.to_ascii_lowercase();
                            if lower.ends_with(".exe") && !lower.starts_with("unins") {
                                return Some(format!(r"{{app}}\{}", name));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
