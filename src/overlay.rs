#[cfg(windows)]
mod win {
    use crate::settings::OverlaySettings;
    use anyhow::Result;
    use std::ffi::c_void;
    use std::path::Path;
    use std::ptr::null_mut;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{
        BOOL, COLORREF, ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HANDLE, HWND, LPARAM, LRESULT, POINT,
        RECT, SIZE, WPARAM,
    };
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateCompatibleDC, CreateDIBSection, CreateFontIndirectW, CreateFontW,
        CreateRoundRectRgn, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
        FillRect, FrameRect, GetTextExtentPoint32W, GetTextMetricsW, LineTo, MoveToEx,
        RedrawWindow, SelectObject, SetBkMode, SetTextColor, SetWindowRgn, AC_SRC_ALPHA,
        AC_SRC_OVER, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION,
        CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DT_LEFT, DT_NOCLIP,
        DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, HBRUSH, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
        RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW, TEXTMETRICW, TRANSPARENT,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
        RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE,
        REG_OPTION_NON_VOLATILE, REG_SZ,
    };
    use windows::Win32::UI::Controls::Dialogs::{
        ChooseColorW, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW,
    };
    use windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
    use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_GETTASKBARPOS, APPBARDATA};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EnumChildWindows,
        FindWindowExW, FindWindowW, GetCursorPos, GetMessageW, GetWindowLongPtrW, GetWindowRect,
        GetWindowTextW, LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassW, SendMessageW,
        SetForegroundWindow, SetTimer, SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow,
        SystemParametersInfoW, TranslateMessage, UpdateLayeredWindow, BN_CLICKED, CS_HREDRAW,
        CS_VREDRAW, CW_USEDEFAULT, EN_CHANGE, ES_AUTOHSCROLL, ES_NUMBER, GWLP_USERDATA, HMENU,
        HTCLIENT, IDC_ARROW, MSG, NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SWP_NOACTIVATE,
        SWP_SHOWWINDOW, SW_SHOW, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, ULW_ALPHA,
        WA_INACTIVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATE, WM_CLOSE, WM_COMMAND,
        WM_CONTEXTMENU, WM_CREATE, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY,
        WM_DISPLAYCHANGE, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCHITTEST,
        WM_PAINT, WM_RBUTTONUP, WM_SETFONT, WM_SETTINGCHANGE, WM_TIMER, WNDCLASSW, WS_BORDER,
        WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        WS_TABSTOP, WS_VISIBLE,
    };

    const WINDOW_WIDTH: i32 = 92;
    const WINDOW_HEIGHT: i32 = 42;
    const TIMER_REPOSITION: usize = 2;
    const OVERLAY_PADDING_X: i32 = 3;
    const OVERLAY_PADDING_Y: i32 = 3;
    const LABEL_LEFT: i32 = OVERLAY_PADDING_X;
    const PREFIX_VALUE_GAP: i32 = 4;
    const TRAY_GAP: i32 = 2;
    const MENU_SETTINGS: usize = 101;
    const TIMER_TOOLTIP: usize = 3;
    const TOOLTIP_WIDTH: i32 = 260;
    const TOOLTIP_HEIGHT: i32 = 80;
    const SETTINGS_WIDTH_BASE: i32 = 304;
    const SETTINGS_ROW_HEIGHT: i32 = 40;
    const SETTINGS_HEIGHT_BASE: i32 = SETTINGS_ROW_HEIGHT * 6;
    const SETTINGS_LABEL_LEFT: i32 = 20;
    const SETTINGS_VALUE_LEFT: i32 = 178;
    const SETTINGS_VALUE_RIGHT: i32 = 270;
    const SETTINGS_CONTROL_HEIGHT: i32 = 26;
    const IDC_FONT_SIZE: i32 = 1001;
    const IDC_CLAUDE_PREFIX: i32 = 1003;
    const IDC_CODEX_PREFIX: i32 = 1004;
    const IDC_EXIT: i32 = 1006;
    const FONT_MIN: i32 = 8;
    const FONT_MAX: i32 = 32;
    const STARTUP_VALUE_NAME: &str = "CcBalanceOverlay";
    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const COLOR_SWATCH_RECT: RECT = RECT {
        left: SETTINGS_VALUE_LEFT,
        top: SETTINGS_ROW_HEIGHT + 11,
        right: SETTINGS_VALUE_LEFT + 48,
        bottom: SETTINGS_ROW_HEIGHT + 29,
    };
    const COLOR_ROW_RECT: RECT = RECT {
        left: 0,
        top: SETTINGS_ROW_HEIGHT,
        right: SETTINGS_WIDTH_BASE,
        bottom: SETTINGS_ROW_HEIGHT * 2,
    };
    const EXIT_ROW_RECT: RECT = RECT {
        left: 0,
        top: SETTINGS_ROW_HEIGHT * 5,
        right: SETTINGS_WIDTH_BASE,
        bottom: SETTINGS_ROW_HEIGHT * 6,
    };
    const STARTUP_ROW_RECT: RECT = RECT {
        left: 0,
        top: SETTINGS_ROW_HEIGHT * 4,
        right: SETTINGS_WIDTH_BASE,
        bottom: SETTINGS_ROW_HEIGHT * 5,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct OverlayLayout {
        width: i32,
        height: i32,
        line_height: i32,
        first_line_top: i32,
        value_left: i32,
        value_right: i32,
    }

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
                        WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_LAYERED.0,
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
                let tray_rect = tray_notify_rect();
                let window_rect = current_window_rect(self.hwnd);
                let (x, y) = overlay_position(rect, tray_rect, window_rect);
                unsafe {
                    let _ = SetWindowPos(
                        self.hwnd,
                        HWND(-1isize as *mut c_void),
                        x,
                        y,
                        window_rect.right - window_rect.left,
                        window_rect.bottom - window_rect.top,
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

    fn current_window_rect(hwnd: HWND) -> RECT {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: WINDOW_WIDTH,
            bottom: WINDOW_HEIGHT,
        };
        unsafe {
            let _ = GetWindowRect(hwnd, &mut rect);
        }
        rect
    }

    extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if msg == WM_CREATE {
                let createstruct =
                    lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
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
            let width = TOOLTIP_WIDTH;
            let height = TOOLTIP_HEIGHT;
            let x = rect.right - width;
            let y = rect.top - height - 6;
            let _ = SetWindowPos(
                self.tooltip_hwnd,
                HWND(-1isize as *mut c_void),
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }

        unsafe fn hide_tooltip(&self) {
            if !self.tooltip_hwnd.is_invalid() {
                let _ = ShowWindow(
                    self.tooltip_hwnd,
                    windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
                );
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

            let mut owner_rect = RECT::default();
            let settings_width = SETTINGS_WIDTH_BASE;
            let settings_height = SETTINGS_HEIGHT_BASE;
            let (x, y) = if GetWindowRect(self.hwnd, &mut owner_rect).is_ok() {
                let above = owner_rect.top - settings_height - 8;
                let below = owner_rect.bottom + 8;
                (owner_rect.left, if above >= 0 { above } else { below })
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
                settings_width,
                settings_height,
                None,
                HMENU(null_mut()),
                instance,
                Some(raw),
            ) {
                Ok(hwnd) => hwnd,
                Err(_) => return,
            };
            self.settings_hwnd = hwnd;
            let rgn = CreateRoundRectRgn(0, 0, settings_width + 1, settings_height + 1, 12, 12);
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
            set_control_text(
                self.settings_hwnd,
                IDC_FONT_SIZE,
                &self.settings.font_size.to_string(),
            );
            set_control_text(
                self.settings_hwnd,
                IDC_CLAUDE_PREFIX,
                &self.settings.claude_prefix,
            );
            set_control_text(
                self.settings_hwnd,
                IDC_CODEX_PREFIX,
                &self.settings.codex_prefix,
            );
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

        unsafe fn apply_startup_enabled(&mut self, enabled: bool) {
            if let Err(err) = set_startup_enabled(enabled) {
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
    }

    extern "system" fn settings_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if msg == WM_CREATE {
                let createstruct =
                    lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
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
                    let x = loword(lparam.0 as usize) as i16 as i32;
                    let y = hiword(lparam.0 as usize) as i16 as i32;
                    if !ptr.is_null() && point_in_rect_base(x, y, &COLOR_ROW_RECT) {
                        (*ptr).open_color_dialog();
                        return LRESULT(0);
                    }
                    if !ptr.is_null() && point_in_rect_base(x, y, &STARTUP_ROW_RECT) {
                        let enabled = !startup_enabled();
                        (*ptr).apply_startup_enabled(enabled);
                        let _ = RedrawWindow(
                            hwnd,
                            None,
                            None,
                            RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN,
                        );
                        return LRESULT(0);
                    }
                    if point_in_rect_base(x, y, &EXIT_ROW_RECT) {
                        PostQuitMessage(0);
                        return LRESULT(0);
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
                    let hdc = windows::Win32::Graphics::Gdi::HDC(wparam.0 as *mut c_void);
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, COLORREF(0x00000000));
                    let brush = CreateSolidBrush(COLORREF(0x00ffffff));
                    LRESULT(brush.0 as isize)
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
        let edit_font = settings_panel_font();
        create_edit(
            hwnd,
            IDC_FONT_SIZE,
            SETTINGS_VALUE_LEFT,
            centered_control_top(0),
            SETTINGS_VALUE_RIGHT - SETTINGS_VALUE_LEFT,
            SETTINGS_CONTROL_HEIGHT,
            ES_NUMBER | ES_AUTOHSCROLL,
            edit_font,
        );

        create_edit(
            hwnd,
            IDC_CLAUDE_PREFIX,
            SETTINGS_VALUE_LEFT,
            centered_control_top(2),
            SETTINGS_VALUE_RIGHT - SETTINGS_VALUE_LEFT,
            SETTINGS_CONTROL_HEIGHT,
            ES_AUTOHSCROLL,
            edit_font,
        );

        create_edit(
            hwnd,
            IDC_CODEX_PREFIX,
            SETTINGS_VALUE_LEFT,
            centered_control_top(3),
            SETTINGS_VALUE_RIGHT - SETTINGS_VALUE_LEFT,
            SETTINGS_CONTROL_HEIGHT,
            ES_AUTOHSCROLL,
            edit_font,
        );
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

    fn row_top(row: i32) -> i32 {
        row * SETTINGS_ROW_HEIGHT
    }

    fn row_bottom(row: i32) -> i32 {
        row_top(row + 1)
    }

    fn centered_control_top(row: i32) -> i32 {
        row_top(row) + (SETTINGS_ROW_HEIGHT - SETTINGS_CONTROL_HEIGHT) / 2
    }

    unsafe fn ui_font(base_height: i32) -> windows::Win32::Graphics::Gdi::HFONT {
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
            if base_height != 0 {
                font.lfHeight = base_height;
            }
            font.lfWeight = FW_NORMAL.0 as i32;
            return CreateFontIndirectW(&font);
        }
        CreateFontW(
            base_height,
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

    unsafe fn settings_panel_font() -> windows::Win32::Graphics::Gdi::HFONT {
        ui_font(0)
    }

    unsafe fn tooltip_font() -> windows::Win32::Graphics::Gdi::HFONT {
        ui_font(-20)
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
        let font = settings_panel_font();
        let _ = EnumChildWindows(
            hwnd,
            Some(refresh_settings_child_font),
            LPARAM(font.0 as isize),
        );
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
            right: SETTINGS_WIDTH_BASE,
            bottom: SETTINGS_HEIGHT_BASE,
        };
        let bg = CreateSolidBrush(COLORREF(0x00ffffff));
        FillRect(hdc, &rect, HBRUSH(bg.0));
        let border = CreateSolidBrush(COLORREF(0x00d0d0d0));
        FrameRect(hdc, &rect, HBRUSH(border.0));

        let old_pen = SelectObject(
            hdc,
            windows::Win32::Graphics::Gdi::CreatePen(
                windows::Win32::Graphics::Gdi::PS_SOLID,
                1,
                COLORREF(0x00e5e5e5),
            ),
        );
        for y in [row_top(1), row_top(2), row_top(3), row_top(4), row_top(5)] {
            let _ = MoveToEx(hdc, 0, y, None);
            let _ = LineTo(hdc, SETTINGS_WIDTH_BASE, y);
        }
        paint_settings_text(hdc);
        paint_palette_swatch(hdc, HBRUSH(border.0));
        paint_startup_row(hdc, HBRUSH(border.0));
        paint_exit_row(hdc);
        let pen = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
        let _ = DeleteObject(bg);
        let _ = DeleteObject(border);
        let _ = EndPaint(hwnd, &ps);
    }

    unsafe fn paint_settings_text(hdc: windows::Win32::Graphics::Gdi::HDC) {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00000000));
        let font = settings_panel_font();
        let old = SelectObject(hdc, font);
        draw_panel_text(
            hdc,
            "字号",
            SETTINGS_LABEL_LEFT,
            row_top(0),
            SETTINGS_VALUE_LEFT - 12,
            row_bottom(0),
        );
        draw_panel_text(
            hdc,
            "颜色",
            SETTINGS_LABEL_LEFT,
            row_top(1),
            SETTINGS_VALUE_LEFT - 12,
            row_bottom(1),
        );
        draw_panel_text(
            hdc,
            "Claude 前缀",
            SETTINGS_LABEL_LEFT,
            row_top(2),
            SETTINGS_VALUE_LEFT - 12,
            row_bottom(2),
        );
        draw_panel_text(
            hdc,
            "Codex 前缀",
            SETTINGS_LABEL_LEFT,
            row_top(3),
            SETTINGS_VALUE_LEFT - 12,
            row_bottom(3),
        );
        SelectObject(hdc, old);
        let _ = DeleteObject(font);
    }

    unsafe fn draw_panel_text(
        hdc: windows::Win32::Graphics::Gdi::HDC,
        text: &str,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) {
        let mut rect = RECT {
            left,
            top,
            right,
            bottom,
        };
        let mut text = wide(text);
        DrawTextW(
            hdc,
            &mut text,
            &mut rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
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

    unsafe fn paint_startup_row(hdc: windows::Win32::Graphics::Gdi::HDC, border: HBRUSH) {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00000000));
        let font = settings_panel_font();
        let old = SelectObject(hdc, font);

        let box_rect = RECT {
            left: SETTINGS_VALUE_LEFT,
            top: STARTUP_ROW_RECT.top + 10,
            right: SETTINGS_VALUE_LEFT + 20,
            bottom: STARTUP_ROW_RECT.top + 30,
        };
        if startup_enabled() {
            let fill = CreateSolidBrush(COLORREF(0x00d77800));
            FillRect(hdc, &box_rect, HBRUSH(fill.0));
            let _ = DeleteObject(fill);
            SetTextColor(hdc, COLORREF(0x00ffffff));
            let check_font = ui_font(-18);
            let old_check = SelectObject(hdc, check_font);
            let mut check_rect = box_rect;
            let mut check = wide("✓");
            DrawTextW(
                hdc,
                &mut check,
                &mut check_rect,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
            SelectObject(hdc, old_check);
            let _ = DeleteObject(check_font);
            SetTextColor(hdc, COLORREF(0x00000000));
        } else {
            FrameRect(hdc, &box_rect, border);
        }

        let mut text_rect = RECT {
            left: SETTINGS_LABEL_LEFT,
            top: STARTUP_ROW_RECT.top,
            right: SETTINGS_VALUE_LEFT - 12,
            bottom: STARTUP_ROW_RECT.bottom,
        };
        let mut text = wide("开机启动");
        DrawTextW(
            hdc,
            &mut text,
            &mut text_rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(hdc, old);
        let _ = DeleteObject(font);
    }

    unsafe fn paint_exit_row(hdc: windows::Win32::Graphics::Gdi::HDC) {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00000000));
        let font = settings_panel_font();
        let old = SelectObject(hdc, font);
        let mut text_rect = RECT {
            left: SETTINGS_LABEL_LEFT,
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
        let _ = DeleteObject(font);
    }

    extern "system" fn tooltip_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
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
        let font = tooltip_font();
        let old = SelectObject(hdc, font);
        let mut text_rect = RECT {
            left: 12,
            top: 10,
            right: TOOLTIP_WIDTH - 12,
            bottom: TOOLTIP_HEIGHT - 10,
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
        let font = overlay_text_font(settings.font_size);
        let old_font = SelectObject(hdc, font);
        let layout = overlay_layout(hdc, text);

        let mut bits: *mut c_void = null_mut();
        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: layout.width,
                biHeight: -layout.height,
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
                SelectObject(hdc, old_font);
                let _ = DeleteObject(font);
                let _ = DeleteDC(hdc);
                return;
            }
        };
        if bitmap.is_invalid() || bits.is_null() {
            let _ = DeleteObject(bitmap);
            SelectObject(hdc, old_font);
            let _ = DeleteObject(font);
            let _ = DeleteDC(hdc);
            return;
        }

        let old_bitmap = SelectObject(hdc, bitmap);
        let pixels = std::slice::from_raw_parts_mut(
            bits as *mut u8,
            (layout.width * layout.height * 4) as usize,
        );
        pixels.fill(0);
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = 1;
        }

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00ffffff));
        for (index, line) in text.lines().take(2).enumerate() {
            draw_overlay_line(
                hdc,
                line,
                layout.first_line_top + index as i32 * layout.line_height,
                layout,
            );
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
            cx: layout.width,
            cy: layout.height,
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

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn loword(value: usize) -> u16 {
        (value & 0xffff) as u16
    }

    fn hiword(value: usize) -> u16 {
        ((value >> 16) & 0xffff) as u16
    }

    fn point_in_rect_base(x: i32, y: i32, rect: &RECT) -> bool {
        x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
    }

    fn overlay_position(taskbar: RECT, tray: Option<RECT>, window: RECT) -> (i32, i32) {
        let anchor_left = tray.map(|rect| rect.left).unwrap_or(taskbar.right - 590);
        let window_width = (window.right - window.left).max(WINDOW_WIDTH);
        let window_height = (window.bottom - window.top).max(WINDOW_HEIGHT);
        let x = anchor_left - window_width - TRAY_GAP;
        let taskbar_height = taskbar.bottom - taskbar.top;
        let y = taskbar.top + (taskbar_height - window_height).max(0) / 2;
        (x, y)
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

    fn startup_enabled() -> bool {
        let Ok(exe) = std::env::current_exe() else {
            return false;
        };
        let expected = startup_command_for_exe(&exe);
        unsafe { startup_value().as_deref() == Some(expected.as_str()) }
    }

    fn set_startup_enabled(enabled: bool) -> Result<()> {
        unsafe {
            if enabled {
                let exe = std::env::current_exe()?;
                let command = startup_command_for_exe(&exe);
                set_startup_value(&command)?;
                let saved = startup_value().unwrap_or_default();
                if saved != command {
                    anyhow::bail!(
                        "startup value was not persisted correctly: expected `{command}`, got `{saved}`"
                    );
                }
            } else {
                delete_startup_value()?;
                if startup_value().is_some() {
                    anyhow::bail!("startup value still exists after deletion");
                }
            }
        }
        Ok(())
    }

    fn startup_command_for_exe(exe: &Path) -> String {
        format!("\"{}\"", normalize_startup_path(exe))
    }

    fn normalize_startup_path(exe: &Path) -> String {
        let raw = exe.to_string_lossy();
        if let Some(path) = raw.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{path}")
        } else if let Some(path) = raw.strip_prefix(r"\\?\") {
            path.to_string()
        } else {
            raw.into_owned()
        }
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

    unsafe fn open_run_key(
        access: windows::Win32::System::Registry::REG_SAM_FLAGS,
    ) -> Result<HKEY> {
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
            KEY_SET_VALUE | KEY_QUERY_VALUE,
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

    unsafe fn overlay_layout(hdc: windows::Win32::Graphics::Gdi::HDC, text: &str) -> OverlayLayout {
        let mut metrics = TEXTMETRICW::default();
        let _ = GetTextMetricsW(hdc, &mut metrics);
        let line_height = (metrics.tmHeight + metrics.tmExternalLeading).max(18);

        let mut label_width = 0;
        let mut value_width = 0;
        for line in text.lines().take(2) {
            let (label, value) = line.split_once(' ').unwrap_or((line, ""));
            label_width = label_width.max(text_width(hdc, label));
            value_width = value_width.max(text_width(hdc, value));
        }

        let value_left = LABEL_LEFT + label_width + PREFIX_VALUE_GAP;
        let value_right = value_left + value_width;
        let width = (value_right + OVERLAY_PADDING_X).max(WINDOW_WIDTH);
        let height = (OVERLAY_PADDING_Y * 2 + line_height * 2).max(WINDOW_HEIGHT);

        OverlayLayout {
            width,
            height,
            line_height,
            first_line_top: OVERLAY_PADDING_Y,
            value_left,
            value_right,
        }
    }

    unsafe fn text_width(hdc: windows::Win32::Graphics::Gdi::HDC, value: &str) -> i32 {
        let mut size = SIZE::default();
        let wide_text = wide(value);
        let text_slice = &wide_text[..wide_text.len().saturating_sub(1)];
        if !text_slice.is_empty() {
            let _ = GetTextExtentPoint32W(hdc, text_slice, &mut size);
        }
        size.cx
    }

    unsafe fn draw_overlay_line(
        hdc: windows::Win32::Graphics::Gdi::HDC,
        line: &str,
        top: i32,
        layout: OverlayLayout,
    ) {
        let (label, value) = line.split_once(' ').unwrap_or((line, ""));
        let label_wide = wide(label);
        let mut label_rect = RECT {
            left: LABEL_LEFT,
            top,
            right: layout.value_left - PREFIX_VALUE_GAP + 1,
            bottom: top + layout.line_height,
        };
        let mut value_rect = RECT {
            left: layout.value_left,
            top,
            right: layout.value_right,
            bottom: top + layout.line_height,
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
            DT_LEFT | DT_SINGLELINE | DT_NOCLIP,
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

        #[test]
        fn positions_overlay_vertically_centered_in_bottom_taskbar() {
            let taskbar = RECT {
                left: 0,
                top: 996,
                right: 1920,
                bottom: 1080,
            };
            let tray = RECT {
                left: 1520,
                top: 996,
                right: 1920,
                bottom: 1080,
            };
            let window = RECT {
                left: 0,
                top: 0,
                right: WINDOW_WIDTH,
                bottom: WINDOW_HEIGHT,
            };

            assert_eq!(
                overlay_position(taskbar, Some(tray), window),
                (
                    1520 - WINDOW_WIDTH - TRAY_GAP,
                    996 + (84 - WINDOW_HEIGHT) / 2
                )
            );
        }

        #[test]
        fn positions_overlay_using_actual_scaled_window_size() {
            let taskbar = RECT {
                left: 0,
                top: 1704,
                right: 2880,
                bottom: 1800,
            };
            let tray = RECT {
                left: 2160,
                top: 1704,
                right: 2880,
                bottom: 1800,
            };
            let scaled_window = RECT {
                left: 2066,
                top: 1748,
                right: 2158,
                bottom: 1790,
            };

            assert_eq!(
                overlay_position(taskbar, Some(tray), scaled_window),
                (2160 - 92 - TRAY_GAP, 1704 + (96 - 42) / 2)
            );
        }

        #[test]
        fn positions_overlay_using_dynamic_window_width() {
            let taskbar = RECT {
                left: 0,
                top: 1704,
                right: 2880,
                bottom: 1800,
            };
            let tray = RECT {
                left: 2160,
                top: 1704,
                right: 2880,
                bottom: 1800,
            };
            let wide_window = RECT {
                left: 2010,
                top: 1730,
                right: 2158,
                bottom: 1774,
            };

            assert_eq!(
                overlay_position(taskbar, Some(tray), wide_window),
                (2160 - 148 - TRAY_GAP, 1704 + (96 - 44) / 2)
            );
        }

        #[test]
        fn startup_command_normalizes_extended_length_paths() {
            let normal = std::path::Path::new(r"C:\Tools\cc-balance-overlay.exe");
            assert_eq!(
                startup_command_for_exe(normal),
                r#""C:\Tools\cc-balance-overlay.exe""#
            );

            let extended = std::path::Path::new(r"\\?\C:\Tools\cc-balance-overlay.exe");
            assert_eq!(
                startup_command_for_exe(extended),
                r#""C:\Tools\cc-balance-overlay.exe""#
            );

            let unc = std::path::Path::new(r"\\?\UNC\server\share\cc-balance-overlay.exe");
            assert_eq!(
                startup_command_for_exe(unc),
                r#""\\server\share\cc-balance-overlay.exe""#
            );
        }
    }
}

#[cfg(windows)]
pub use win::{OverlayCommand, OverlayWindow};

#[cfg(not(windows))]
pub struct OverlayWindow;
