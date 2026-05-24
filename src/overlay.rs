#[cfg(windows)]
mod win {
    use anyhow::Result;
    use crate::settings::OverlaySettings;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{
        BOOL, COLORREF, ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HANDLE, HWND, LPARAM, LRESULT, POINT,
        RECT, SIZE, WPARAM,
    };
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateCompatibleDC, CreateDIBSection, CreateFontIndirectW, CreateFontW, CreateRoundRectRgn,
        CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect,
        GetStockObject, GetTextExtentPoint32W, LineTo, MoveToEx, RedrawWindow, SelectObject,
        SetBkMode, SetTextColor, SetWindowRgn, AC_SRC_ALPHA,
        AC_SRC_OVER, ANTIALIASED_QUALITY, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
        CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DT_LEFT, DT_NOCLIP,
        DT_RIGHT, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, HBRUSH, OUT_DEFAULT_PRECIS,
        PAINTSTRUCT, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW, TRANSPARENT,
        WHITE_BRUSH,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
        RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE,
        REG_OPTION_NON_VOLATILE, REG_SZ,
    };
    use windows::Win32::UI::Controls::Dialogs::{
        ChooseColorW, CHOOSECOLORW, CC_FULLOPEN, CC_RGBINIT,
    };
    use windows::Win32::UI::Controls::{
        CreateUpDownControl, UDS_ALIGNRIGHT, UDS_ARROWKEYS, UDS_SETBUDDYINT,
    };
    use windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
    use windows::Win32::UI::Shell::{
        SHAppBarMessage, APPBARDATA, ABM_GETSTATE, ABM_GETTASKBARPOS, ABS_AUTOHIDE,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowExW,
        FindWindowW, GetCursorPos, GetMessageW,
        GetWindowLongPtrW, GetWindowRect, GetWindowTextW, LoadCursorW, MessageBoxW,
        PostQuitMessage, RegisterClassW, SendMessageW, SetForegroundWindow, SetTimer,
        SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow, TranslateMessage,
        UpdateLayeredWindow,
        BM_GETCHECK, BM_SETCHECK, BN_CLICKED, BS_AUTOCHECKBOX, CS_HREDRAW,
        CS_VREDRAW, CW_USEDEFAULT, EN_CHANGE, ES_AUTOHSCROLL, ES_NUMBER, GWLP_USERDATA, HMENU,
        HTCLIENT, IDC_ARROW, MSG, SW_SHOW, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
        ULW_ALPHA, WINDOW_EX_STYLE,
        WINDOW_STYLE, WM_ACTIVATE, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_CREATE,
        WM_CTLCOLORSTATIC, WM_DESTROY, WM_DISPLAYCHANGE, WM_ERASEBKGND, WM_MOUSEMOVE, WM_NCHITTEST, WM_PAINT,
        EnumChildWindows, WM_RBUTTONUP, WM_KEYDOWN, WM_LBUTTONUP, WM_SETFONT, WM_SETTINGCHANGE, WM_TIMER, WNDCLASSW, WA_INACTIVE, WS_BORDER, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
        NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
        SystemParametersInfoW,
    };

    const WINDOW_WIDTH: i32 = 92;
    const WINDOW_HEIGHT: i32 = 42;
    const TIMER_REPOSITION: usize = 2;
    const LINE_HEIGHT: i32 = 18;
    const FIRST_LINE_TOP: i32 = 4;
    const LABEL_LEFT: i32 = 1;
    const PREFIX_VALUE_GAP: i32 = 4;
    const VALUE_RIGHT: i32 = WINDOW_WIDTH - 2;
    const MENU_SETTINGS: usize = 101;
    const TIMER_TOOLTIP: usize = 3;
    const TOOLTIP_WIDTH: i32 = 220;
    const TOOLTIP_HEIGHT: i32 = 54;
    const SETTINGS_WIDTH: i32 = 216;
    const SETTINGS_HEIGHT: i32 = 214;
    const IDC_FONT_SIZE: i32 = 1001;
    const IDC_COLOR_BUTTON: i32 = 1002;
    const IDC_CLAUDE_PREFIX: i32 = 1003;
    const IDC_CODEX_PREFIX: i32 = 1004;
    const IDC_EXIT: i32 = 1006;
    const IDC_STARTUP: i32 = 1007;
    const FONT_MIN: i32 = 8;
    const FONT_MAX: i32 = 32;
    const STARTUP_VALUE_NAME: &str = "CcBalanceOverlay";
    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const COLOR_SWATCH_RECT: RECT = RECT {
        left: 124,
        top: 55,
        right: 158,
        bottom: 73,
    };
    const COLOR_ROW_RECT: RECT = RECT {
        left: 0,
        top: 46,
        right: SETTINGS_WIDTH,
        bottom: 82,
    };
    const EXIT_ROW_RECT: RECT = RECT {
        left: 0,
        top: 170,
        right: SETTINGS_WIDTH,
        bottom: 206,
    };

    pub struct OverlayWindow {
        hwnd: HWND,
        tooltip_hwnd: HWND,
        settings_hwnd: HWND,
        text: String,
        tooltip: String,
        settings: OverlaySettings,
        pending_command: Option<OverlayCommand>,
        suppress_settings_events: bool,
        color_dialog_open: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OverlayCommand {
        Quit,
        RefreshSettings,
    }

    impl OverlayWindow {
        pub fn new(settings: OverlaySettings) -> Result<Box<Self>> {
            unsafe {
                let _ = SetProcessDpiAwarenessContext(
                    windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
                );
            }

            let instance = unsafe { GetModuleHandleW(None)? };
            let class_name = wide("CcBalanceOverlayWindow");
            let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
            let wc = WNDCLASSW {
                hCursor: cursor,
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                ..Default::default()
            };
            unsafe { RegisterClassW(&wc) };

            let mut window = Box::new(Self {
                hwnd: HWND(null_mut()),
                tooltip_hwnd: HWND(null_mut()),
                settings_hwnd: HWND(null_mut()),
                text: "C --\nX --".to_string(),
                tooltip: "Claude: --\nCodex: --".to_string(),
                settings,
                pending_command: None,
                suppress_settings_events: false,
                color_dialog_open: false,
            });
            let raw = window.as_mut() as *mut Self as *mut c_void;
            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(
                        WS_EX_TOPMOST.0
                            | WS_EX_TOOLWINDOW.0
                            | WS_EX_NOACTIVATE.0
                            | WS_EX_LAYERED.0,
                    ),
                    PCWSTR(class_name.as_ptr()),
                    PCWSTR(wide("CC Balance").as_ptr()),
                    WS_POPUP,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                    None,
                    HMENU(null_mut()),
                    instance,
                    Some(raw),
                )
            };
            let hwnd = hwnd?;
            window.hwnd = hwnd;
            let tooltip_hwnd = unsafe { create_tooltip_window(instance.into(), hwnd) }?;
            window.tooltip_hwnd = tooltip_hwnd;
            unsafe {
                SetTimer(hwnd, TIMER_REPOSITION, 3000, None);
                SetTimer(hwnd, TIMER_TOOLTIP, 500, None);
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
            window.reposition();
            window.render();
            Ok(window)
        }

        pub fn set_text(&mut self, text: impl Into<String>) {
            self.text = text.into();
            self.render();
        }

        pub fn set_tooltip(&mut self, tooltip: impl Into<String>) {
            self.tooltip = tooltip.into();
            unsafe {
                let title = wide(&self.tooltip);
                let _ = SetWindowTextW(self.hwnd, PCWSTR(title.as_ptr()));
                if !self.tooltip_hwnd.is_invalid() {
                    let _ = SetWindowTextW(self.tooltip_hwnd, PCWSTR(title.as_ptr()));
                }
            }
        }

        pub fn apply_settings(&mut self, settings: OverlaySettings) {
            self.settings = settings;
            self.render();
        }

        pub fn run_message_loop<F>(&mut self, mut on_tick: F) -> Result<()>
        where
            F: FnMut(&mut Self) + 'static,
        {
            let mut last_tick = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(60))
                .unwrap_or_else(std::time::Instant::now);
            loop {
                let mut msg = MSG::default();
                let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if result.0 == -1 {
                    return Err(anyhow::anyhow!("GetMessageW failed"));
                }
                if result.0 == 0 {
                    break;
                }
                if msg.message == WM_TIMER && msg.wParam.0 == TIMER_REPOSITION {
                    self.reposition();
                    if let Some(command) = self.pending_command.take() {
                        if matches!(command, OverlayCommand::RefreshSettings) {
                            on_tick(self);
                        }
                    } else if last_tick.elapsed() >= std::time::Duration::from_secs(30) {
                        on_tick(self);
                        last_tick = std::time::Instant::now();
                    }
                } else if msg.message == WM_TIMER && msg.wParam.0 == TIMER_TOOLTIP {
                    unsafe {
                        self.update_tooltip_visibility();
                    }
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            Ok(())
        }

        fn reposition(&self) {
            if let Some(rect) = taskbar_rect() {
                let auto_hide = taskbar_auto_hide();
                let padding = if auto_hide { 4 } else { 10 };
                let anchor_left = tray_notify_rect()
                    .map(|tray| tray.left)
                    .unwrap_or(rect.right - 590);
                let x = anchor_left - WINDOW_WIDTH - 2;
                let y = rect.bottom - WINDOW_HEIGHT - padding;
                unsafe {
                    let _ = SetWindowPos(
                        self.hwnd,
                        HWND(-1isize as *mut c_void),
                        x,
                        y,
                        WINDOW_WIDTH,
                        WINDOW_HEIGHT,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                }
                self.render();
            }
        }

        fn render(&self) {
            unsafe {
                render_layered_text(self.hwnd, &self.text, &self.settings);
            }
        }
    }

    extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            if msg == WM_CREATE {
                let createstruct = lparam.0
                    as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
                if !createstruct.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*createstruct).lpCreateParams as isize);
                }
            }
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayWindow;
            match msg {
                WM_DISPLAYCHANGE => {
                    if !ptr.is_null() {
                        (*ptr).reposition();
                    }
                    LRESULT(0)
                }
                WM_NCHITTEST => LRESULT(HTCLIENT as isize),
                WM_RBUTTONUP => {
                    if !ptr.is_null() {
                        (*ptr).show_settings_window();
                    }
                    LRESULT(0)
                }
                WM_MOUSEMOVE => {
                    if !ptr.is_null() {
                        (*ptr).show_tooltip();
                    }
                    LRESULT(0)
                }
                WM_CONTEXTMENU => {
                    if !ptr.is_null() {
                        (*ptr).show_settings_window();
                    }
                    LRESULT(0)
                }
                WM_COMMAND => {
                    if !ptr.is_null() {
                        (*ptr).handle_menu_command(loword(wparam.0 as usize) as usize);
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    if !ptr.is_null() && !(*ptr).tooltip_hwnd.is_invalid() {
                        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(
                            (*ptr).tooltip_hwnd,
                        );
                    }
                    if !ptr.is_null() && !(*ptr).settings_hwnd.is_invalid() {
                        let _ = DestroyWindow((*ptr).settings_hwnd);
                    }
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    impl OverlayWindow {
        unsafe fn show_tooltip(&self) {
            if self.tooltip_hwnd.is_invalid() {
                return;
            }
            let mut rect = RECT::default();
            if GetWindowRect(self.hwnd, &mut rect).is_err() {
                return;
            }
            let x = rect.right - TOOLTIP_WIDTH;
            let y = rect.top - TOOLTIP_HEIGHT - 6;
            let _ = SetWindowPos(
                self.tooltip_hwnd,
                HWND(-1isize as *mut c_void),
                x,
                y,
                TOOLTIP_WIDTH,
                TOOLTIP_HEIGHT,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }

        unsafe fn hide_tooltip(&self) {
            if !self.tooltip_hwnd.is_invalid() {
                let _ = ShowWindow(self.tooltip_hwnd, windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
            }
        }

        unsafe fn update_tooltip_visibility(&self) {
            let mut cursor = POINT::default();
            let mut rect = RECT::default();
            if GetCursorPos(&mut cursor).is_err() || GetWindowRect(self.hwnd, &mut rect).is_err() {
                return;
            }
            let inside = cursor.x >= rect.left
                && cursor.x <= rect.right
                && cursor.y >= rect.top
                && cursor.y <= rect.bottom;
            if inside {
                self.show_tooltip();
            } else {
                self.hide_tooltip();
            }
        }

        unsafe fn handle_menu_command(&mut self, command: usize) {
            match command {
                MENU_SETTINGS => {
                    self.show_settings_window();
                }
                _ => {}
            }
        }

        fn save_settings_and_refresh(&mut self) {
            if let Err(err) = self.settings.save() {
                unsafe {
                    let text = wide(&format!("保存设置失败：{err}"));
                    let caption = wide("CC Balance");
                    let _ = MessageBoxW(
                        self.hwnd,
                        PCWSTR(text.as_ptr()),
                        PCWSTR(caption.as_ptr()),
                        Default::default(),
                    );
                }
                return;
            }
            self.render();
            self.pending_command = Some(OverlayCommand::RefreshSettings);
        }

        unsafe fn show_settings_window(&mut self) {
            if !self.settings_hwnd.is_invalid() {
                self.sync_settings_controls();
                let _ = ShowWindow(self.settings_hwnd, SW_SHOW);
                let _ = SetForegroundWindow(self.settings_hwnd);
                return;
            }

            let Ok(instance) = GetModuleHandleW(None) else {
                return;
            };
            let class_name = wide("CcBalanceSettingsWindow");
            let cursor = match LoadCursorW(None, IDC_ARROW) {
                Ok(cursor) => cursor,
                Err(_) => Default::default(),
            };
            let wc = WNDCLASSW {
                hCursor: cursor,
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(settings_proc),
                ..Default::default()
            };
            RegisterClassW(&wc);

            let mut cursor = POINT::default();
            let mut owner_rect = RECT::default();
            let (x, y) = if GetCursorPos(&mut cursor).is_ok() {
                (cursor.x - SETTINGS_WIDTH + 10, (cursor.y - SETTINGS_HEIGHT - 6).max(0))
            } else if GetWindowRect(self.hwnd, &mut owner_rect).is_ok() {
                (owner_rect.right - SETTINGS_WIDTH, owner_rect.top - SETTINGS_HEIGHT - 8)
            } else {
                (CW_USEDEFAULT, CW_USEDEFAULT)
            };

            let raw = self as *mut Self as *mut c_void;
            let hwnd = match CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(wide("CC Balance Settings").as_ptr()),
                WS_POPUP,
                x,
                y,
                SETTINGS_WIDTH,
                SETTINGS_HEIGHT,
                None,
                HMENU(null_mut()),
                instance,
                Some(raw),
            ) {
                Ok(hwnd) => hwnd,
                Err(_) => return,
            };
            self.settings_hwnd = hwnd;
            let rgn = CreateRoundRectRgn(0, 0, SETTINGS_WIDTH + 1, SETTINGS_HEIGHT + 1, 12, 12);
            let _ = SetWindowRgn(hwnd, rgn, true);
            self.sync_settings_controls();
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = SetForegroundWindow(hwnd);
            let _ = RedrawWindow(
                hwnd,
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
        }

        unsafe fn sync_settings_controls(&mut self) {
            if self.settings_hwnd.is_invalid() {
                return;
            }
            self.suppress_settings_events = true;
            set_control_text(self.settings_hwnd, IDC_FONT_SIZE, &self.settings.font_size.to_string());
            set_control_text(self.settings_hwnd, IDC_CLAUDE_PREFIX, &self.settings.claude_prefix);
            set_control_text(self.settings_hwnd, IDC_CODEX_PREFIX, &self.settings.codex_prefix);
            set_checkbox_checked(self.settings_hwnd, IDC_STARTUP, startup_enabled());
            self.suppress_settings_events = false;
        }

        unsafe fn handle_settings_command(&mut self, control_id: i32, notify_code: u32) {
            if self.suppress_settings_events {
                return;
            }
            match control_id {
                IDC_FONT_SIZE if notify_code == EN_CHANGE => {
                    if let Some(value) = get_control_text(self.settings_hwnd, IDC_FONT_SIZE)
                        .and_then(|text| text.trim().parse::<i32>().ok())
                    {
                        let value = value.clamp(FONT_MIN, FONT_MAX);
                        if self.settings.font_size != value {
                            self.settings.font_size = value;
                            self.save_settings_and_refresh();
                        }
                    }
                }
                IDC_CLAUDE_PREFIX if notify_code == EN_CHANGE => {
                    if let Some(value) = get_control_text(self.settings_hwnd, IDC_CLAUDE_PREFIX) {
                        let value = value.trim().to_string();
                        if self.settings.claude_prefix != value {
                            self.settings.claude_prefix = value;
                            self.save_settings_and_refresh();
                        }
                    }
                }
                IDC_CODEX_PREFIX if notify_code == EN_CHANGE => {
                    if let Some(value) = get_control_text(self.settings_hwnd, IDC_CODEX_PREFIX) {
                        let value = value.trim().to_string();
                        if self.settings.codex_prefix != value {
                            self.settings.codex_prefix = value;
                            self.save_settings_and_refresh();
                        }
                    }
                }
                IDC_COLOR_BUTTON if notify_code == BN_CLICKED => {
                    self.open_color_dialog();
                }
                IDC_STARTUP if notify_code == BN_CLICKED => {
                    let enabled = is_checkbox_checked(self.settings_hwnd, IDC_STARTUP);
                    if let Err(err) = set_startup_enabled(enabled) {
                        set_checkbox_checked(self.settings_hwnd, IDC_STARTUP, !enabled);
                        let text = wide(&format!("设置开机启动失败：{err}"));
                        let caption = wide("CC Balance");
                        let _ = MessageBoxW(
                            self.settings_hwnd,
                            PCWSTR(text.as_ptr()),
                            PCWSTR(caption.as_ptr()),
                            Default::default(),
                        );
                    }
                }
                IDC_EXIT if notify_code == BN_CLICKED => {
                    PostQuitMessage(0);
                }
                _ => {}
            }
        }

        unsafe fn open_color_dialog(&mut self) {
            self.color_dialog_open = true;
            if let Some(color) = choose_text_color(self.settings_hwnd, &self.settings.color) {
                if self.settings.color != color {
                    self.settings.color = color;
                    self.save_settings_and_refresh();
                    let _ = RedrawWindow(
                        self.settings_hwnd,
                        None,
                        None,
                        RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN,
                    );
                }
            }
            self.color_dialog_open = false;
        }
    }

    extern "system" fn settings_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if msg == WM_CREATE {
                let createstruct = lparam.0
                    as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
                if !createstruct.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*createstruct).lpCreateParams as isize);
                    create_settings_controls(hwnd);
                }
                return LRESULT(0);
            }

            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayWindow;
            match msg {
                WM_COMMAND => {
                    if !ptr.is_null() {
                        let control_id = loword(wparam.0 as usize) as i32;
                        let notify_code = hiword(wparam.0 as usize) as u32;
                        (*ptr).handle_settings_command(control_id, notify_code);
                    }
                    LRESULT(0)
                }
                WM_CLOSE => {
                    if !ptr.is_null() {
                        (*ptr).settings_hwnd = HWND(null_mut());
                    }
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                }
                WM_ACTIVATE => {
                    if loword(wparam.0 as usize) as u32 == WA_INACTIVE {
                        if !ptr.is_null() && (*ptr).color_dialog_open {
                            return LRESULT(0);
                        }
                        if !ptr.is_null() {
                            (*ptr).settings_hwnd = HWND(null_mut());
                        }
                        let _ = DestroyWindow(hwnd);
                        return LRESULT(0);
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_SETTINGCHANGE => {
                    refresh_settings_fonts(hwnd);
                    let _ = RedrawWindow(
                        hwnd,
                        None,
                        None,
                        RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN,
                    );
                    LRESULT(0)
                }
                WM_KEYDOWN => {
                    if wparam.0 as u16 == VK_ESCAPE.0 {
                        if !ptr.is_null() {
                            (*ptr).settings_hwnd = HWND(null_mut());
                        }
                        let _ = DestroyWindow(hwnd);
                    }
                    LRESULT(0)
                }
                WM_LBUTTONUP => {
                    let x = loword(lparam.0 as usize) as i32;
                    let y = hiword(lparam.0 as usize) as i32;
                    if !ptr.is_null() && point_in_rect(x, y, &COLOR_ROW_RECT) {
                        (*ptr).open_color_dialog();
                        return LRESULT(0);
                    }
                    if point_in_rect(x, y, &EXIT_ROW_RECT) {
                        PostQuitMessage(0);
                        return LRESULT(0);
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_CTLCOLORSTATIC => {
                    let hdc = windows::Win32::Graphics::Gdi::HDC(wparam.0 as *mut c_void);
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, COLORREF(0x00000000));
                    LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
                }
                WM_ERASEBKGND => LRESULT(1),
                WM_PAINT => {
                    paint_settings_panel(hwnd);
                    LRESULT(0)
                }
                WM_DESTROY => {
                    if !ptr.is_null() && (*ptr).settings_hwnd == hwnd {
                        (*ptr).settings_hwnd = HWND(null_mut());
                    }
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    unsafe fn create_tooltip_window(
        instance: windows::Win32::Foundation::HINSTANCE,
        owner: HWND,
    ) -> windows::core::Result<HWND> {
        let class_name = wide("CcBalanceOverlayTooltip");
        let wc = WNDCLASSW {
            hInstance: instance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(tooltip_proc),
            ..Default::default()
        };
        RegisterClassW(&wc);
        CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WS_POPUP,
            0,
            0,
            TOOLTIP_WIDTH,
            TOOLTIP_HEIGHT,
            owner,
            HMENU(null_mut()),
            instance,
            None,
        )
    }

    unsafe fn create_settings_controls(hwnd: HWND) {
        let font = system_ui_font();
        create_label(hwnd, "字号", 24, 17, 70, 22, font);
        let larger_font = system_ui_font_delta(-2);
        let font_edit = create_edit(
            hwnd,
            IDC_FONT_SIZE,
            120,
            15,
            56,
            24,
            ES_NUMBER | ES_AUTOHSCROLL,
            larger_font,
        );
        let _spin = CreateUpDownControl(
            (WS_CHILD.0 | WS_VISIBLE.0) | UDS_ALIGNRIGHT | UDS_SETBUDDYINT | UDS_ARROWKEYS,
            0,
            0,
            0,
            0,
            hwnd,
            0,
            GetModuleHandleW(None).unwrap_or_default(),
            font_edit,
            FONT_MAX,
            FONT_MIN,
            FONT_MIN,
        );

        create_label(hwnd, "颜色", 24, 53, 70, 22, font);

        create_label(hwnd, "Claude 前缀", 24, 89, 86, 22, font);
        create_edit(
            hwnd,
            IDC_CLAUDE_PREFIX,
            120,
            87,
            66,
            24,
            ES_AUTOHSCROLL,
            font,
        );

        create_label(hwnd, "Codex 前缀", 24, 125, 86, 22, font);
        create_edit(
            hwnd,
            IDC_CODEX_PREFIX,
            120,
            123,
            66,
            24,
            ES_AUTOHSCROLL,
            font,
        );

        create_checkbox(hwnd, IDC_STARTUP, "开机启动", 24, 153, 100, 24, font);
    }

    unsafe fn create_label(
        hwnd: HWND,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        font: windows::Win32::Graphics::Gdi::HFONT,
    ) -> HWND {
        let control = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            PCWSTR(wide(text).as_ptr()),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            width,
            height,
            hwnd,
            HMENU(null_mut()),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(control, font);
        control
    }

    unsafe fn create_edit(
        hwnd: HWND,
        id: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        extra_style: i32,
        font: windows::Win32::Graphics::Gdi::HFONT,
    ) -> HWND {
        let control = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | WINDOW_STYLE(extra_style as u32),
            x,
            y,
            width,
            height,
            hwnd,
            HMENU(id as *mut c_void),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(control, font);
        control
    }

    unsafe fn create_checkbox(
        hwnd: HWND,
        id: i32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        font: windows::Win32::Graphics::Gdi::HFONT,
    ) -> HWND {
        let control = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            PCWSTR(wide(text).as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            x,
            y,
            width,
            height,
            hwnd,
            HMENU(id as *mut c_void),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(control, font);
        control
    }

    unsafe fn system_ui_font() -> windows::Win32::Graphics::Gdi::HFONT {
        system_ui_font_delta(0)
    }

    unsafe fn system_ui_font_delta(height_delta: i32) -> windows::Win32::Graphics::Gdi::HFONT {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        if SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some(&mut metrics as *mut _ as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
        {
            let mut font = metrics.lfMenuFont;
            font.lfHeight += height_delta;
            return CreateFontIndirectW(&font);
        }
        CreateFontW(
            -15,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            ANTIALIASED_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR::null(),
        )
    }

    unsafe fn set_control_font(hwnd: HWND, font: windows::Win32::Graphics::Gdi::HFONT) {
        if !hwnd.is_invalid() && !font.is_invalid() {
            SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
        }
    }

    unsafe fn overlay_text_font(font_size: i32) -> windows::Win32::Graphics::Gdi::HFONT {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        if SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some(&mut metrics as *mut _ as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
        {
            let mut font = metrics.lfMessageFont;
            font.lfHeight = -font_size;
            font.lfWeight = FW_NORMAL.0 as i32;
            return CreateFontIndirectW(&font);
        }
        CreateFontW(
            -font_size,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            ANTIALIASED_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR::null(),
        )
    }

    unsafe fn refresh_settings_fonts(hwnd: HWND) {
        let font = system_ui_font();
        let _ = EnumChildWindows(
            hwnd,
            Some(refresh_settings_child_font),
            LPARAM(font.0 as isize),
        );
        let larger_font = system_ui_font_delta(-2);
        if let Ok(control) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, IDC_FONT_SIZE)
        {
            set_control_font(control, larger_font);
        }
    }

    extern "system" fn refresh_settings_child_font(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            set_control_font(
                hwnd,
                windows::Win32::Graphics::Gdi::HFONT(lparam.0 as *mut c_void),
            );
        }
        BOOL(1)
    }

    unsafe fn paint_settings_panel(hwnd: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let rect = RECT {
            left: 0,
            top: 0,
            right: SETTINGS_WIDTH,
            bottom: SETTINGS_HEIGHT,
        };
        let white = CreateSolidBrush(COLORREF(0x00ffffff));
        FillRect(hdc, &rect, HBRUSH(white.0));
        let border = CreateSolidBrush(COLORREF(0x00d8d8d8));
        FrameRect(hdc, &rect, HBRUSH(border.0));

        let old_pen = SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::CreatePen(
                windows::Win32::Graphics::Gdi::PS_SOLID,
                1,
                COLORREF(0x00e6e6e6),
            ),
        );
        for y in [46, 82, 150, 170] {
            let _ = MoveToEx(hdc, 0, y, None);
            let _ = LineTo(hdc, SETTINGS_WIDTH, y);
        }
        paint_palette_swatch(hdc, HBRUSH(border.0));
        paint_exit_row(hdc, HBRUSH(border.0));
        let pen = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
        let _ = DeleteObject(white);
        let _ = DeleteObject(border);
        let _ = EndPaint(hwnd, &ps);
    }

    unsafe fn paint_palette_swatch(hdc: windows::Win32::Graphics::Gdi::HDC, border: HBRUSH) {
        let colors = [
            COLORREF(0x000052ff),
            COLORREF(0x0000b7ff),
            COLORREF(0x0000d26a),
            COLORREF(0x00d0c000),
            COLORREF(0x00ff6a00),
            COLORREF(0x00b050ff),
        ];
        let left = COLOR_SWATCH_RECT.left;
        let top = COLOR_SWATCH_RECT.top;
        let width = COLOR_SWATCH_RECT.right - COLOR_SWATCH_RECT.left;
        let height = COLOR_SWATCH_RECT.bottom - COLOR_SWATCH_RECT.top;
        let stripe_width = width / colors.len() as i32;
        for (index, color) in colors.iter().enumerate() {
            let stripe_left = left + index as i32 * stripe_width;
            let stripe_right = if index == colors.len() - 1 {
                left + width
            } else {
                stripe_left + stripe_width
            };
            let brush = CreateSolidBrush(*color);
            let rect = RECT {
                left: stripe_left,
                top,
                right: stripe_right,
                bottom: top + height,
            };
            FillRect(hdc, &rect, HBRUSH(brush.0));
            let _ = DeleteObject(brush);
        }
        let swatch_rect = RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        };
        FrameRect(hdc, &swatch_rect, border);
    }

    unsafe fn paint_exit_row(hdc: windows::Win32::Graphics::Gdi::HDC, _border: HBRUSH) {
        let bg = CreateSolidBrush(COLORREF(0x00ffffff));
        FillRect(hdc, &EXIT_ROW_RECT, HBRUSH(bg.0));
        let _ = DeleteObject(bg);

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00000000));
        let font = system_ui_font();
        let old = SelectObject(hdc, font);
        let mut text_rect = RECT {
            left: 24,
            top: EXIT_ROW_RECT.top,
            right: EXIT_ROW_RECT.right - 24,
            bottom: EXIT_ROW_RECT.bottom,
        };
        let mut text = wide("退出");
        DrawTextW(
            hdc,
            &mut text,
            &mut text_rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(hdc, old);
    }

    extern "system" fn tooltip_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match msg {
                WM_PAINT => {
                    paint_tooltip(hwnd);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    unsafe fn paint_tooltip(hwnd: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let rect = RECT {
            left: 0,
            top: 0,
            right: TOOLTIP_WIDTH,
            bottom: TOOLTIP_HEIGHT,
        };
        let brush = CreateSolidBrush(COLORREF(0x00282828));
        FillRect(hdc, &rect, HBRUSH(brush.0));
        let _ = DeleteObject(brush);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00ffffff));
        let font_name = wide("Segoe UI");
        let font = CreateFontW(
            -13,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            ANTIALIASED_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR(font_name.as_ptr()),
        );
        let old = SelectObject(hdc, font);
        let mut text_rect = RECT {
            left: 10,
            top: 8,
            right: TOOLTIP_WIDTH - 10,
            bottom: TOOLTIP_HEIGHT - 8,
        };
        let mut title = [0u16; 256];
        let len = windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut title);
        let mut text = title[..len as usize]
            .iter()
            .copied()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        DrawTextW(hdc, &mut text, &mut text_rect, DT_LEFT | DT_NOCLIP);
        SelectObject(hdc, old);
        let _ = DeleteObject(font);
        let _ = EndPaint(hwnd, &ps);
    }

    unsafe fn render_layered_text(hwnd: HWND, text: &str, settings: &OverlaySettings) {
        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            return;
        }
        let mut bits: *mut c_void = null_mut();
        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: WINDOW_WIDTH,
                biHeight: -WINDOW_HEIGHT,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let bitmap = match CreateDIBSection(
            hdc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            HANDLE(null_mut()),
            0,
        ) {
            Ok(bitmap) => bitmap,
            Err(_) => {
                let _ = DeleteDC(hdc);
                return;
            }
        };
        if bitmap.is_invalid() || bits.is_null() {
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(hdc);
            return;
        }

        let old_bitmap = SelectObject(hdc, bitmap);
        let pixels =
            std::slice::from_raw_parts_mut(bits as *mut u8, (WINDOW_WIDTH * WINDOW_HEIGHT * 4) as usize);
        pixels.fill(0);
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = 1;
        }

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00ffffff));
        let font = overlay_text_font(settings.font_size);
        let old_font = SelectObject(hdc, font);
        for (index, line) in text.lines().take(2).enumerate() {
            draw_overlay_line(hdc, line, FIRST_LINE_TOP + index as i32 * LINE_HEIGHT);
        }

        let text_color = settings.text_color_bgr();
        let text_red = (text_color & 0xff) as u8;
        let text_green = ((text_color >> 8) & 0xff) as u8;
        let text_blue = ((text_color >> 16) & 0xff) as u8;
        for pixel in pixels.chunks_exact_mut(4) {
            let coverage = pixel[0].max(pixel[1]).max(pixel[2]);
            if coverage <= 1 {
                continue;
            }
            let alpha = text_coverage_alpha(coverage);
            pixel[0] = premultiply_channel(text_blue, alpha);
            pixel[1] = premultiply_channel(text_green, alpha);
            pixel[2] = premultiply_channel(text_red, alpha);
            pixel[3] = alpha;
        }

        let mut hwnd_rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut hwnd_rect);
        let dst_point = POINT {
            x: hwnd_rect.left,
            y: hwnd_rect.top,
        };
        let size = SIZE {
            cx: WINDOW_WIDTH,
            cy: WINDOW_HEIGHT,
        };
        let src_point = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let _ = UpdateLayeredWindow(
            hwnd,
            None,
            Some(&dst_point),
            Some(&size),
            hdc,
            Some(&src_point),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
        let mut rect_after = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect_after);

        SelectObject(hdc, old_font);
        let _ = DeleteObject(font);
        SelectObject(hdc, old_bitmap);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(hdc);
    }

    fn taskbar_rect() -> Option<RECT> {
        let mut data = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            ..Default::default()
        };
        let result = unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut data) };
        (result != 0).then_some(data.rc)
    }

    fn tray_notify_rect() -> Option<RECT> {
        unsafe {
            let tray = FindWindowW(PCWSTR(wide("Shell_TrayWnd").as_ptr()), PCWSTR::null()).ok()?;
            if tray.0.is_null() {
                return None;
            }
            let notify = FindWindowExW(
                tray,
                HWND(null_mut()),
                PCWSTR(wide("TrayNotifyWnd").as_ptr()),
                PCWSTR::null(),
            )
            .ok()?;
            if notify.0.is_null() {
                return None;
            }
            let mut rect = RECT::default();
            GetWindowRect(notify, &mut rect).ok()?;
            Some(rect)
        }
    }

    fn taskbar_auto_hide() -> bool {
        let mut data = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            ..Default::default()
        };
        let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) };
        state & ABS_AUTOHIDE as usize != 0
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn loword(value: usize) -> u16 {
        (value & 0xffff) as u16
    }

    fn hiword(value: usize) -> u16 {
        ((value >> 16) & 0xffff) as u16
    }

    fn point_in_rect(x: i32, y: i32, rect: &RECT) -> bool {
        x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
    }

    unsafe fn set_control_text(parent: HWND, id: i32, value: &str) {
        if let Ok(control) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(parent, id) {
            let text = wide(value);
            let _ = SetWindowTextW(control, PCWSTR(text.as_ptr()));
        }
    }

    unsafe fn get_control_text(parent: HWND, id: i32) -> Option<String> {
        let control = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(parent, id).ok()?;
        let mut buffer = [0u16; 128];
        let len = GetWindowTextW(control, &mut buffer);
        Some(String::from_utf16_lossy(&buffer[..len as usize]))
    }

    unsafe fn set_checkbox_checked(parent: HWND, id: i32, checked: bool) {
        if let Ok(control) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(parent, id) {
            let state = if checked { 1 } else { 0 };
            let _ = SendMessageW(control, BM_SETCHECK, WPARAM(state), LPARAM(0));
        }
    }

    unsafe fn is_checkbox_checked(parent: HWND, id: i32) -> bool {
        let Ok(control) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(parent, id) else {
            return false;
        };
        SendMessageW(control, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == 1
    }

    fn startup_enabled() -> bool {
        unsafe { startup_value().is_some() }
    }

    fn set_startup_enabled(enabled: bool) -> Result<()> {
        unsafe {
            if enabled {
                let exe = std::env::current_exe()?;
                let command = format!("\"{}\"", exe.display());
                set_startup_value(&command)?;
            } else {
                delete_startup_value()?;
            }
        }
        Ok(())
    }

    unsafe fn startup_value() -> Option<String> {
        let key = open_run_key(KEY_QUERY_VALUE).ok()?;
        let value_name = wide(STARTUP_VALUE_NAME);
        let mut value_type = REG_SZ;
        let mut len = 0u32;
        let first = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            None,
            Some(&mut len),
        );
        if first != ERROR_SUCCESS || value_type != REG_SZ || len == 0 {
            let _ = RegCloseKey(key);
            return None;
        }
        let mut buffer = vec![0u8; len as usize];
        let second = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some(buffer.as_mut_ptr()),
            Some(&mut len),
        );
        let _ = RegCloseKey(key);
        if second != ERROR_SUCCESS || value_type != REG_SZ {
            return None;
        }
        let words = std::slice::from_raw_parts(buffer.as_ptr() as *const u16, buffer.len() / 2);
        let end = words.iter().position(|ch| *ch == 0).unwrap_or(words.len());
        Some(String::from_utf16_lossy(&words[..end]))
    }

    unsafe fn set_startup_value(command: &str) -> Result<()> {
        let key = create_run_key()?;
        let value_name = wide(STARTUP_VALUE_NAME);
        let value = wide(command);
        let data = std::slice::from_raw_parts(
            value.as_ptr() as *const u8,
            value.len() * std::mem::size_of::<u16>(),
        );
        let result = RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(data));
        let _ = RegCloseKey(key);
        if result != ERROR_SUCCESS {
            anyhow::bail!("RegSetValueExW failed: {}", result.0);
        }
        Ok(())
    }

    unsafe fn delete_startup_value() -> Result<()> {
        let key = open_run_key(KEY_SET_VALUE)?;
        let value_name = wide(STARTUP_VALUE_NAME);
        let result = RegDeleteValueW(key, PCWSTR(value_name.as_ptr()));
        let _ = RegCloseKey(key);
        if result != ERROR_SUCCESS && result != ERROR_FILE_NOT_FOUND {
            anyhow::bail!("RegDeleteValueW failed: {}", result.0);
        }
        Ok(())
    }

    unsafe fn open_run_key(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Result<HKEY> {
        let mut key = HKEY::default();
        let subkey = wide(RUN_KEY);
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            access,
            &mut key,
        );
        if result != ERROR_SUCCESS {
            anyhow::bail!("RegOpenKeyExW failed: {}", result.0);
        }
        Ok(key)
    }

    unsafe fn create_run_key() -> Result<HKEY> {
        let mut key = HKEY::default();
        let subkey = wide(RUN_KEY);
        let result = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        );
        if result != ERROR_SUCCESS {
            anyhow::bail!("RegCreateKeyExW failed: {}", result.0);
        }
        Ok(key)
    }

    fn colorref_to_hex(color: COLORREF) -> String {
        let raw = color.0;
        let r = raw & 0xff;
        let g = (raw >> 8) & 0xff;
        let b = (raw >> 16) & 0xff;
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    fn hex_to_colorref(value: &str) -> COLORREF {
        let raw = value.trim().trim_start_matches('#');
        if raw.len() != 6 {
            return COLORREF(0x00ffffff);
        }
        let Ok(r) = u32::from_str_radix(&raw[0..2], 16) else {
            return COLORREF(0x00ffffff);
        };
        let Ok(g) = u32::from_str_radix(&raw[2..4], 16) else {
            return COLORREF(0x00ffffff);
        };
        let Ok(b) = u32::from_str_radix(&raw[4..6], 16) else {
            return COLORREF(0x00ffffff);
        };
        COLORREF(r | (g << 8) | (b << 16))
    }

    unsafe fn choose_text_color(owner: HWND, current: &str) -> Option<String> {
        let mut custom_colors = [
            COLORREF(0x00ffffff),
            COLORREF(0x00d0f8d7),
            COLORREF(0x00ffe8d7),
            COLORREF(0x00d7e8ff),
            COLORREF(0x00fff2d7),
            COLORREF(0x00f0d7ff),
            COLORREF(0x00e0e0e0),
            COLORREF(0x00b0b0b0),
            COLORREF(0x00808080),
            COLORREF(0x00404040),
            COLORREF(0x00000000),
            COLORREF(0x0000ffff),
            COLORREF(0x0000ff00),
            COLORREF(0x00ff0000),
            COLORREF(0x00ff00ff),
            COLORREF(0x00ffff00),
        ];
        let mut dialog = CHOOSECOLORW {
            lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
            hwndOwner: owner,
            rgbResult: hex_to_colorref(current),
            lpCustColors: custom_colors.as_mut_ptr(),
            Flags: CC_RGBINIT | CC_FULLOPEN,
            ..Default::default()
        };
        if ChooseColorW(&mut dialog).as_bool() {
            Some(colorref_to_hex(dialog.rgbResult))
        } else {
            None
        }
    }

    unsafe fn draw_overlay_line(hdc: windows::Win32::Graphics::Gdi::HDC, line: &str, top: i32) {
        let (label, value) = line.split_once(' ').unwrap_or((line, ""));
        let mut label_size = SIZE::default();
        let label_wide = wide(label);
        let label_text_slice = &label_wide[..label_wide.len().saturating_sub(1)];
        if !label_text_slice.is_empty() {
            let _ = GetTextExtentPoint32W(hdc, label_text_slice, &mut label_size);
        }
        let value_left = (LABEL_LEFT + label_size.cx + PREFIX_VALUE_GAP).min(VALUE_RIGHT - 20);
        let mut label_rect = RECT {
            left: LABEL_LEFT,
            top,
            right: value_left - PREFIX_VALUE_GAP + 1,
            bottom: top + LINE_HEIGHT,
        };
        let mut value_rect = RECT {
            left: value_left,
            top,
            right: VALUE_RIGHT,
            bottom: top + LINE_HEIGHT,
        };
        let mut label_text = label_wide;
        let mut value_text = wide(value);
        DrawTextW(
            hdc,
            &mut label_text,
            &mut label_rect,
            DT_LEFT | DT_SINGLELINE | DT_NOCLIP,
        );
        DrawTextW(
            hdc,
            &mut value_text,
            &mut value_rect,
            DT_RIGHT | DT_SINGLELINE | DT_NOCLIP,
        );
    }

    fn text_coverage_alpha(value: u8) -> u8 {
        if value == 0 {
            return 0;
        }
        value
    }

    fn premultiply_channel(channel: u8, alpha: u8) -> u8 {
        ((channel as u16 * alpha as u16 + 127) / 255) as u8
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn premultiplies_text_color_channels_for_layered_windows() {
            assert_eq!(premultiply_channel(255, 255), 255);
            assert_eq!(premultiply_channel(255, 128), 128);
            assert_eq!(premultiply_channel(64, 128), 32);
            assert_eq!(premultiply_channel(0, 200), 0);
        }
    }

}

#[cfg(windows)]
pub use win::{OverlayCommand, OverlayWindow};

#[cfg(not(windows))]
pub struct OverlayWindow;
