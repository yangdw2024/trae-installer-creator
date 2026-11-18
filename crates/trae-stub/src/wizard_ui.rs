use crate::installer::{perform_installation, update_desktop_shortcuts_and_manifest, InstallOutcome};
use crate::win32_util::{
    create_ui_font, get_window_text, msg_box, pick_folder, query_free_disk_mb, set_window_text,
    shell_execute_open, to_wide,
};
use anyhow::Result;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::thread;
use trae_core::{expand_constants, InstallerPayloadConfig, PackageFooter};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, SetBkMode, SetTextColor,
    COLOR_BTNFACE, COLOR_WINDOW, HBRUSH, HDC, HFONT, PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_PROGRESS_CLASS, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX,
    PBM_SETPOS, PBM_SETRANGE32, PROGRESS_CLASSW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, IsDialogMessageW, LoadCursorW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SendMessageW, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON,
    BS_DEFPUSHBUTTON, BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW, EN_CHANGE, ES_AUTOHSCROLL,
    ES_AUTOVSCROLL, ES_MULTILINE, ES_READONLY, GWLP_USERDATA, IDC_ARROW, IDYES, MB_ICONERROR,
    MB_ICONQUESTION, MB_OK, MB_YESNO, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_SHOW, WM_APP,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORSTATIC, WM_DESTROY, WM_PAINT, WM_SETFONT,
    WNDCLASSEXW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_GROUP, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};

const ID_BTN_BACK: usize = 1001;
const ID_BTN_NEXT: usize = 1002;
const ID_BTN_CANCEL: usize = 1003;
const ID_RADIO_ACCEPT: usize = 1101;
const ID_RADIO_DECLINE: usize = 1102;
const ID_EDIT_DIR: usize = 1201;
const ID_BTN_BROWSE: usize = 1202;
const ID_CHK_DESKTOP: usize = 1401;
const ID_CHK_LAUNCH: usize = 1402;

const WM_APP_PROGRESS: u32 = WM_APP + 1;
const WM_APP_DONE: u32 = WM_APP + 2;
const WM_APP_ERROR: u32 = WM_APP + 3;

const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;

struct ProgressMsg {
    curr: usize,
    total: usize,
    rel_path: String,
}

struct WizardState {
    self_exe: PathBuf,
    footer: PackageFooter,
    config: InstallerPayloadConfig,
    current_page: usize,
    font_normal: HFONT,
    font_title: HFONT,
    font_banner: HFONT,
    font_mono: HFONT,
    white_brush: HBRUSH,
    banner_brush: HBRUSH,

    // Controls per page
    page_controls: [Vec<HWND>; 5],

    // Shared navigation
    btn_back: HWND,
    btn_next: HWND,
    btn_cancel: HWND,

    // Page 1 (License)
    radio_accept: HWND,
    radio_decline: HWND,

    // Page 2 (Dir)
    edit_dir: HWND,
    lbl_space: HWND,

    // Page 3 (Progress)
    progress_bar: HWND,
    lbl_status: HWND,
    edit_log: HWND,

    // Page 4 (Finish)
    chk_desktop: HWND,
    chk_launch: HWND,

    outcome: Option<InstallOutcome>,
}

pub fn run_wizard(
    self_exe: PathBuf,
    footer: PackageFooter,
    config: InstallerPayloadConfig,
) -> Result<()> {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_PROGRESS_CLASS | ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&icc);

        let hinstance = GetModuleHandleW(null());
        let class_name = to_wide("TraeSetupWizardClass");

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
            lpszClassName: class_name.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);

        let win_w = 620;
        let win_h = 460;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = ((screen_w - win_w) / 2).max(0);
        let y = ((screen_h - win_h) / 2).max(0);

        let state = Box::new(WizardState {
            self_exe,
            footer,
            config: config.clone(),
            current_page: 0,
            font_normal: create_ui_font(9, false),
            font_title: create_ui_font(12, true),
            font_banner: create_ui_font(13, true),
            font_mono: create_ui_font(9, false),
            white_brush: CreateSolidBrush(0x00FFFFFF),
            banner_brush: CreateSolidBrush(0x00D77800), // BGR for #0078D7
            page_controls: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            btn_back: null_mut(),
            btn_next: null_mut(),
            btn_cancel: null_mut(),
            radio_accept: null_mut(),
            radio_decline: null_mut(),
            edit_dir: null_mut(),
            lbl_space: null_mut(),
            progress_bar: null_mut(),
            lbl_status: null_mut(),
            edit_log: null_mut(),
            chk_desktop: null_mut(),
            chk_launch: null_mut(),
            outcome: None,
        });

        let state_ptr = Box::into_raw(state);
        let title_str = format!("{} 安装程序", config.setup.app_name);
        let w_title = to_wide(&title_str);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            w_title.as_ptr(),
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
            anyhow::bail!("CreateWindowExW failed");
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
    Ok(())
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let cs = &*(lparam as *const windows_sys::Win32::UI::WindowsAndMessaging::CREATESTRUCTW);
        let state_ptr = cs.lpCreateParams as *mut WizardState;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
        init_controls(hwnd, &mut *state_ptr);
        show_page(hwnd, &mut *state_ptr, 0);
        return 0;
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WizardState;
    if state_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let state = &mut *state_ptr;

    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc: HDC = BeginPaint(hwnd, &mut ps);
            paint_banner(hdc, state);
            EndPaint(hwnd, &ps);
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            state.white_brush as isize
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as usize;
            let code = ((wparam >> 16) & 0xFFFF) as u32;
            handle_command(hwnd, state, id, code);
            0
        }
        WM_APP_PROGRESS => {
            let pmsg = Box::from_raw(lparam as *mut ProgressMsg);
            let pct = if pmsg.total > 0 {
                ((pmsg.curr * 100) / pmsg.total) as usize
            } else {
                100
            };
            SendMessageW(state.progress_bar, PBM_SETPOS, pct, 0);
            set_window_text(state.lbl_status, &format!("正在安装：{}", pmsg.rel_path));
            append_log(state.edit_log, &format!("已解压：{}\r\n", pmsg.rel_path));
            0
        }
        WM_APP_DONE => {
            let outcome = Box::from_raw(lparam as *mut InstallOutcome);
            state.outcome = Some(*outcome);
            SendMessageW(state.progress_bar, PBM_SETPOS, 100, 0);
            set_window_text(state.lbl_status, "安装完成！");
            show_page(hwnd, state, 4);
            0
        }
        WM_APP_ERROR => {
            let err_box = Box::from_raw(lparam as *mut String);
            msg_box(
                hwnd,
                "安装失败",
                &format!("安装过程中发生错误：\n{}", *err_box),
                MB_OK | MB_ICONERROR,
            );
            DestroyWindow(hwnd);
            0
        }
        WM_CLOSE => {
            if state.current_page == 3 {
                return 0;
            }
            if state.current_page == 4 {
                DestroyWindow(hwnd);
                return 0;
            }
            if msg_box(
                hwnd,
                "取消安装",
                "确定要退出安装程序吗？",
                MB_YESNO | MB_ICONQUESTION,
            ) == IDYES
            {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            DeleteObject(state.font_normal as _);
            DeleteObject(state.font_title as _);
            DeleteObject(state.font_banner as _);
            DeleteObject(state.font_mono as _);
            DeleteObject(state.white_brush as _);
            DeleteObject(state.banner_brush as _);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(state_ptr));
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_banner(hdc: HDC, state: &WizardState) {
    let banner_rect = RECT {
        left: 0,
        top: 0,
        right: 620,
        bottom: 64,
    };
    FillRect(hdc, &banner_rect, state.banner_brush);

    let footer_rect = RECT {
        left: 0,
        top: 372,
        right: 620,
        bottom: 430,
    };
    FillRect(hdc, &footer_rect, (COLOR_BTNFACE + 1) as HBRUSH);

    let sep_rect = RECT {
        left: 0,
        top: 371,
        right: 620,
        bottom: 372,
    };
    let sep_brush = CreateSolidBrush(0x00D0D0D0);
    FillRect(hdc, &sep_rect, sep_brush);
    DeleteObject(sep_brush as _);

    let page_title = match state.current_page {
        0 => "欢迎使用",
        1 => "最终用户许可协议",
        2 => "选择安装位置",
        3 => "正在安装",
        _ => "安装完成",
    };

    let old_font = windows_sys::Win32::Graphics::Gdi::SelectObject(hdc, state.font_banner as _);
    SetBkMode(hdc, TRANSPARENT as i32);
    SetTextColor(hdc, 0x00FFFFFF);

    let title_line = format!("{}  —  {}", state.config.setup.app_name, page_title);
    let w_title = to_wide(&title_line);
    let mut text_rect = RECT {
        left: 22,
        top: 18,
        right: 590,
        bottom: 54,
    };
    windows_sys::Win32::Graphics::Gdi::DrawTextW(
        hdc,
        w_title.as_ptr(),
        -1,
        &mut text_rect,
        0,
    );
    windows_sys::Win32::Graphics::Gdi::SelectObject(hdc, old_font);
}

unsafe fn create_ctrl(
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
    let w_cls = to_wide(class);
    let w_txt = to_wide(text);
    let ctrl = CreateWindowExW(
        ex_style,
        w_cls.as_ptr(),
        w_txt.as_ptr(),
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
    if !ctrl.is_null() && !font.is_null() {
        SendMessageW(ctrl, WM_SETFONT, font as usize, 1);
    }
    ctrl
}

unsafe fn init_controls(hwnd: HWND, state: &mut WizardState) {
    let app_name = state.config.setup.app_name.clone();
    let app_ver = state.config.setup.app_version.clone();
    let publisher = state.config.setup.app_publisher.clone();

    // Bottom Navigation Buttons
    state.btn_back = create_ctrl(
        hwnd,
        "BUTTON",
        "< 上一步",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        20,
        384,
        92,
        28,
        ID_BTN_BACK,
        state.font_normal,
    );
    state.btn_next = create_ctrl(
        hwnd,
        "BUTTON",
        "下一步 >",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON as u32,
        0,
        396,
        384,
        96,
        28,
        ID_BTN_NEXT,
        state.font_normal,
    );
    state.btn_cancel = create_ctrl(
        hwnd,
        "BUTTON",
        "取消",
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        502,
        384,
        92,
        28,
        ID_BTN_CANCEL,
        state.font_normal,
    );

    // Page 0: Welcome
    let p0_h1 = create_ctrl(
        hwnd,
        "STATIC",
        &format!("欢迎使用 {} 安装向导", app_name),
        WS_CHILD,
        0,
        28,
        92,
        540,
        28,
        0,
        state.font_title,
    );
    let p0_ver = create_ctrl(
        hwnd,
        "STATIC",
        &format!("软件版本：{}    发布者：{}", app_ver, publisher),
        WS_CHILD,
        0,
        28,
        130,
        540,
        22,
        0,
        state.font_normal,
    );
    let p0_desc = create_ctrl(
        hwnd,
        "STATIC",
        &format!(
            "本向导将引导您在计算机上安装 {}。\n\n\
             共包含 {} 个文件（约 {:.2} MB）。\n\n\
             点击“下一步”继续，或点击“取消”退出安装。",
            app_name,
            state.config.files_meta.len(),
            (state.config.total_uncompressed_size as f64) / (1024.0 * 1024.0)
        ),
        WS_CHILD,
        0,
        28,
        172,
        540,
        140,
        0,
        state.font_normal,
    );
    state.page_controls[0] = vec![p0_h1, p0_ver, p0_desc];

    // Page 1: License
    let p1_lbl = create_ctrl(
        hwnd,
        "STATIC",
        "请仔细阅读以下许可协议。您必须接受此协议才能继续安装：",
        WS_CHILD,
        0,
        24,
        82,
        556,
        22,
        0,
        state.font_normal,
    );
    let license_crlf = state
        .config
        .setup
        .license_text
        .replace("\r\n", "\n")
        .replace('\n', "\r\n");
    let p1_edit = create_ctrl(
        hwnd,
        "EDIT",
        &license_crlf,
        WS_CHILD
            | WS_BORDER
            | WS_VSCROLL
            | WS_TABSTOP
            | ES_MULTILINE as u32
            | ES_READONLY as u32
            | ES_AUTOVSCROLL as u32,
        0,
        24,
        108,
        556,
        192,
        0,
        state.font_mono,
    );
    state.radio_accept = create_ctrl(
        hwnd,
        "BUTTON",
        "我接受协议(&A)",
        WS_CHILD | WS_GROUP | WS_TABSTOP | BS_AUTORADIOBUTTON as u32,
        0,
        24,
        310,
        240,
        22,
        ID_RADIO_ACCEPT,
        state.font_normal,
    );
    state.radio_decline = create_ctrl(
        hwnd,
        "BUTTON",
        "我不接受协议(&D)",
        WS_CHILD | WS_TABSTOP | BS_AUTORADIOBUTTON as u32,
        0,
        24,
        336,
        240,
        22,
        ID_RADIO_DECLINE,
        state.font_normal,
    );
    SendMessageW(state.radio_decline, BM_SETCHECK, BST_CHECKED, 0);
    state.page_controls[1] = vec![p1_lbl, p1_edit, state.radio_accept, state.radio_decline];

    // Page 2: Directory
    let p2_lbl = create_ctrl(
        hwnd,
        "STATIC",
        &format!("安装程序将把 {} 安装到以下文件夹：", app_name),
        WS_CHILD,
        0,
        24,
        92,
        556,
        24,
        0,
        state.font_normal,
    );
    let default_dir = expand_constants(&state.config.setup.default_dir_name, None, &app_name);
    state.edit_dir = create_ctrl(
        hwnd,
        "EDIT",
        &default_dir,
        WS_CHILD | WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32,
        0,
        24,
        124,
        452,
        26,
        ID_EDIT_DIR,
        state.font_normal,
    );
    let p2_browse = create_ctrl(
        hwnd,
        "BUTTON",
        "浏览...",
        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
        0,
        486,
        123,
        94,
        28,
        ID_BTN_BROWSE,
        state.font_normal,
    );
    let p2_hint = create_ctrl(
        hwnd,
        "STATIC",
        "要继续，请点击“安装”。如需安装在其他磁盘，请点击“浏览...”或直接修改路径。",
        WS_CHILD,
        0,
        24,
        168,
        556,
        42,
        0,
        state.font_normal,
    );
    state.lbl_space = create_ctrl(
        hwnd,
        "STATIC",
        "",
        WS_CHILD,
        0,
        24,
        226,
        556,
        24,
        0,
        state.font_normal,
    );
    refresh_disk_space(state);
    state.page_controls[2] = vec![p2_lbl, state.edit_dir, p2_browse, p2_hint, state.lbl_space];

    // Page 3: Progress
    state.lbl_status = create_ctrl(
        hwnd,
        "STATIC",
        "正在准备安装...",
        WS_CHILD,
        0,
        24,
        88,
        556,
        22,
        0,
        state.font_normal,
    );
    state.progress_bar = create_ctrl(
        hwnd,
        &crate::win32_util::from_wide(std::slice::from_raw_parts(
            PROGRESS_CLASSW,
            17,
        )),
        "",
        WS_CHILD,
        0,
        24,
        116,
        556,
        24,
        0,
        state.font_normal,
    );
    SendMessageW(state.progress_bar, PBM_SETRANGE32, 0, 100);
    state.edit_log = create_ctrl(
        hwnd,
        "EDIT",
        "",
        WS_CHILD
            | WS_BORDER
            | WS_VSCROLL
            | ES_MULTILINE as u32
            | ES_READONLY as u32
            | ES_AUTOVSCROLL as u32,
        0,
        24,
        152,
        556,
        200,
        0,
        state.font_mono,
    );
    state.page_controls[3] = vec![state.lbl_status, state.progress_bar, state.edit_log];

    // Page 4: Finish
    let p4_h1 = create_ctrl(
        hwnd,
        "STATIC",
        &format!("{} 安装完成！", app_name),
        WS_CHILD,
        0,
        28,
        96,
        540,
        28,
        0,
        state.font_title,
    );
    let p4_desc = create_ctrl(
        hwnd,
        "STATIC",
        "安装程序已在您的计算机上完成部署，并自动生成了原生卸载程序 unins000.exe。",
        WS_CHILD,
        0,
        28,
        136,
        540,
        40,
        0,
        state.font_normal,
    );
    state.chk_desktop = create_ctrl(
        hwnd,
        "BUTTON",
        "创建桌面快捷方式",
        WS_CHILD | WS_TABSTOP | BS_AUTOCHECKBOX as u32,
        0,
        28,
        192,
        300,
        24,
        ID_CHK_DESKTOP,
        state.font_normal,
    );
    SendMessageW(state.chk_desktop, BM_SETCHECK, BST_CHECKED, 0);
    state.chk_launch = create_ctrl(
        hwnd,
        "BUTTON",
        &format!("完成后启动 {}", app_name),
        WS_CHILD | WS_TABSTOP | BS_AUTOCHECKBOX as u32,
        0,
        28,
        226,
        300,
        24,
        ID_CHK_LAUNCH,
        state.font_normal,
    );
    SendMessageW(state.chk_launch, BM_SETCHECK, BST_UNCHECKED, 0);
    state.page_controls[4] = vec![p4_h1, p4_desc, state.chk_desktop, state.chk_launch];
}

unsafe fn show_page(hwnd: HWND, state: &mut WizardState, page: usize) {
    state.current_page = page;
    for (idx, ctrls) in state.page_controls.iter().enumerate() {
        let cmd = if idx == page { SW_SHOW } else { SW_HIDE };
        for &c in ctrls {
            ShowWindow(c, cmd);
        }
    }

    match page {
        0 => {
            EnableWindow(state.btn_back, 0);
            EnableWindow(state.btn_next, 1);
            EnableWindow(state.btn_cancel, 1);
            set_window_text(state.btn_next, "下一步 >");
        }
        1 => {
            EnableWindow(state.btn_back, 1);
            let accepted = SendMessageW(state.radio_accept, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
            EnableWindow(state.btn_next, if accepted { 1 } else { 0 });
            EnableWindow(state.btn_cancel, 1);
            set_window_text(state.btn_next, "下一步 >");
        }
        2 => {
            EnableWindow(state.btn_back, 1);
            EnableWindow(state.btn_next, 1);
            EnableWindow(state.btn_cancel, 1);
            set_window_text(state.btn_next, "安装 >");
            refresh_disk_space(state);
        }
        3 => {
            EnableWindow(state.btn_back, 0);
            EnableWindow(state.btn_next, 0);
            EnableWindow(state.btn_cancel, 0);
            set_window_text(state.btn_next, "安装中...");
        }
        _ => {
            EnableWindow(state.btn_back, 0);
            EnableWindow(state.btn_next, 1);
            EnableWindow(state.btn_cancel, 0);
            set_window_text(state.btn_next, "完成");
        }
    }

    windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, null(), 1);
}

unsafe fn refresh_disk_space(state: &WizardState) {
    let dir_txt = get_window_text(state.edit_dir);
    let required_mb = (state.config.total_uncompressed_size as f64) / (1024.0 * 1024.0);
    let text = match query_free_disk_mb(&dir_txt) {
        Some(free_mb) => format!(
            "所需磁盘空间：{:.2} MB    所选驱动器可用空间：{} MB",
            required_mb, free_mb
        ),
        None => format!("所需磁盘空间：{:.2} MB", required_mb),
    };
    set_window_text(state.lbl_space, &text);
}

unsafe fn handle_command(hwnd: HWND, state: &mut WizardState, id: usize, code: u32) {
    match id {
        ID_BTN_BACK => {
            if state.current_page > 0 && state.current_page < 3 {
                show_page(hwnd, state, state.current_page - 1);
            }
        }
        ID_BTN_NEXT => match state.current_page {
            0 | 1 => show_page(hwnd, state, state.current_page + 1),
            2 => {
                let target_dir = get_window_text(state.edit_dir).trim().to_string();
                if target_dir.is_empty() {
                    msg_box(hwnd, "提示", "请输入或选择有效的安装目录！", MB_OK | MB_ICONERROR);
                    return;
                }
                show_page(hwnd, state, 3);
                start_install_worker(hwnd, state, PathBuf::from(target_dir));
            }
            4 => {
                finish_wizard(hwnd, state);
            }
            _ => {}
        },
        ID_BTN_CANCEL => {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
        ID_RADIO_ACCEPT | ID_RADIO_DECLINE => {
            if state.current_page == 1 {
                let accepted =
                    SendMessageW(state.radio_accept, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                EnableWindow(state.btn_next, if accepted { 1 } else { 0 });
            }
        }
        ID_BTN_BROWSE => {
            if let Some(chosen) = pick_folder(hwnd, "选择安装目录") {
                set_window_text(state.edit_dir, &chosen);
                refresh_disk_space(state);
            }
        }
        ID_EDIT_DIR if code == EN_CHANGE => {
            refresh_disk_space(state);
        }
        _ => {}
    }
}

unsafe fn start_install_worker(hwnd: HWND, state: &WizardState, install_dir: PathBuf) {
    let hwnd_val = hwnd as isize;
    let self_exe = state.self_exe.clone();
    let footer = state.footer;
    let config = state.config.clone();

    thread::spawn(move || {
        let target_hwnd = hwnd_val as HWND;
        let result = perform_installation(
            &self_exe,
            &footer,
            &config,
            &install_dir,
            false,
            |curr, total, rel_path| {
                let payload = Box::new(ProgressMsg {
                    curr,
                    total,
                    rel_path: rel_path.to_string(),
                });
                PostMessageW(target_hwnd, WM_APP_PROGRESS, 0, Box::into_raw(payload) as isize);
            },
        );

        match result {
            Ok(outcome) => {
                let boxed = Box::new(outcome);
                PostMessageW(target_hwnd, WM_APP_DONE, 0, Box::into_raw(boxed) as isize);
            }
            Err(err) => {
                let boxed = Box::new(format!("{:#}", err));
                PostMessageW(target_hwnd, WM_APP_ERROR, 0, Box::into_raw(boxed) as isize);
            }
        }
    });
}

unsafe fn finish_wizard(hwnd: HWND, state: &mut WizardState) {
    let want_desktop = SendMessageW(state.chk_desktop, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
    let want_launch = SendMessageW(state.chk_launch, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;

    if let Some(ref mut outcome) = state.outcome {
        if want_desktop {
            let _ = update_desktop_shortcuts_and_manifest(
                &state.self_exe,
                &state.config,
                outcome,
            );
        }
        if want_launch {
            if let Some(ref exe) = outcome.primary_exe {
                shell_execute_open(exe, &outcome.install_dir);
            }
        }
    }
    DestroyWindow(hwnd);
}

unsafe fn append_log(edit_hwnd: HWND, line: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
    let len = GetWindowTextLengthW(edit_hwnd) as usize;
    const EM_SETSEL: u32 = 0x00B1;
    const EM_REPLACESEL: u32 = 0x00C2;
    SendMessageW(edit_hwnd, EM_SETSEL, len, len as isize);
    let w_line = to_wide(line);
    SendMessageW(edit_hwnd, EM_REPLACESEL, 0, w_line.as_ptr() as isize);
}
