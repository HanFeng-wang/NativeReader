use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    FrameRect, GetDC, GetTextMetricsW, IntersectClipRect, InvalidateRect, MapWindowPoints,
    ReleaseDC, RestoreDC, SaveDC, ScreenToClient, SelectObject, SetBkColor, SetBkMode,
    SetTextColor, UpdateWindow, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DT_CENTER,
    DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, FW_NORMAL,
    FW_SEMIBOLD, HBRUSH, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, TEXTMETRICW, TRANSPARENT,
};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, LoadLibraryW};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY,
    OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Controls::RichEdit::{
    CFE_LINK, CFM_LINK, CHARFORMAT2W, CHARRANGE, EDITSTREAM, EM_EXGETSEL, EM_EXSETSEL,
    EM_FINDTEXTEXW, EM_GETTEXTRANGE, EM_SETBKGNDCOLOR, EM_SETCHARFORMAT, EM_SETEVENTMASK,
    EM_SETTARGETDEVICE, EM_SETUNDOLIMIT, EM_STREAMIN, ENLINK, ENM_CHANGE, ENM_LINK, ENM_SCROLL,
    ENM_SELCHANGE, EN_LINK, EN_SELCHANGE, FINDTEXTEXW, SCF_SELECTION, SF_RTF, TEXTRANGEW,
};
use windows::Win32::UI::Controls::{
    SetWindowTheme, EM_CHARFROMPOS, EM_GETFIRSTVISIBLELINE, EM_GETLINECOUNT, EM_LINEFROMCHAR,
    EM_LINEINDEX, EM_LINESCROLL, EM_POSFROMCHAR, EM_REPLACESEL, EM_SCROLLCARET, EM_SETMARGINS,
    EM_SETMODIFY, EM_SETREADONLY, EM_SETSEL,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetFocus, GetKeyState, SetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
    VK_0, VK_1, VK_2, VK_ADD, VK_C, VK_CONTROL, VK_D, VK_ESCAPE, VK_F, VK_F3, VK_F5, VK_O,
    VK_OEM_MINUS, VK_OEM_PLUS, VK_RETURN, VK_S, VK_SHIFT, VK_SUBTRACT, VK_T, VK_V, VK_W,
};
use windows::Win32::UI::Shell::{
    DefSubclassProc, DragAcceptFiles, DragFinish, DragQueryFileW, SetWindowSubclass, ShellExecuteW,
    HDROP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRect, AppendMenuW, CreateAcceleratorTableW, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyAcceleratorTable, DestroyMenu, DestroyWindow, DispatchMessageW,
    FindWindowExW, GetAncestor, GetClientRect, GetCursorPos, GetMessageW, GetWindowLongPtrW,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsIconic, KillTimer, LoadCursorW,
    LoadIconW, LoadImageW, MessageBoxW, MoveWindow, PostMessageW, PostQuitMessage, RegisterClassW,
    SendMessageW, SetCursor, SetForegroundWindow, SetTimer, SetWindowLongPtrW, SetWindowPos,
    SetWindowTextW, ShowWindow, TrackPopupMenu, TranslateAcceleratorW, TranslateMessage, ACCEL,
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, EN_CHANGE, FCONTROL, FSHIFT, FVIRTKEY, GA_ROOT,
    GWLP_USERDATA, HACCEL, HICON, HMENU, HWND_NOTOPMOST, HWND_TOPMOST, IDC_ARROW, IDC_HAND,
    IDI_APPLICATION, IDNO, IDYES, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MB_ICONERROR,
    MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_YESNOCANCEL, MF_GRAYED, MF_SEPARATOR,
    MF_STRING, MINMAXINFO, MSG, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE,
    SW_RESTORE, SW_SHOW, SW_SHOWNORMAL, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TPM_TOPALIGN, WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATE, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU,
    WM_COPY, WM_COPYDATA, WM_CREATE, WM_CTLCOLOREDIT, WM_CUT, WM_DESTROY, WM_DPICHANGED,
    WM_DROPFILES, WM_ERASEBKGND, WM_GETMINMAXINFO, WM_HSCROLL, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE, WM_NCDESTROY, WM_NOTIFY, WM_PAINT,
    WM_PASTE, WM_SETCURSOR, WM_SETFOCUS, WM_SETFONT, WM_SIZE, WM_THEMECHANGED, WM_TIMER, WM_UNDO,
    WM_VSCROLL, WNDCLASSW, WS_BORDER, WS_CHILD, WS_CLIPCHILDREN, WS_HSCROLL, WS_OVERLAPPEDWINDOW,
    WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};

use crate::app::{copy_text_to_clipboard, read_clipboard_text, Document, Settings};
use crate::associations;
use crate::markdown::{self, MarkdownHeading};
use crate::syntax;
use crate::theme::{self, Color, Theme};

const CLASS_NAME: PCWSTR = w!("NativeReaderMainWindow");
const EDIT_CLASS: PCWSTR = w!("RICHEDIT50W");
const COPYDATA_ACTIVATE_PATH: usize = 0x4E52_5041;
static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);
const TIMER_AUTO_RELOAD: usize = 1;
const TIMER_RESIZE: usize = 2;
const EM_SETCUEBANNER: u32 = 0x1501;
const EM_SETLIMITTEXT: u32 = 0x00C5;
const FR_DOWN: usize = 1;
const FR_WHOLEWORD: usize = 2;
const FR_MATCHCASE: usize = 4;
const WM_MOUSELEAVE: u32 = 0x02A3;
const AUTO_WRAP_LONG_LINE_LIMIT: usize = 8_000;

const CMD_OPEN: u16 = 101;
const CMD_SAVE: u16 = 102;
const CMD_SAVE_AS: u16 = 103;
const CMD_CLIPBOARD: u16 = 104;
const CMD_COPY_ALL: u16 = 105;
const CMD_TOGGLE_VIEW: u16 = 106;
const CMD_SOURCE: u16 = 107;
const CMD_FIND: u16 = 108;
const CMD_FIND_NEXT: u16 = 109;
const CMD_FIND_PREV: u16 = 110;
const CMD_FIND_CLOSE: u16 = 111;
const CMD_REPLACE: u16 = 112;
const CMD_REPLACE_ALL: u16 = 113;
const CMD_TOPMOST: u16 = 114;
const CMD_WRAP: u16 = 115;
const CMD_THEME: u16 = 116;
const CMD_ZOOM_IN: u16 = 117;
const CMD_ZOOM_OUT: u16 = 118;
const CMD_ZOOM_RESET: u16 = 119;
const CMD_RELOAD: u16 = 120;
const CMD_BROWSER: u16 = 121;
const CMD_MATCH_CASE: u16 = 122;
const CMD_WHOLE_WORD: u16 = 123;
const CMD_EDIT_UNDO: u16 = 124;
const CMD_EDIT_CUT: u16 = 125;
const CMD_EDIT_COPY: u16 = 126;
const CMD_EDIT_PASTE: u16 = 127;
const CMD_SELECT_ALL: u16 = 128;
const CMD_TOC: u16 = 129;
const CMD_TOC_BASE: u16 = 1000;

#[derive(Clone, Copy)]
struct Fonts {
    ui: HFONT,
    ui_bold: HFONT,
    small: HFONT,
    code: HFONT,
}

impl Fonts {
    fn empty() -> Self {
        Self {
            ui: HFONT(std::ptr::null_mut()),
            ui_bold: HFONT(std::ptr::null_mut()),
            small: HFONT(std::ptr::null_mut()),
            code: HFONT(std::ptr::null_mut()),
        }
    }

    unsafe fn destroy(self) {
        for font in [self.ui, self.ui_bold, self.small, self.code] {
            if !font.0.is_null() {
                let _ = DeleteObject(HGDIOBJ(font.0));
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Brushes {
    bg: HBRUSH,
    panel: HBRUSH,
    hover: HBRUSH,
    border: HBRUSH,
    accent: HBRUSH,
}

impl Brushes {
    fn empty() -> Self {
        Self {
            bg: HBRUSH(std::ptr::null_mut()),
            panel: HBRUSH(std::ptr::null_mut()),
            hover: HBRUSH(std::ptr::null_mut()),
            border: HBRUSH(std::ptr::null_mut()),
            accent: HBRUSH(std::ptr::null_mut()),
        }
    }

    unsafe fn destroy(self) {
        for brush in [self.bg, self.panel, self.hover, self.border, self.accent] {
            if !brush.0.is_null() {
                let _ = DeleteObject(HGDIOBJ(brush.0));
            }
        }
    }
}

#[derive(Clone)]
struct HitButton {
    id: u16,
    rect: RECT,
    label: String,
    enabled: bool,
    active: bool,
}

struct FindLayout {
    find_edit: RECT,
    replace_edit: RECT,
    buttons: Vec<HitButton>,
}

struct App {
    hwnd: HWND,
    instance: HINSTANCE,
    edit: HWND,
    find_edit: HWND,
    replace_edit: HWND,
    doc: Document,
    settings: Settings,
    theme: Theme,
    fonts: Fonts,
    brushes: Brushes,
    dpi: u32,
    find_visible: bool,
    match_case: bool,
    whole_word: bool,
    hover_id: Option<u16>,
    rendered_links: Vec<(String, String)>,
    link_ranges: Vec<(CHARRANGE, String)>,
    rendered_code_blocks: Vec<String>,
    rendered_headings: Vec<MarkdownHeading>,
    heading_ranges: Vec<(CHARRANGE, usize)>,
    status_message: String,
    search_summary: String,
    line: i32,
    column: i32,
    total_lines: i32,
    current_line: i32,
    source_line_starts: Vec<i32>,
    longest_line_bytes: usize,
    auto_wrap_long_lines: bool,
    last_stamp: Option<(u64, Option<SystemTime>)>,
    suppress_change: bool,
    external_notice: bool,
    last_preview_width: i32,
    pending_initial: Option<PathBuf>,
    restore_topmost_on_activate: bool,
    buttons: Vec<HitButton>,
    find_layout: Option<FindLayout>,
    tracking_mouse: bool,
}

impl App {
    fn new(settings: Settings, initial: Option<PathBuf>) -> Self {
        let theme = theme::for_mode(settings.dark);
        Self {
            hwnd: HWND(std::ptr::null_mut()),
            instance: HINSTANCE(std::ptr::null_mut()),
            edit: HWND(std::ptr::null_mut()),
            find_edit: HWND(std::ptr::null_mut()),
            replace_edit: HWND(std::ptr::null_mut()),
            doc: Document::welcome(),
            settings,
            theme,
            fonts: Fonts::empty(),
            brushes: Brushes::empty(),
            dpi: 96,
            find_visible: false,
            match_case: false,
            whole_word: false,
            hover_id: None,
            rendered_links: Vec::new(),
            link_ranges: Vec::new(),
            rendered_code_blocks: Vec::new(),
            rendered_headings: Vec::new(),
            heading_ranges: Vec::new(),
            status_message: "就绪".to_string(),
            search_summary: String::new(),
            line: 1,
            column: 1,
            total_lines: 1,
            current_line: 0,
            source_line_starts: Vec::new(),
            longest_line_bytes: 0,
            auto_wrap_long_lines: false,
            last_stamp: None,
            suppress_change: false,
            external_notice: false,
            last_preview_width: 0,
            pending_initial: initial,
            restore_topmost_on_activate: false,
            buttons: Vec::new(),
            find_layout: None,
            tracking_mouse: false,
        }
    }

    fn scale(&self, value: i32) -> i32 {
        (value * self.dpi as i32 / 96).max(1)
    }

    fn base_pt(&self) -> f32 {
        10.5 * self.settings.zoom as f32 / 100.0
    }

    unsafe fn create_children(&mut self) -> Result<(), String> {
        self.dpi = {
            let dpi = GetDpiForWindow(self.hwnd);
            if dpi == 0 {
                96
            } else {
                dpi
            }
        };
        self.rebuild_resources();
        let _ = LoadLibraryW(w!("Msftedit.dll"));
        let edit_style = WS_CHILD
            | WS_VISIBLE
            | WS_VSCROLL
            | WS_HSCROLL
            | WS_TABSTOP
            | WINDOW_STYLE((0x0004 | 0x0100 | 0x0040 | 0x0080 | 0x1000) as u32);
        self.edit = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            EDIT_CLASS,
            w!(""),
            edit_style,
            0,
            0,
            10,
            10,
            Some(self.hwnd),
            None,
            Some(self.instance),
            None,
        )
        .map_err(|error| format!("无法创建编辑控件：{error}"))?;
        SetWindowSubclass(self.edit, Some(edit_subclass), 1, self as *mut App as usize);
        SendMessageW(
            self.edit,
            EM_SETUNDOLIMIT,
            Some(WPARAM(200)),
            Some(LPARAM(0)),
        );
        SendMessageW(
            self.edit,
            EM_SETLIMITTEXT,
            Some(WPARAM(0x7FFF_FFFE)),
            Some(LPARAM(0)),
        );

        let find_style = WS_CHILD | WS_BORDER | WS_TABSTOP | WINDOW_STYLE(0x0080);
        self.find_edit = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            find_style,
            0,
            0,
            10,
            10,
            Some(self.hwnd),
            None,
            Some(self.instance),
            None,
        )
        .map_err(|error| format!("无法创建查找框：{error}"))?;
        self.replace_edit = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            find_style,
            0,
            0,
            10,
            10,
            Some(self.hwnd),
            None,
            Some(self.instance),
            None,
        )
        .map_err(|error| format!("无法创建替换框：{error}"))?;
        for control in [self.find_edit, self.replace_edit] {
            SendMessageW(
                control,
                WM_SETFONT,
                Some(WPARAM(self.fonts.ui.0 as usize)),
                Some(LPARAM(1)),
            );
            SetWindowSubclass(control, Some(search_subclass), 2, self as *mut App as usize);
        }
        set_cue_banner(self.find_edit, "查找内容");
        set_cue_banner(self.replace_edit, "替换为（仅源码可替换）");
        ShowWindow(self.find_edit, SW_HIDE);
        ShowWindow(self.replace_edit, SW_HIDE);

        DragAcceptFiles(self.hwnd, true);
        self.apply_theme_to_controls();
        self.apply_topmost();
        self.layout();

        if let Some(path) = self.pending_initial.take() {
            if !self.open_path_without_prompt(&path) {
                self.rebuild_from_source();
            }
        } else {
            self.rebuild_from_source();
        }
        self.refresh_title();
        self.refresh_status();
        SetTimer(Some(self.hwnd), TIMER_AUTO_RELOAD, 1400, None);
        Ok(())
    }

    unsafe fn rebuild_resources(&mut self) {
        self.fonts.destroy();
        self.brushes.destroy();
        let dpi = self.dpi as i32;
        let font = |points: i32, weight: i32, face: PCWSTR| {
            CreateFontW(
                -(points * dpi / 72).max(9),
                0,
                0,
                0,
                weight,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                0,
                face,
            )
        };
        self.fonts = Fonts {
            ui: font(10, FW_NORMAL.0 as i32, w!("Microsoft YaHei UI")),
            ui_bold: font(10, FW_SEMIBOLD.0 as i32, w!("Microsoft YaHei UI")),
            small: font(9, FW_NORMAL.0 as i32, w!("Microsoft YaHei UI")),
            code: font(10, FW_NORMAL.0 as i32, w!("Consolas")),
        };
        self.brushes = Brushes {
            bg: CreateSolidBrush(to_colorref(self.theme.bg)),
            panel: CreateSolidBrush(to_colorref(self.theme.panel)),
            hover: CreateSolidBrush(to_colorref(self.theme.hover)),
            border: CreateSolidBrush(to_colorref(self.theme.border)),
            accent: CreateSolidBrush(to_colorref(self.theme.accent)),
        };
        if !self.find_edit.0.is_null() {
            for control in [self.find_edit, self.replace_edit] {
                SendMessageW(
                    control,
                    WM_SETFONT,
                    Some(WPARAM(self.fonts.ui.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
        }
    }

    unsafe fn apply_theme_to_controls(&self) {
        let dark = if self.theme.dark { 1i32 } else { 0i32 };
        let _ = DwmSetWindowAttribute(
            self.hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const _ as *const c_void,
            std::mem::size_of::<i32>() as u32,
        );
        let sub_app = if self.theme.dark {
            w!("DarkMode_Explorer")
        } else {
            w!("Explorer")
        };
        if !self.edit.0.is_null() {
            SendMessageW(
                self.edit,
                EM_SETBKGNDCOLOR,
                Some(WPARAM(0)),
                Some(LPARAM(to_colorref(self.theme.bg).0 as isize)),
            );
            let _ = SetWindowTheme(self.edit, sub_app, PCWSTR::null());
            SendMessageW(self.edit, WM_THEMECHANGED, None, None);
            InvalidateRect(Some(self.edit), None, true);
            UpdateWindow(self.edit);
        }
        for control in [self.find_edit, self.replace_edit] {
            if !control.0.is_null() {
                let _ = SetWindowTheme(control, sub_app, PCWSTR::null());
                SendMessageW(control, WM_THEMECHANGED, None, None);
            }
        }
        let _ = SetWindowTheme(self.hwnd, sub_app, PCWSTR::null());
        SendMessageW(self.hwnd, WM_THEMECHANGED, None, None);
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn apply_topmost(&self) {
        let after = if self.settings.topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        let _ = SetWindowPos(
            self.hwnd,
            Some(after),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

impl App {
    unsafe fn layout(&mut self) {
        let mut client = RECT::default();
        let _ = GetClientRect(self.hwnd, &mut client);
        let top = self.scale(38);
        let status = self.scale(24);
        let find_height = if self.find_visible { self.scale(76) } else { 0 };
        let gutter = if self.doc.is_rendered_preview() {
            0
        } else {
            self.scale(58)
        };
        let edit_top = top + find_height;
        let edit_width = (client.right - gutter).max(1);
        let edit_height = (client.bottom - edit_top - status).max(1);
        let _ = MoveWindow(self.edit, gutter, edit_top, edit_width, edit_height, true);
        let margin_left = self.scale(12);
        let margin_right = self.scale(12);
        SendMessageW(
            self.edit,
            EM_SETMARGINS,
            Some(WPARAM(0x0003)),
            Some(LPARAM((margin_left | (margin_right << 16)) as isize)),
        );

        self.build_toolbar(client, top);
        self.layout_find_bar(client, top, find_height);
        self.apply_content_layout();
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn build_toolbar(&mut self, client: RECT, top: i32) {
        let mut specs: Vec<(u16, String, bool, bool)> = Vec::new();
        specs.push((CMD_OPEN, "打开".to_string(), true, false));
        specs.push((CMD_SAVE, "保存".to_string(), true, false));
        if self.doc.is_html() {
            specs.push((CMD_BROWSER, "浏览器预览".to_string(), true, false));
            specs.push((CMD_SOURCE, "源码".to_string(), true, true));
        } else if self.doc.is_markdown() {
            let label = if self.doc.show_source {
                "预览"
            } else {
                "源码"
            };
            specs.push((CMD_TOGGLE_VIEW, label.to_string(), true, false));
            if !self.doc.show_source {
                specs.push((CMD_TOC, "目录".to_string(), true, false));
            }
        }
        specs.push((CMD_FIND, "查找".to_string(), true, self.find_visible));
        if !self.doc.is_rendered_preview() {
            let label = if self.auto_wrap_long_lines && !self.settings.wrap_code {
                "自动换行"
            } else {
                "换行"
            };
            specs.push((
                CMD_WRAP,
                label.to_string(),
                true,
                self.settings.wrap_code || self.auto_wrap_long_lines,
            ));
        }
        specs.push((CMD_TOPMOST, "置顶".to_string(), true, self.settings.topmost));
        specs.push((CMD_THEME, "主题".to_string(), true, false));

        let gap = self.scale(6);
        let height = (top - self.scale(12)).max(self.scale(22));
        let widths: Vec<i32> = specs
            .iter()
            .map(|(_, label, _, _)| {
                self.scale((label.chars().count() as i32 * 14 + 24).clamp(46, 106))
            })
            .collect();
        let total = widths.iter().sum::<i32>() + gap * (specs.len() as i32 - 1);
        let mut x = (client.right - self.scale(12) - total).max(self.scale(140));
        let y = self.scale(6);
        self.buttons.clear();
        for ((id, label, enabled, active), width) in specs.into_iter().zip(widths) {
            self.buttons.push(HitButton {
                id,
                rect: RECT {
                    left: x,
                    top: y,
                    right: x + width,
                    bottom: y + height,
                },
                label,
                enabled,
                active,
            });
            x += width + gap;
        }
    }

    unsafe fn layout_find_bar(&mut self, client: RECT, top: i32, find_height: i32) {
        if !self.find_visible || find_height == 0 {
            let _ = ShowWindow(self.find_edit, SW_HIDE);
            let _ = ShowWindow(self.replace_edit, SW_HIDE);
            self.find_layout = None;
            return;
        }
        let gap = self.scale(6);
        let edge = self.scale(12);
        let row_height = self.scale(26);
        let first_y = top + self.scale(8);
        let second_y = first_y + row_height + self.scale(6);
        let width = |label: &str| -> i32 {
            self.scale((label.chars().count() as i32 * 14 + 24).clamp(42, 82))
        };
        let close_w = self.scale(32);
        let whole_w = width("全字");
        let case_w = width("Aa");
        let next_w = width("下一个");
        let prev_w = width("上一个");
        let reserved = close_w + whole_w + case_w + next_w + prev_w + gap * 5;
        let find_width = (client.right - edge * 2 - reserved).max(self.scale(120));
        let find_rect = RECT {
            left: edge,
            top: first_y,
            right: edge + find_width,
            bottom: first_y + row_height,
        };
        let mut x = find_rect.right + gap;
        let mut buttons = Vec::new();
        let push = |buttons: &mut Vec<HitButton>,
                    x: &mut i32,
                    id: u16,
                    label: &str,
                    button_width: i32,
                    y: i32,
                    active: bool,
                    enabled: bool| {
            buttons.push(HitButton {
                id,
                rect: RECT {
                    left: *x,
                    top: y,
                    right: *x + button_width,
                    bottom: y + row_height,
                },
                label: label.to_string(),
                enabled,
                active,
            });
            *x += button_width + gap;
        };
        push(
            &mut buttons,
            &mut x,
            CMD_FIND_PREV,
            "上一个",
            prev_w,
            first_y,
            false,
            true,
        );
        push(
            &mut buttons,
            &mut x,
            CMD_FIND_NEXT,
            "下一个",
            next_w,
            first_y,
            false,
            true,
        );
        push(
            &mut buttons,
            &mut x,
            CMD_MATCH_CASE,
            "Aa",
            case_w,
            first_y,
            self.match_case,
            true,
        );
        push(
            &mut buttons,
            &mut x,
            CMD_WHOLE_WORD,
            "全字",
            whole_w,
            first_y,
            self.whole_word,
            true,
        );
        push(
            &mut buttons,
            &mut x,
            CMD_FIND_CLOSE,
            "×",
            close_w,
            first_y,
            false,
            true,
        );

        let replace_w = width("替换");
        let all_w = width("全部替换");
        let replace_reserved = replace_w + all_w + gap * 2;
        let replace_width = (client.right - edge * 2 - replace_reserved).max(self.scale(120));
        let replace_rect = RECT {
            left: edge,
            top: second_y,
            right: edge + replace_width,
            bottom: second_y + row_height,
        };
        x = replace_rect.right + gap;
        push(
            &mut buttons,
            &mut x,
            CMD_REPLACE,
            "替换",
            replace_w,
            second_y,
            false,
            self.is_editable(),
        );
        push(
            &mut buttons,
            &mut x,
            CMD_REPLACE_ALL,
            "全部替换",
            all_w,
            second_y,
            false,
            self.is_editable(),
        );

        let _ = MoveWindow(
            self.find_edit,
            find_rect.left,
            find_rect.top,
            find_rect.right - find_rect.left,
            find_rect.bottom - find_rect.top,
            true,
        );
        let _ = MoveWindow(
            self.replace_edit,
            replace_rect.left,
            replace_rect.top,
            replace_rect.right - replace_rect.left,
            replace_rect.bottom - replace_rect.top,
            true,
        );
        let _ = ShowWindow(self.find_edit, SW_SHOW);
        let _ = ShowWindow(self.replace_edit, SW_SHOW);
        let _ = EnableWindow(self.replace_edit, self.is_editable());
        self.find_layout = Some(FindLayout {
            find_edit: find_rect,
            replace_edit: replace_rect,
            buttons,
        });
    }

    fn content_width_px(&self) -> i32 {
        let mut client = RECT::default();
        unsafe {
            let _ = GetClientRect(self.hwnd, &mut client);
        }
        let gutter = if self.doc.is_rendered_preview() {
            0
        } else {
            self.scale(58)
        };
        (client.right - gutter - self.scale(26)).max(self.scale(160))
    }

    fn content_width_twips(&self) -> i32 {
        self.content_width_px() * 1440 / self.dpi.max(1) as i32
    }

    unsafe fn apply_content_layout(&mut self) {
        let wrap = if self.doc.is_rendered_preview() {
            true
        } else {
            self.settings.wrap_code || self.auto_wrap_long_lines
        };
        let width = if wrap {
            self.content_width_twips()
        } else {
            1_000_000
        };
        let hdc = GetDC(Some(self.edit));
        SendMessageW(
            self.edit,
            EM_SETTARGETDEVICE,
            Some(WPARAM(hdc.0 as usize)),
            Some(LPARAM(width as isize)),
        );
        let _ = ReleaseDC(Some(self.edit), hdc);
    }

    unsafe fn rebuild_from_source(&mut self) {
        let preview = self.doc.is_rendered_preview();
        let source = self.doc.source.clone();
        self.longest_line_bytes = longest_line_bytes(&source);
        self.auto_wrap_long_lines = !preview
            && !self.settings.wrap_code
            && self.longest_line_bytes > AUTO_WRAP_LONG_LINE_LIMIT;
        self.source_line_starts = if preview {
            Vec::new()
        } else {
            source_line_starts(&source)
        };
        let width = self.content_width_twips();
        let (rtf, links, code_blocks, has_tables, headings) = if preview {
            let rendered = markdown::render(&source, self.theme, self.base_pt(), width);
            (
                rendered.rtf,
                rendered.links,
                rendered.code_blocks,
                rendered.has_tables,
                rendered.headings,
            )
        } else {
            (
                syntax::render_code_editable(
                    &source,
                    self.doc.current_language(),
                    self.theme,
                    self.base_pt(),
                ),
                Vec::new(),
                Vec::new(),
                false,
                Vec::new(),
            )
        };
        self.doc.has_tables = has_tables;
        self.rendered_links = links;
        self.rendered_code_blocks = code_blocks;
        self.rendered_headings = headings;
        self.link_ranges.clear();
        self.heading_ranges.clear();
        self.load_rtf(&rtf, preview);
        if preview {
            self.register_preview_links();
        }
        self.last_preview_width = width;
        self.refresh_status();
        self.invalidate_gutter();
        self.apply_content_layout();
    }

    unsafe fn load_rtf(&mut self, rtf: &str, readonly: bool) {
        let mut state = RtfStream {
            bytes: rtf.as_bytes(),
            position: 0,
        };
        let mut stream = EDITSTREAM {
            dwCookie: &mut state as *mut RtfStream<'_> as usize,
            dwError: 0,
            pfnCallback: Some(edit_stream_callback),
        };
        self.suppress_change = true;
        SendMessageW(self.edit, EM_SETREADONLY, Some(WPARAM(1)), Some(LPARAM(0)));
        SendMessageW(
            self.edit,
            EM_STREAMIN,
            Some(WPARAM(SF_RTF as usize)),
            Some(LPARAM(&mut stream as *mut EDITSTREAM as isize)),
        );
        self.suppress_change = false;
        SendMessageW(
            self.edit,
            EM_SETREADONLY,
            Some(WPARAM(readonly as usize)),
            Some(LPARAM(0)),
        );
        SendMessageW(
            self.edit,
            EM_SETEVENTMASK,
            Some(WPARAM(0)),
            Some(LPARAM(
                (ENM_CHANGE | ENM_SELCHANGE | ENM_SCROLL | ENM_LINK) as isize,
            )),
        );
        SendMessageW(self.edit, EM_SETMODIFY, Some(WPARAM(0)), Some(LPARAM(0)));
        SendMessageW(
            self.edit,
            EM_EXSETSEL,
            Some(WPARAM(0)),
            Some(LPARAM(
                &CHARRANGE { cpMin: 0, cpMax: 0 } as *const _ as isize
            )),
        );
        SendMessageW(self.edit, EM_SCROLLCARET, None, None);
        if stream.dwError != 0 {
            self.status_message = format!("排版失败（RichEdit 错误 {}）", stream.dwError);
        }
    }

    unsafe fn register_preview_links(&mut self) {
        if self.rendered_links.is_empty() {
            return;
        }
        let links = self.rendered_links.clone();
        let mut cursor = 0i32;
        for (label, target) in links {
            let Some(range) = self.find_exact_text(&label, cursor) else {
                continue;
            };
            let mut format = CHARFORMAT2W::default();
            format.Base.cbSize = std::mem::size_of::<CHARFORMAT2W>() as u32;
            format.Base.dwMask = CFM_LINK;
            format.Base.dwEffects = CFE_LINK;
            SendMessageW(
                self.edit,
                EM_EXSETSEL,
                Some(WPARAM(0)),
                Some(LPARAM(&range as *const CHARRANGE as isize)),
            );
            SendMessageW(
                self.edit,
                EM_SETCHARFORMAT,
                Some(WPARAM(SCF_SELECTION as usize)),
                Some(LPARAM(&format as *const CHARFORMAT2W as isize)),
            );
            self.link_ranges.push((range, target));
            cursor = range.cpMax;
        }
        SendMessageW(
            self.edit,
            EM_EXSETSEL,
            Some(WPARAM(0)),
            Some(LPARAM(
                &CHARRANGE { cpMin: 0, cpMax: 0 } as *const CHARRANGE as isize,
            )),
        );
        SendMessageW(self.edit, EM_SCROLLCARET, None, None);
    }

    unsafe fn register_heading_ranges(&mut self) {
        self.heading_ranges.clear();
        if !self.doc.is_rendered_preview() || self.rendered_headings.is_empty() {
            return;
        }
        let count = self.rendered_headings.len();
        let anchor = markdown::HEADING_ANCHOR.to_string();
        let mut cursor = 0i32;
        for index in 0..count {
            let Some(range) = self.find_exact_text(&anchor, cursor) else {
                continue;
            };
            let start = range.cpMin + 1;
            self.heading_ranges.push((
                CHARRANGE {
                    cpMin: start,
                    cpMax: start,
                },
                index,
            ));
            cursor = range.cpMax;
        }
    }

    unsafe fn show_toc_menu(&mut self) {
        if !self.doc.is_rendered_preview() {
            self.status_message = "目录仅在 Markdown 预览中可用。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        if self.rendered_headings.is_empty() {
            self.status_message = "本文档没有可跳转的标题。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        if self.heading_ranges.is_empty() {
            self.register_heading_ranges();
        }
        if self.heading_ranges.is_empty() {
            self.status_message = "未能在预览中定位标题。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let max_items =
            (u16::MAX as usize - CMD_TOC_BASE as usize + 1).min(self.heading_ranges.len());
        for (index, (_, heading_index)) in self.heading_ranges.iter().take(max_items).enumerate() {
            let heading = &self.rendered_headings[*heading_index];
            let indent = "  ".repeat(heading.level.saturating_sub(1).min(5));
            let label = format!("{indent}{}", menu_text(&heading.text));
            append_menu_item(menu, CMD_TOC_BASE + index as u16, &label, true);
        }
        let mut points = [self
            .button_rect(CMD_TOC)
            .map(|rect| POINT {
                x: rect.left,
                y: rect.bottom,
            })
            .unwrap_or(POINT {
                x: self.scale(120),
                y: self.scale(38),
            })];
        MapWindowPoints(Some(self.hwnd), None, &mut points);
        let _ = SetForegroundWindow(self.hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_TOPALIGN,
            points[0].x,
            points[0].y,
            None,
            self.hwnd,
            None,
        )
        .0 as u16;
        let _ = DestroyMenu(menu);
        let _ = PostMessageW(Some(self.hwnd), 0, WPARAM(0), LPARAM(0));
        if command >= CMD_TOC_BASE {
            self.on_command(WPARAM(command as usize), LPARAM(0));
        }
    }

    unsafe fn jump_to_heading(&mut self, range: CHARRANGE, title: &str) {
        let target = CHARRANGE {
            cpMin: range.cpMin,
            cpMax: range.cpMin,
        };
        SendMessageW(
            self.edit,
            EM_EXSETSEL,
            Some(WPARAM(0)),
            Some(LPARAM(&target as *const CHARRANGE as isize)),
        );
        SendMessageW(self.edit, EM_SCROLLCARET, None, None);
        let target_line = SendMessageW(
            self.edit,
            EM_LINEFROMCHAR,
            Some(WPARAM(range.cpMin.max(0) as usize)),
            Some(LPARAM(0)),
        )
        .0 as i32;
        let first_line = SendMessageW(self.edit, EM_GETFIRSTVISIBLELINE, None, None).0 as i32;
        let delta = target_line - first_line - 1;
        if delta != 0 {
            SendMessageW(
                self.edit,
                EM_LINESCROLL,
                Some(WPARAM(0)),
                Some(LPARAM(delta as isize)),
            );
        }
        let _ = SetFocus(Some(self.edit));
        self.status_message = format!("已跳转到「{title}」");
        self.refresh_status();
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn link_target_at_point(&self, point: POINT) -> Option<String> {
        if !self.doc.is_rendered_preview() || self.link_ranges.is_empty() {
            return None;
        }
        let length = self.edit_text_length();
        if length <= 0 {
            return None;
        }
        let mut point = point;
        let character = SendMessageW(
            self.edit,
            EM_CHARFROMPOS,
            None,
            Some(LPARAM(&mut point as *mut POINT as isize)),
        )
        .0;
        if character < 0 {
            return None;
        }
        let character = (character as i32).clamp(0, length - 1);
        self.link_ranges
            .iter()
            .find(|(range, _)| character >= range.cpMin && character < range.cpMax)
            .map(|(_, target)| target.clone())
    }

    unsafe fn try_activate_link_at(&mut self, point: POINT) -> bool {
        let Some(target) = self.link_target_at_point(point) else {
            return false;
        };
        if let Some(index) = target.strip_prefix("native-reader://copy-code/") {
            match index
                .parse::<usize>()
                .ok()
                .and_then(|index| self.rendered_code_blocks.get(index).cloned())
            {
                Some(code) => match copy_text_to_clipboard(&code) {
                    Ok(()) => self.status_message = "代码块已复制到剪贴板。".to_string(),
                    Err(error) => self.status_message = error,
                },
                None => self.status_message = "找不到对应的代码块。".to_string(),
            }
        } else {
            self.shell_open_url(&target);
        }
        InvalidateRect(Some(self.hwnd), None, false);
        true
    }

    fn is_editable(&self) -> bool {
        !self.doc.is_rendered_preview()
    }

    unsafe fn current_text(&self) -> String {
        if self.doc.is_rendered_preview() {
            self.doc.source.clone()
        } else {
            normalize_newlines(&get_window_text(self.edit))
        }
    }

    unsafe fn sync_source_from_edit(&mut self) {
        if self.suppress_change || self.doc.is_rendered_preview() {
            return;
        }
        let text = normalize_newlines(&get_window_text(self.edit));
        if text != self.doc.source {
            self.doc.source = text;
            self.doc.dirty = true;
            self.longest_line_bytes = longest_line_bytes(&self.doc.source);
            let auto_wrap_long_lines =
                !self.settings.wrap_code && self.longest_line_bytes > AUTO_WRAP_LONG_LINE_LIMIT;
            let layout_changed = auto_wrap_long_lines != self.auto_wrap_long_lines;
            self.auto_wrap_long_lines = auto_wrap_long_lines;
            self.source_line_starts = source_line_starts(&self.doc.source);
            if layout_changed {
                self.apply_content_layout();
            }
            self.refresh_title();
            self.refresh_status();
            InvalidateRect(Some(self.hwnd), None, false);
        }
    }

    unsafe fn refresh_title(&self) {
        set_window_title(self.hwnd, &self.doc.display_title());
    }

    unsafe fn refresh_status(&mut self) {
        let mut range = CHARRANGE::default();
        SendMessageW(
            self.edit,
            EM_EXGETSEL,
            Some(WPARAM(0)),
            Some(LPARAM(&mut range as *mut CHARRANGE as isize)),
        );
        let position = range.cpMin.max(0) as i32;
        if !self.doc.is_rendered_preview() && !self.source_line_starts.is_empty() {
            let line = self
                .source_line_starts
                .partition_point(|start| *start <= position)
                .saturating_sub(1);
            let line_start = self.source_line_starts[line];
            self.line = line as i32 + 1;
            self.column = (position - line_start + 1).max(1);
            self.current_line = line as i32;
            self.total_lines = self.source_line_starts.len() as i32;
        } else {
            let line = SendMessageW(
                self.edit,
                EM_LINEFROMCHAR,
                Some(WPARAM(position as usize)),
                Some(LPARAM(0)),
            )
            .0 as i32;
            let line_start = SendMessageW(
                self.edit,
                EM_LINEINDEX,
                Some(WPARAM(line.max(0) as usize)),
                Some(LPARAM(0)),
            )
            .0 as i32;
            self.line = line.max(0) + 1;
            self.column = (position - line_start.max(0) + 1).max(1);
            self.current_line = line.max(0);
            self.total_lines = SendMessageW(self.edit, EM_GETLINECOUNT, None, None).0 as i32;
        }
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn invalidate_gutter(&self) {
        if self.doc.is_rendered_preview() {
            return;
        }
        let mut client = RECT::default();
        let _ = GetClientRect(self.hwnd, &mut client);
        let rect = RECT {
            left: 0,
            top: self.scale(38),
            right: self.scale(58),
            bottom: client.bottom,
        };
        InvalidateRect(Some(self.hwnd), Some(&rect), false);
    }

    unsafe fn open_path_without_prompt(&mut self, path: &Path) -> bool {
        match Document::open(path) {
            Ok(document) => {
                self.last_stamp = document.stamp();
                self.doc = document;
                self.rebuild_from_source();
                self.refresh_title();
                self.refresh_status();
                self.status_message = if self.auto_wrap_long_lines {
                    format!(
                        "已打开 {} · 检测到超长行，已自动换行以保证流畅",
                        self.doc.title
                    )
                } else {
                    format!("已打开 {}", self.doc.title)
                };
                true
            }
            Err(error) => {
                self.show_error(&error);
                self.status_message = error;
                false
            }
        }
    }

    fn current_file_matches(&self, path: &Path) -> bool {
        self.doc
            .path
            .as_deref()
            .is_some_and(|current| same_path(current, path))
    }

    fn is_pristine_welcome(&self) -> bool {
        self.doc.path.is_none() && !self.doc.dirty && self.doc.title == "Native Reader"
    }

    unsafe fn request_open_path(&mut self, path: &Path) {
        let path = canonical_path(path);
        if self.current_file_matches(&path) {
            bring_window_to_front(self.hwnd);
            self.status_message = "该文件已在此窗口中打开。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        if activate_existing_window(&path, self.hwnd) {
            self.status_message = "该文件已在另一个窗口中打开，已切换过去。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        if self.is_pristine_welcome() {
            self.open_path_with_prompt(&path);
            return;
        }
        match create_app_window(self.instance, &self.settings, Some(path.clone())) {
            Ok(_) => {
                self.status_message = format!(
                    "已在新窗口中打开 {}",
                    path.file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("文件")
                );
                InvalidateRect(Some(self.hwnd), None, false);
            }
            Err(error) => {
                self.show_error(&error);
                self.status_message = error;
            }
        }
    }

    unsafe fn open_path_with_prompt(&mut self, path: &Path) -> bool {
        if !self.confirm_unsaved() {
            return false;
        }
        self.open_path_without_prompt(path)
    }

    unsafe fn open_file_dialog(&mut self) -> Option<PathBuf> {
        let mut file = vec![0u16; 32768];
        if let Some(path) = &self.doc.path {
            let text: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().collect();
            let count = text.len().min(file.len() - 1);
            file[..count].copy_from_slice(&text[..count]);
        }
        let filter: Vec<u16> = concat!(
            "所有支持的文本文件\0*.md;*.markdown;*.html;*.htm;*.xhtml;*.txt;*.rs;*.c;*.cpp;*.h;*.hpp;*.cs;*.java;*.js;*.ts;*.tsx;*.jsx;*.py;*.go;*.json;*.toml;*.yaml;*.yml;*.sql;*.sh;*.ps1;*.css;*.ini;*.lua;*.bat;*.cmd\0",
            "Markdown (*.md)\0*.md;*.markdown\0",
            "HTML (*.html)\0*.html;*.htm;*.xhtml\0",
            "代码与文本 (*.txt;*.rs;*.py;*.js;*.json)\0*.txt;*.rs;*.c;*.cpp;*.h;*.hpp;*.cs;*.java;*.js;*.ts;*.tsx;*.py;*.go;*.json;*.toml;*.yaml;*.yml;*.sql;*.sh;*.ps1;*.css;*.ini;*.lua;*.bat\0",
            "所有文件 (*.*)\0*.*\0\0"
        )
        .encode_utf16()
        .collect();
        let mut dialog = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: self.hwnd,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            nFilterIndex: 1,
            lpstrFile: PWSTR(file.as_mut_ptr()),
            nMaxFile: file.len() as u32,
            lpstrTitle: w!("打开文件"),
            Flags: OFN_EXPLORER
                | OFN_FILEMUSTEXIST
                | OFN_PATHMUSTEXIST
                | OFN_NOCHANGEDIR
                | OFN_HIDEREADONLY,
            ..Default::default()
        };
        if !GetOpenFileNameW(&mut dialog).as_bool() {
            return None;
        }
        Some(PathBuf::from(wide_buffer_to_string(&file)))
    }

    unsafe fn save_as_dialog(&mut self, default_extension: &str) -> Option<PathBuf> {
        let mut file = vec![0u16; 32768];
        let mut suggestion = self.doc.title.clone();
        if !suggestion.contains('.') {
            suggestion.push('.');
            suggestion.push_str(default_extension);
        }
        let suggestion: Vec<u16> = suggestion.encode_utf16().collect();
        let count = suggestion.len().min(file.len() - 1);
        file[..count].copy_from_slice(&suggestion[..count]);
        let filter: Vec<u16> = concat!(
            "Markdown (*.md)\0*.md;*.markdown\0",
            "HTML (*.html)\0*.html;*.htm;*.xhtml\0",
            "纯文本与代码 (*.txt;*.rs;*.py;*.js;*.json)\0*.txt;*.rs;*.c;*.cpp;*.h;*.hpp;*.cs;*.java;*.js;*.ts;*.tsx;*.py;*.go;*.json;*.toml;*.yaml;*.yml;*.sql;*.sh;*.ps1;*.css;*.ini;*.lua;*.bat\0",
            "所有文件 (*.*)\0*.*\0\0"
        )
        .encode_utf16()
        .collect();
        let extension = wide(default_extension);
        let mut dialog = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: self.hwnd,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            nFilterIndex: 1,
            lpstrFile: PWSTR(file.as_mut_ptr()),
            nMaxFile: file.len() as u32,
            lpstrTitle: w!("另存为"),
            lpstrDefExt: PCWSTR(extension.as_ptr()),
            Flags: OFN_EXPLORER
                | OFN_PATHMUSTEXIST
                | OFN_NOCHANGEDIR
                | OFN_HIDEREADONLY
                | OFN_OVERWRITEPROMPT,
            ..Default::default()
        };
        if !GetSaveFileNameW(&mut dialog).as_bool() {
            return None;
        }
        Some(PathBuf::from(wide_buffer_to_string(&file)))
    }

    unsafe fn save_document_with_extension(&mut self, extension: &str) -> bool {
        let text = self.current_text();
        let path = match self.doc.path.clone() {
            Some(path) => path,
            None => match self.save_as_dialog(extension) {
                Some(path) => path,
                None => return false,
            },
        };
        match self.doc.save_as(&path, text) {
            Ok(()) => {
                if self.doc.is_html() {
                    self.doc.show_source = true;
                }
                self.last_stamp = self.doc.stamp();
                self.refresh_title();
                self.refresh_status();
                self.status_message = format!("已保存 {}", self.doc.title);
                true
            }
            Err(error) => {
                self.show_error(&error);
                self.status_message = error;
                false
            }
        }
    }

    unsafe fn save_document(&mut self) -> bool {
        let extension = if self.doc.is_html() {
            "html"
        } else if self.doc.is_markdown() {
            "md"
        } else {
            "txt"
        };
        self.save_document_with_extension(extension)
    }

    unsafe fn open_html_in_browser(&mut self) {
        if !self.doc.is_html() {
            self.status_message = "当前文件不是 HTML。".to_string();
            return;
        }
        if (self.doc.dirty || self.doc.path.is_none()) && !self.save_document_with_extension("html")
        {
            return;
        }
        let Some(path) = self.doc.path.clone() else {
            return;
        };
        let absolute = std::fs::canonicalize(&path).unwrap_or(path);
        if self.shell_open_path(&absolute) {
            self.status_message =
                "已在默认浏览器打开；CSS、JS 和相对资源由浏览器加载。".to_string();
        }
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn shell_open_path(&self, path: &Path) -> bool {
        let value = wide(&path.to_string_lossy());
        let result = ShellExecuteW(
            Some(self.hwnd),
            w!("open"),
            PCWSTR(value.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
        if result.0 as isize <= 32 {
            self.show_error(&format!("无法调用默认程序打开：{}", path.display()));
            false
        } else {
            true
        }
    }

    unsafe fn shell_open_url(&self, url: &str) -> bool {
        let lower = url.trim().to_ascii_lowercase();
        if lower.starts_with("javascript:") || lower.starts_with("data:") {
            self.show_error("出于安全原因，不打开脚本或 data: 链接。");
            return false;
        }
        let value = wide(url);
        let result = ShellExecuteW(
            Some(self.hwnd),
            w!("open"),
            PCWSTR(value.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
        result.0 as isize > 32
    }

    unsafe fn confirm_unsaved(&mut self) -> bool {
        if !self.doc.dirty {
            return true;
        }
        let message = wide(&format!("“{}”有未保存的修改。是否保存？", self.doc.title));
        let result = MessageBoxW(
            Some(self.hwnd),
            PCWSTR(message.as_ptr()),
            w!("Native Reader"),
            MB_YESNOCANCEL | MB_ICONINFORMATION | MB_SETFOREGROUND,
        );
        if result == IDYES {
            self.save_document()
        } else {
            result == IDNO
        }
    }

    unsafe fn save_geometry(&mut self) {
        let mut rect = RECT::default();
        if GetWindowRect(self.hwnd, &mut rect).is_ok() {
            self.settings.x = Some(rect.left);
            self.settings.y = Some(rect.top);
            self.settings.width = (rect.right - rect.left).max(self.scale(480));
            self.settings.height = (rect.bottom - rect.top).max(self.scale(360));
        }
    }

    unsafe fn paste_plain_text(&mut self) {
        if !self.is_editable() {
            self.status_message = "只读预览中不能粘贴；按 Ctrl+2 切到源码。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        match read_clipboard_text() {
            Ok(text) => {
                let text = normalize_newlines(&text);
                let value = wide(&text);
                SendMessageW(
                    self.edit,
                    EM_REPLACESEL,
                    Some(WPARAM(1)),
                    Some(LPARAM(value.as_ptr() as isize)),
                );
                self.sync_source_from_edit();
                self.status_message = "已粘贴纯文本。".to_string();
            }
            Err(error) => {
                self.show_error(&error);
                self.status_message = error;
            }
        }
    }

    unsafe fn show_edit_context_menu(&mut self) {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let editable = self.is_editable();
        append_menu_item(menu, CMD_EDIT_UNDO, "撤销\tCtrl+Z", editable);
        append_menu_separator(menu);
        append_menu_item(menu, CMD_EDIT_CUT, "剪切\tCtrl+X", editable);
        append_menu_item(menu, CMD_EDIT_COPY, "复制\tCtrl+C", true);
        append_menu_item(menu, CMD_EDIT_PASTE, "粘贴纯文本\tCtrl+V", editable);
        append_menu_separator(menu);
        append_menu_item(menu, CMD_SELECT_ALL, "全选\tCtrl+A", true);
        append_menu_item(menu, CMD_COPY_ALL, "复制全文", true);

        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(self.hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_TOPALIGN,
            point.x,
            point.y,
            None,
            self.hwnd,
            None,
        )
        .0 as u16;
        let _ = DestroyMenu(menu);
        let _ = PostMessageW(Some(self.hwnd), 0, WPARAM(0), LPARAM(0));
        let _ = SetFocus(Some(self.edit));
        if command != 0 {
            self.on_command(WPARAM(command as usize), LPARAM(0));
        }
    }

    unsafe fn show_find_bar(&mut self) {
        if !self.find_visible {
            self.find_visible = true;
            self.layout();
        }
        let _ = SetFocus(Some(self.find_edit));
        SendMessageW(self.find_edit, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
        self.status_message = "查找：Enter 下一个，Shift+Enter 上一个，Esc 关闭。".to_string();
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn hide_find_bar(&mut self) {
        if self.find_visible {
            self.find_visible = false;
            self.search_summary.clear();
            let _ = SetFocus(Some(self.edit));
            self.layout();
        }
    }

    unsafe fn find_next(&mut self, forward: bool) {
        if !self.find_visible {
            self.show_find_bar();
        }
        let needle = get_window_text(self.find_edit);
        if needle.is_empty() {
            self.search_summary = "请输入查找内容".to_string();
            self.refresh_status();
            return;
        }
        let selection = self.selection();
        let start = if forward {
            selection.cpMax
        } else {
            selection.cpMin
        };
        let mut wrapped = false;
        let mut found = self.find_match(&needle, start, forward);
        if found.is_none() {
            wrapped = true;
            let wrap_start = if forward { 0 } else { self.edit_text_length() };
            found = self.find_match(&needle, wrap_start, forward);
        }
        match found {
            Some(range) => {
                SendMessageW(
                    self.edit,
                    EM_EXSETSEL,
                    Some(WPARAM(0)),
                    Some(LPARAM(&range as *const CHARRANGE as isize)),
                );
                SendMessageW(self.edit, EM_SCROLLCARET, None, None);
                self.search_summary = if wrapped {
                    "已回绕到另一端".to_string()
                } else {
                    String::new()
                };
                self.status_message = if wrapped {
                    "已找到（搜索已回绕）".to_string()
                } else {
                    "已找到".to_string()
                };
                self.refresh_status();
                self.invalidate_gutter();
            }
            None => {
                self.search_summary = "无匹配".to_string();
                self.status_message = "未找到匹配内容。".to_string();
                self.refresh_status();
            }
        }
    }

    unsafe fn find_exact_text(&self, needle: &str, start: i32) -> Option<CHARRANGE> {
        if needle.is_empty() {
            return None;
        }
        let length = self.edit_text_length();
        let start = start.clamp(0, length);
        if start >= length {
            return None;
        }
        let value = wide(needle);
        let mut find = FINDTEXTEXW {
            chrg: CHARRANGE {
                cpMin: start,
                cpMax: length,
            },
            lpstrText: PCWSTR(value.as_ptr()),
            chrgText: CHARRANGE::default(),
        };
        let result = SendMessageW(
            self.edit,
            EM_FINDTEXTEXW,
            Some(WPARAM((FR_DOWN | FR_MATCHCASE) as usize)),
            Some(LPARAM(&mut find as *mut FINDTEXTEXW as isize)),
        );
        if result.0 < 0 {
            None
        } else {
            Some(find.chrgText)
        }
    }

    unsafe fn find_match(&self, needle: &str, start: i32, forward: bool) -> Option<CHARRANGE> {
        let length = self.edit_text_length();
        let start = start.clamp(0, length);
        let range = if forward {
            CHARRANGE {
                cpMin: start,
                cpMax: length,
            }
        } else {
            CHARRANGE {
                cpMin: 0,
                cpMax: start,
            }
        };
        if range.cpMin >= range.cpMax {
            return None;
        }
        let value = wide(needle);
        let mut flags = 0usize;
        if forward {
            flags |= FR_DOWN;
        }
        if self.match_case {
            flags |= FR_MATCHCASE;
        }
        if self.whole_word {
            flags |= FR_WHOLEWORD;
        }
        let mut find = FINDTEXTEXW {
            chrg: range,
            lpstrText: PCWSTR(value.as_ptr()),
            chrgText: CHARRANGE::default(),
        };
        let result = SendMessageW(
            self.edit,
            EM_FINDTEXTEXW,
            Some(WPARAM(flags)),
            Some(LPARAM(&mut find as *mut FINDTEXTEXW as isize)),
        );
        if result.0 < 0 {
            None
        } else {
            Some(find.chrgText)
        }
    }

    fn edit_text_length(&self) -> i32 {
        unsafe { GetWindowTextLengthW(self.edit).max(0) }
    }

    unsafe fn selection(&self) -> CHARRANGE {
        let mut range = CHARRANGE::default();
        SendMessageW(
            self.edit,
            EM_EXGETSEL,
            Some(WPARAM(0)),
            Some(LPARAM(&mut range as *mut CHARRANGE as isize)),
        );
        range
    }

    unsafe fn range_text(&self, range: CHARRANGE) -> String {
        let length = (range.cpMax - range.cpMin).max(0) as usize;
        if length == 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; length + 1];
        let mut text_range = TEXTRANGEW {
            chrg: range,
            lpstrText: PWSTR(buffer.as_mut_ptr()),
        };
        SendMessageW(
            self.edit,
            EM_GETTEXTRANGE,
            Some(WPARAM(0)),
            Some(LPARAM(&mut text_range as *mut TEXTRANGEW as isize)),
        );
        let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(length);
        String::from_utf16_lossy(&buffer[..end])
    }

    unsafe fn replace_current(&mut self) {
        if !self.is_editable() {
            self.status_message = "Markdown 预览是只读的；按 Ctrl+2 切到源码再替换。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        let needle = get_window_text(self.find_edit);
        let replacement = get_window_text(self.replace_edit);
        if needle.is_empty() {
            self.search_summary = "请输入查找内容".to_string();
            self.refresh_status();
            return;
        }
        let mut selected = self.selection();
        let mut selected_text = self.range_text(selected);
        let matches = if self.match_case {
            selected_text == needle
        } else {
            selected_text.to_lowercase() == needle.to_lowercase()
        };
        if !matches {
            self.find_next(true);
            selected = self.selection();
            selected_text = self.range_text(selected);
            let matches = if self.match_case {
                selected_text == needle
            } else {
                selected_text.to_lowercase() == needle.to_lowercase()
            };
            if !matches {
                self.search_summary = "无匹配".to_string();
                self.refresh_status();
                return;
            }
        }
        let value = wide(&replacement);
        self.suppress_change = true;
        SendMessageW(
            self.edit,
            EM_REPLACESEL,
            Some(WPARAM(1)),
            Some(LPARAM(value.as_ptr() as isize)),
        );
        self.suppress_change = false;
        self.sync_source_from_edit();
        self.search_summary = "已替换".to_string();
        self.status_message = "已替换当前匹配。".to_string();
        self.refresh_status();
    }

    unsafe fn replace_all(&mut self) {
        if !self.is_editable() {
            self.status_message = "Markdown 预览是只读的；按 Ctrl+2 切到源码再替换。".to_string();
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        let needle = get_window_text(self.find_edit);
        if needle.is_empty() {
            self.search_summary = "请输入查找内容".to_string();
            self.refresh_status();
            return;
        }
        let replacement = get_window_text(self.replace_edit);
        let replacement_len = replacement.encode_utf16().count() as i32;
        let mut start = 0i32;
        let mut count = 0usize;
        self.suppress_change = true;
        loop {
            let Some(range) = self.find_match(&needle, start, true) else {
                break;
            };
            SendMessageW(
                self.edit,
                EM_EXSETSEL,
                Some(WPARAM(0)),
                Some(LPARAM(&range as *const CHARRANGE as isize)),
            );
            let value = wide(&replacement);
            SendMessageW(
                self.edit,
                EM_REPLACESEL,
                Some(WPARAM(1)),
                Some(LPARAM(value.as_ptr() as isize)),
            );
            count += 1;
            start = range.cpMin + replacement_len;
            if count > 1_000_000 {
                break;
            }
        }
        self.suppress_change = false;
        if count > 0 {
            self.sync_source_from_edit();
            self.search_summary = format!("已替换 {count} 处");
            self.status_message = self.search_summary.clone();
        } else {
            self.search_summary = "无匹配".to_string();
            self.status_message = "未找到匹配内容。".to_string();
        }
        self.refresh_status();
    }

    unsafe fn toggle_view(&mut self) {
        if self.doc.is_html() {
            self.open_html_in_browser();
        } else if self.doc.is_markdown() {
            self.doc.show_source = !self.doc.show_source;
            self.rebuild_from_source();
            self.layout();
            self.refresh_title();
            self.refresh_status();
            self.status_message = if self.doc.show_source {
                "Markdown 源码编辑".to_string()
            } else {
                "Markdown 只读预览".to_string()
            };
        } else {
            self.doc.show_source = true;
            self.rebuild_from_source();
            self.layout();
        }
    }

    unsafe fn show_source(&mut self) {
        if self.doc.is_markdown() && !self.doc.show_source {
            self.doc.show_source = true;
            self.rebuild_from_source();
            self.layout();
            self.refresh_title();
            self.refresh_status();
            self.status_message = "源码编辑".to_string();
        }
        if !self.doc.is_rendered_preview() {
            let _ = SetFocus(Some(self.edit));
        } else {
            self.status_message = "Markdown 预览为只读；按 Ctrl+2 查看源码。".to_string();
        }
    }

    unsafe fn on_command(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let command = wparam.0 as u16;
        if command >= CMD_TOC_BASE {
            if self.heading_ranges.is_empty() {
                self.register_heading_ranges();
            }
            let index = (command - CMD_TOC_BASE) as usize;
            if let Some((range, heading_index)) = self.heading_ranges.get(index).copied() {
                let title = self
                    .rendered_headings
                    .get(heading_index)
                    .map(|heading| heading.text.clone())
                    .unwrap_or_default();
                self.jump_to_heading(range, &title);
            }
            InvalidateRect(Some(self.hwnd), None, false);
            return;
        }
        match command {
            CMD_OPEN => {
                if let Some(path) = self.open_file_dialog() {
                    self.request_open_path(&path);
                }
            }
            CMD_SAVE => {
                self.save_document();
            }
            CMD_SAVE_AS => {
                let extension = if self.doc.is_html() {
                    "html"
                } else if self.doc.is_markdown() {
                    "md"
                } else {
                    "txt"
                };
                if let Some(path) = self.save_as_dialog(extension) {
                    let text = self.current_text();
                    match self.doc.save_as(&path, text) {
                        Ok(()) => {
                            if self.doc.is_html() {
                                self.doc.show_source = true;
                            }
                            self.last_stamp = self.doc.stamp();
                            self.refresh_title();
                            self.refresh_status();
                            self.status_message = format!("已保存 {}", self.doc.title);
                        }
                        Err(error) => {
                            self.show_error(&error);
                            self.status_message = error;
                        }
                    }
                }
            }
            CMD_CLIPBOARD => match read_clipboard_text() {
                Ok(text) => {
                    if self.confirm_unsaved() {
                        self.doc = Document::from_clipboard(text);
                        self.last_stamp = None;
                        self.rebuild_from_source();
                        self.layout();
                        self.refresh_title();
                        self.refresh_status();
                        self.status_message = "已从剪贴板载入（尚未保存到文件）".to_string();
                    }
                }
                Err(error) => {
                    self.show_error(&error);
                    self.status_message = error;
                }
            },
            CMD_COPY_ALL => {
                let text = self.current_text();
                match copy_text_to_clipboard(&text) {
                    Ok(()) => self.status_message = "已复制完整原文到剪贴板。".to_string(),
                    Err(error) => self.status_message = error,
                }
            }
            CMD_TOGGLE_VIEW => self.toggle_view(),
            CMD_SOURCE => self.show_source(),
            CMD_BROWSER => self.open_html_in_browser(),
            CMD_FIND => self.show_find_bar(),
            CMD_TOC => self.show_toc_menu(),
            CMD_FIND_NEXT => self.find_next(true),
            CMD_FIND_PREV => self.find_next(false),
            CMD_FIND_CLOSE => self.hide_find_bar(),
            CMD_REPLACE => self.replace_current(),
            CMD_REPLACE_ALL => self.replace_all(),
            CMD_MATCH_CASE => {
                self.match_case = !self.match_case;
                self.layout();
                self.status_message = if self.match_case {
                    "查找区分大小写".to_string()
                } else {
                    "查找不区分大小写".to_string()
                };
            }
            CMD_WHOLE_WORD => {
                self.whole_word = !self.whole_word;
                self.layout();
                self.status_message = if self.whole_word {
                    "全字匹配已开启".to_string()
                } else {
                    "全字匹配已关闭".to_string()
                };
            }
            CMD_EDIT_UNDO => {
                if self.is_editable() {
                    SendMessageW(self.edit, WM_UNDO, None, None);
                    self.sync_source_from_edit();
                    self.status_message = "已撤销上一步编辑。".to_string();
                }
            }
            CMD_EDIT_CUT => {
                if self.is_editable() {
                    SendMessageW(self.edit, WM_CUT, None, None);
                    self.sync_source_from_edit();
                    self.status_message = "已剪切所选文本。".to_string();
                }
            }
            CMD_EDIT_COPY => {
                let selection = self.selection();
                if selection.cpMax > selection.cpMin {
                    SendMessageW(self.edit, WM_COPY, None, None);
                    self.status_message = "已复制所选文本。".to_string();
                } else {
                    self.status_message = "请先选择要复制的文本。".to_string();
                }
            }
            CMD_EDIT_PASTE => self.paste_plain_text(),
            CMD_SELECT_ALL => {
                SendMessageW(self.edit, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
                let _ = SetFocus(Some(self.edit));
                self.refresh_status();
                self.status_message = "已全选。".to_string();
            }
            CMD_TOPMOST => {
                self.settings.topmost = !self.settings.topmost;
                self.apply_topmost();
                self.settings.save();
                self.layout();
                self.status_message = if self.settings.topmost {
                    "窗口已置顶".to_string()
                } else {
                    "窗口已取消置顶".to_string()
                };
            }
            CMD_WRAP => {
                if self.doc.is_rendered_preview() {
                    self.status_message = "Markdown 预览始终自动换行。".to_string();
                } else {
                    self.settings.wrap_code = !self.settings.wrap_code;
                    self.auto_wrap_long_lines = !self.settings.wrap_code
                        && self.longest_line_bytes > AUTO_WRAP_LONG_LINE_LIMIT;
                    self.apply_content_layout();
                    self.layout();
                    self.settings.save();
                    self.status_message = if self.settings.wrap_code {
                        "源码自动换行已开启".to_string()
                    } else if self.auto_wrap_long_lines {
                        "检测到超长行，已自动换行以保证流畅".to_string()
                    } else {
                        "源码自动换行已关闭".to_string()
                    };
                }
            }
            CMD_THEME => {
                self.settings.dark = !self.settings.dark;
                self.theme = theme::for_mode(self.settings.dark);
                self.rebuild_resources();
                self.apply_theme_to_controls();
                self.rebuild_from_source();
                self.layout();
                self.settings.save();
                self.status_message = if self.settings.dark {
                    "已切换到深色主题".to_string()
                } else {
                    "已切换到浅色主题".to_string()
                };
            }
            CMD_ZOOM_IN | CMD_ZOOM_OUT | CMD_ZOOM_RESET => {
                self.settings.zoom = match command {
                    CMD_ZOOM_IN => (self.settings.zoom + 10).min(200),
                    CMD_ZOOM_OUT => (self.settings.zoom - 10).max(70),
                    _ => 100,
                };
                self.rebuild_from_source();
                self.layout();
                self.settings.save();
                self.status_message = format!("缩放 {}%", self.settings.zoom);
            }
            CMD_RELOAD => {
                if self.confirm_unsaved() {
                    if let Some(path) = self.doc.path.clone() {
                        if let Ok(mut document) = Document::open(&path) {
                            document.show_source = self.doc.show_source;
                            self.doc = document;
                            self.last_stamp = self.doc.stamp();
                            self.rebuild_from_source();
                            self.layout();
                            self.refresh_title();
                            self.refresh_status();
                            self.status_message = "已重新载入文件".to_string();
                        }
                    }
                }
            }
            _ => {}
        }
        InvalidateRect(Some(self.hwnd), None, false);
    }

    unsafe fn on_notify(&mut self, lparam: LPARAM) -> LRESULT {
        let header = &*(lparam.0 as *const windows::Win32::UI::Controls::NMHDR);
        if header.hwndFrom != self.edit {
            return LRESULT(0);
        }
        match header.code {
            EN_CHANGE => {
                if !self.suppress_change {
                    self.sync_source_from_edit();
                }
                LRESULT(0)
            }
            EN_SELCHANGE => {
                self.refresh_status();
                self.invalidate_gutter();
                LRESULT(0)
            }
            EN_LINK => {
                let link = &*(lparam.0 as *const ENLINK);
                if link.msg == WM_SETCURSOR {
                    if let Ok(cursor) = LoadCursorW(None, IDC_HAND) {
                        SetCursor(Some(cursor));
                    }
                    return LRESULT(1);
                }
                LRESULT(1)
            }
            _ => LRESULT(0),
        }
    }

    unsafe fn on_ctlcolor(&self, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        let hdc = HDC(wparam.0 as *mut c_void);
        let _ = SetTextColor(hdc, to_colorref(self.theme.fg));
        let _ = SetBkColor(hdc, to_colorref(self.theme.panel));
        LRESULT(self.brushes.panel.0 as isize)
    }

    unsafe fn on_drop(&mut self, wparam: WPARAM) {
        let drop = HDROP(wparam.0 as *mut c_void);
        let count = DragQueryFileW(drop, 0xFFFF_FFFF, None);
        let mut paths = Vec::with_capacity(count as usize);
        for index in 0..count {
            let length = DragQueryFileW(drop, index, None) as usize;
            if length == 0 {
                continue;
            }
            let mut buffer = vec![0u16; length + 1];
            if DragQueryFileW(drop, index, Some(&mut buffer)) > 0 {
                paths.push(PathBuf::from(wide_buffer_to_string(&buffer)));
            }
        }
        DragFinish(drop);
        for path in paths {
            self.request_open_path(&path);
        }
    }

    unsafe fn on_timer(&mut self, timer: usize) {
        if timer == TIMER_AUTO_RELOAD {
            if self.doc.path.is_some() && !self.doc.dirty {
                let stamp = self.doc.stamp();
                if stamp.is_some() && stamp != self.last_stamp {
                    if let Some(path) = self.doc.path.clone() {
                        if let Ok(mut document) = Document::open(&path) {
                            document.show_source = self.doc.show_source;
                            self.doc = document;
                            self.last_stamp = self.doc.stamp();
                            self.rebuild_from_source();
                            self.layout();
                            self.refresh_title();
                            self.refresh_status();
                            self.status_message = "检测到外部修改，已自动重新载入".to_string();
                        }
                    }
                }
            }
        } else if timer == TIMER_RESIZE {
            KillTimer(Some(self.hwnd), TIMER_RESIZE).ok();
            if self.doc.is_rendered_preview() {
                let width = self.content_width_twips();
                if (width - self.last_preview_width).abs() > 40 {
                    self.rebuild_from_source();
                }
            }
        }
    }

    unsafe fn on_size(&mut self) {
        self.layout();
        if self.doc.is_rendered_preview() {
            SetTimer(Some(self.hwnd), TIMER_RESIZE, 180, None);
        }
    }

    unsafe fn on_mouse_move(&mut self, lparam: LPARAM) {
        let x = loword_signed(lparam.0);
        let y = hiword_signed(lparam.0);
        let hover = self.hit_button(x, y);
        if hover != self.hover_id {
            let previous = self.hover_id;
            self.hover_id = hover;
            self.invalidate_button(previous);
            self.invalidate_button(hover);
        }
        if !self.tracking_mouse {
            let mut track = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: self.hwnd,
                dwHoverTime: 0,
            };
            if TrackMouseEvent(&mut track).is_ok() {
                self.tracking_mouse = true;
            }
        }
    }

    fn button_rect(&self, id: u16) -> Option<RECT> {
        self.buttons
            .iter()
            .find(|button| button.id == id)
            .or_else(|| {
                self.find_layout
                    .as_ref()
                    .and_then(|layout| layout.buttons.iter().find(|button| button.id == id))
            })
            .map(|button| button.rect)
    }

    unsafe fn invalidate_button(&self, id: Option<u16>) {
        if let Some(rect) = id.and_then(|id| self.button_rect(id)) {
            let _ = InvalidateRect(Some(self.hwnd), Some(&rect), false);
        }
    }

    unsafe fn clear_hover(&mut self) {
        let previous = self.hover_id.take();
        self.tracking_mouse = false;
        self.invalidate_button(previous);
    }

    unsafe fn on_left_click(&mut self, lparam: LPARAM) {
        let x = loword_signed(lparam.0);
        let y = hiword_signed(lparam.0);
        if let Some(command) = self.hit_button(x, y) {
            self.on_command(WPARAM(command as usize), LPARAM(0));
        }
    }

    fn hit_button(&self, x: i32, y: i32) -> Option<u16> {
        for button in &self.buttons {
            if point_in_rect(x, y, button.rect) && button.enabled {
                return Some(button.id);
            }
        }
        if let Some(layout) = &self.find_layout {
            for button in &layout.buttons {
                if point_in_rect(x, y, button.rect) && button.enabled {
                    return Some(button.id);
                }
            }
        }
        None
    }

    fn cursor_is_hand(&self) -> bool {
        self.hover_id.is_some()
    }

    unsafe fn paint(&mut self) {
        let mut paint = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
        let hdc = BeginPaint(self.hwnd, &mut paint);
        let mut client = RECT::default();
        let _ = GetClientRect(self.hwnd, &mut client);
        FillRect(hdc, &client, self.brushes.bg);

        let top = self.scale(38);
        let status_height = self.scale(24);
        let top_rect = RECT {
            left: 0,
            top: 0,
            right: client.right,
            bottom: top,
        };
        FillRect(hdc, &top_rect, self.brushes.panel);
        let top_border = RECT {
            left: 0,
            top: top - self.scale(1),
            right: client.right,
            bottom: top,
        };
        FillRect(hdc, &top_border, self.brushes.border);

        let status_top = client.bottom - status_height;
        let status_rect = RECT {
            left: 0,
            top: status_top,
            right: client.right,
            bottom: client.bottom,
        };
        FillRect(hdc, &status_rect, self.brushes.panel);
        let status_border = RECT {
            left: 0,
            top: status_top,
            right: client.right,
            bottom: status_top + self.scale(1),
        };
        FillRect(hdc, &status_border, self.brushes.border);

        let _ = SetBkMode(hdc, TRANSPARENT);
        let gutter_rect = RECT {
            left: 0,
            top,
            right: self.scale(58),
            bottom: status_top,
        };
        let dirty = paint.rcPaint;
        let gutter_dirty = dirty.left < gutter_rect.right
            && dirty.right > gutter_rect.left
            && dirty.top < gutter_rect.bottom
            && dirty.bottom > gutter_rect.top;
        if !self.doc.is_rendered_preview() && gutter_dirty {
            self.paint_gutter(hdc, top, status_top);
        }
        self.paint_title(hdc, top);
        self.paint_buttons(hdc);
        self.paint_status(hdc, status_rect);
        let _ = EndPaint(self.hwnd, &paint);
    }

    unsafe fn paint_title(&self, hdc: HDC, top: i32) {
        let right = self
            .buttons
            .first()
            .map(|button| button.rect.left - self.scale(10))
            .unwrap_or_else(|| self.scale(600));
        let mut rect = RECT {
            left: self.scale(14),
            top: self.scale(9),
            right: right.max(self.scale(160)),
            bottom: top - self.scale(7),
        };
        let mut text: Vec<u16> = self.doc.display_title().encode_utf16().collect();
        let _ = SelectObject(hdc, HGDIOBJ(self.fonts.ui_bold.0));
        let _ = SetTextColor(hdc, to_colorref(self.theme.fg));
        DrawTextW(
            hdc,
            &mut text,
            &mut rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
    }

    unsafe fn paint_buttons(&self, hdc: HDC) {
        let _ = SelectObject(hdc, HGDIOBJ(self.fonts.ui.0));
        for button in &self.buttons {
            self.paint_button(hdc, button);
        }
        if let Some(layout) = &self.find_layout {
            for button in &layout.buttons {
                self.paint_button(hdc, button);
            }
        }
    }

    unsafe fn paint_button(&self, hdc: HDC, button: &HitButton) {
        let hover = self.hover_id.map(|id| id == button.id).unwrap_or(false);
        let brush = if button.active {
            self.brushes.accent
        } else if hover && button.enabled {
            self.brushes.hover
        } else {
            self.brushes.panel
        };
        FillRect(hdc, &button.rect, brush);
        FrameRect(hdc, &button.rect, self.brushes.border);
        let color = if !button.enabled {
            self.theme.muted
        } else if button.active {
            Color::rgb(255, 255, 255)
        } else {
            self.theme.fg
        };
        let _ = SetTextColor(hdc, to_colorref(color));
        let mut rect = button.rect;
        let mut text: Vec<u16> = button.label.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut text,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }

    unsafe fn paint_gutter(&self, hdc: HDC, top: i32, bottom: i32) {
        let gutter_right = self.scale(58);
        let rect = RECT {
            left: 0,
            top,
            right: gutter_right,
            bottom,
        };
        FillRect(hdc, &rect, self.brushes.panel);
        let border = RECT {
            left: gutter_right - self.scale(1),
            top,
            right: gutter_right,
            bottom,
        };
        FillRect(hdc, &border, self.brushes.border);

        let mut edit_rect = RECT::default();
        if GetWindowRect(self.edit, &mut edit_rect).is_err() {
            return;
        }
        let mut corners = [
            POINT {
                x: edit_rect.left,
                y: edit_rect.top,
            },
            POINT {
                x: edit_rect.right,
                y: edit_rect.bottom,
            },
        ];
        MapWindowPoints(None, Some(self.hwnd), &mut corners);
        let edit_top = corners[0].y.max(top);
        let edit_bottom = corners[1].y.min(bottom);
        if edit_top >= edit_bottom {
            return;
        }

        let saved = SaveDC(hdc);
        if saved == 0 {
            return;
        }
        let _ = IntersectClipRect(hdc, 0, edit_top, gutter_right, edit_bottom);

        let length = self.edit_text_length();
        if length <= 0 {
            let _ = RestoreDC(hdc, saved);
            return;
        }
        let line_height = self.line_height().max(self.scale(15));
        let mut anchor = POINT {
            x: self.scale(2),
            y: 0,
        };
        let anchor_char = SendMessageW(
            self.edit,
            EM_CHARFROMPOS,
            None,
            Some(LPARAM(&mut anchor as *mut POINT as isize)),
        )
        .0;
        let first_char = if anchor_char >= 0 && anchor_char < length as isize {
            anchor_char as i32
        } else {
            0
        };
        let first_line = self
            .source_line_starts
            .partition_point(|start| *start <= first_char)
            .saturating_sub(1);

        let _ = SelectObject(hdc, HGDIOBJ(self.fonts.small.0));
        let _ = SetBkMode(hdc, TRANSPARENT);
        let mut line = first_line.saturating_sub(1);
        let mut last_line_top = i32::MIN;
        while line < self.source_line_starts.len() {
            let start = self.source_line_starts[line];
            if start < 0 || start >= length {
                break;
            }
            let position = SendMessageW(
                self.edit,
                EM_POSFROMCHAR,
                Some(WPARAM(start as usize)),
                Some(LPARAM(0)),
            )
            .0;
            if position == -1 {
                break;
            }
            let line_top = corners[0].y + hiword_signed(position);
            if line_top > edit_bottom + line_height {
                break;
            }
            if line_top < edit_top {
                line += 1;
                continue;
            }
            if line_top <= last_line_top {
                line += 1;
                continue;
            }
            last_line_top = line_top;
            let mut number = RECT {
                left: self.scale(3),
                top: line_top,
                right: self.scale(50),
                bottom: line_top + line_height,
            };
            let color = if line as i32 == self.current_line {
                self.theme.accent
            } else {
                self.theme.muted
            };
            let _ = SetTextColor(hdc, to_colorref(color));
            let mut text: Vec<u16> = (line + 1).to_string().encode_utf16().collect();
            DrawTextW(
                hdc,
                &mut text,
                &mut number,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
            );
            line += 1;
        }
        let _ = RestoreDC(hdc, saved);
    }

    unsafe fn line_height(&self) -> i32 {
        let hdc = GetDC(Some(self.edit));
        if hdc.0.is_null() {
            return self.scale(18);
        }
        let _ = SelectObject(hdc, HGDIOBJ(self.fonts.code.0));
        let mut metrics = TEXTMETRICW::default();
        let height = if GetTextMetricsW(hdc, &mut metrics).as_bool() {
            metrics.tmHeight + metrics.tmExternalLeading
        } else {
            self.scale(18)
        };
        let _ = ReleaseDC(Some(self.edit), hdc);
        height.max(self.scale(15))
    }

    unsafe fn paint_status(&self, hdc: HDC, rect: RECT) {
        let mode = self.doc.mode_label();
        let dirty = if self.doc.dirty { " | 已修改" } else { "" };
        let auto_wrap = if self.auto_wrap_long_lines {
            " | 超长行自动换行"
        } else {
            ""
        };
        let left = format!(
            "Ln {}, Col {}   |   {}   |   {}   |   {}   |   {}{}{}",
            self.line,
            self.column,
            self.doc.kind_label(),
            self.doc.encoding,
            self.doc.size_label,
            mode,
            dirty,
            auto_wrap
        );
        let _ = SelectObject(hdc, HGDIOBJ(self.fonts.small.0));
        let _ = SetTextColor(hdc, to_colorref(self.theme.muted));
        let mut left_rect = RECT {
            left: self.scale(12),
            top: rect.top,
            right: rect.right - self.scale(240),
            bottom: rect.bottom,
        };
        let mut text: Vec<u16> = left.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut text,
            &mut left_rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        let right_text = if self.search_summary.is_empty() {
            &self.status_message
        } else {
            &self.search_summary
        };
        if !right_text.is_empty() {
            let _ = SetTextColor(hdc, to_colorref(self.theme.fg));
            let mut right_rect = RECT {
                left: rect.right - self.scale(360),
                top: rect.top,
                right: rect.right - self.scale(12),
                bottom: rect.bottom,
            };
            let mut text: Vec<u16> = right_text.encode_utf16().collect();
            DrawTextW(
                hdc,
                &mut text,
                &mut right_rect,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            );
        }
    }

    unsafe fn on_dpi_changed(&mut self, lparam: LPARAM) {
        let suggested = &*(lparam.0 as *const RECT);
        let _ = SetWindowPos(
            self.hwnd,
            None,
            suggested.left,
            suggested.top,
            suggested.right - suggested.left,
            suggested.bottom - suggested.top,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        self.dpi = {
            let dpi = GetDpiForWindow(self.hwnd);
            if dpi == 0 {
                96
            } else {
                dpi
            }
        };
        self.rebuild_resources();
        self.apply_theme_to_controls();
        self.rebuild_from_source();
        self.layout();
    }

    unsafe fn on_activate(&mut self, wparam: WPARAM) {
        if (wparam.0 & 0xffff) != 0 && self.settings.topmost {
            self.apply_topmost();
        }
    }

    unsafe fn show_error(&self, message: &str) {
        let message = wide(message);
        MessageBoxW(
            Some(self.hwnd),
            PCWSTR(message.as_ptr()),
            w!("Native Reader"),
            MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
        );
    }
}

struct RtfStream<'a> {
    bytes: &'a [u8],
    position: usize,
}

unsafe extern "system" fn edit_stream_callback(
    cookie: usize,
    buffer: *mut u8,
    count: i32,
    written: *mut i32,
) -> u32 {
    if count <= 0 || written.is_null() {
        if !written.is_null() {
            *written = 0;
        }
        return 0;
    }
    let state = &mut *(cookie as *mut RtfStream<'_>);
    let remaining = state.bytes.len().saturating_sub(state.position);
    let amount = remaining.min(count as usize);
    if amount > 0 {
        std::ptr::copy_nonoverlapping(state.bytes.as_ptr().add(state.position), buffer, amount);
        state.position += amount;
    }
    *written = amount as i32;
    0
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn source_line_starts(source: &str) -> Vec<i32> {
    let mut starts = Vec::new();
    let mut offset = 0i32;
    for raw_line in source.split('\n') {
        starts.push(offset);
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        offset = offset.saturating_add(line.encode_utf16().count() as i32 + 1);
    }
    starts
}

fn longest_line_bytes(source: &str) -> usize {
    source
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).len())
        .max()
        .unwrap_or(0)
}

fn wide_buffer_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

fn loword_signed(value: isize) -> i32 {
    (value as u32 & 0xffff) as i16 as i32
}

fn hiword_signed(value: isize) -> i32 {
    ((value as u32 >> 16) & 0xffff) as i16 as i32
}

fn menu_text(text: &str) -> String {
    text.replace('&', "&&")
        .replace('\t', " ")
        .replace('\r', " ")
        .replace('\n', " ")
}

unsafe fn append_menu_item(menu: HMENU, command: u16, label: &str, enabled: bool) {
    let mut flags = MF_STRING;
    if !enabled {
        flags |= MF_GRAYED;
    }
    let text = wide(label);
    let _ = AppendMenuW(menu, flags, command as usize, PCWSTR(text.as_ptr()));
}

unsafe fn append_menu_separator(menu: HMENU) {
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
}

fn point_in_rect(x: i32, y: i32, rect: RECT) -> bool {
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
}

fn to_colorref(color: Color) -> COLORREF {
    COLORREF(color.r as u32 | ((color.g as u32) << 8) | ((color.b as u32) << 16))
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn set_cue_banner(hwnd: HWND, text: &str) {
    let value = wide(text);
    SendMessageW(
        hwnd,
        EM_SETCUEBANNER,
        Some(WPARAM(1)),
        Some(LPARAM(value.as_ptr() as isize)),
    );
}

unsafe fn get_window_text(hwnd: HWND) -> String {
    let length = GetWindowTextLengthW(hwnd).max(0) as usize;
    let mut buffer = vec![0u16; length + 1];
    let copied = GetWindowTextW(hwnd, &mut buffer).max(0) as usize;
    String::from_utf16_lossy(&buffer[..copied.min(buffer.len())])
}

unsafe fn set_window_title(hwnd: HWND, title: &str) {
    let value = wide(title);
    let _ = SetWindowTextW(hwnd, PCWSTR(value.as_ptr()));
}

fn canonical_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn same_path(left: &Path, right: &Path) -> bool {
    canonical_path(left).to_string_lossy().to_lowercase()
        == canonical_path(right).to_string_lossy().to_lowercase()
}

unsafe fn copy_data_path(data: &COPYDATASTRUCT) -> Option<PathBuf> {
    if data.lpData.is_null() || data.cbData < std::mem::size_of::<u16>() as u32 {
        return None;
    }
    let length = data.cbData as usize / std::mem::size_of::<u16>();
    let units = std::slice::from_raw_parts(data.lpData as *const u16, length);
    let end = units.iter().position(|unit| *unit == 0)?;
    if end == 0 {
        return None;
    }
    Some(PathBuf::from(String::from_utf16_lossy(&units[..end])))
}

unsafe fn open_window_count() -> usize {
    let mut count = 0usize;
    let mut after = None;
    loop {
        let Ok(hwnd) = FindWindowExW(None, after, CLASS_NAME, PCWSTR::null()) else {
            break;
        };
        count += 1;
        after = Some(hwnd);
    }
    count
}

unsafe fn activate_existing_window(path: &Path, sender: HWND) -> bool {
    let canonical = canonical_path(path);
    let path_wide: Vec<u16> = canonical
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut data = COPYDATASTRUCT {
        dwData: COPYDATA_ACTIVATE_PATH,
        cbData: (path_wide.len() * std::mem::size_of::<u16>()) as u32,
        lpData: path_wide.as_ptr() as *mut c_void,
    };
    let mut after = None;
    loop {
        let Ok(hwnd) = FindWindowExW(None, after, CLASS_NAME, PCWSTR::null()) else {
            break;
        };
        if hwnd != sender {
            let handled = SendMessageW(
                hwnd,
                WM_COPYDATA,
                Some(WPARAM(sender.0 as usize)),
                Some(LPARAM(&mut data as *mut COPYDATASTRUCT as isize)),
            );
            if handled.0 == 1 {
                return true;
            }
        }
        after = Some(hwnd);
    }
    false
}

unsafe fn bring_window_to_front(hwnd: HWND) {
    if IsIconic(hwnd).as_bool() {
        let _ = ShowWindow(hwnd, SW_RESTORE);
    }
    let _ = SetForegroundWindow(hwnd);
}

unsafe fn create_app_window(
    instance: HINSTANCE,
    settings: &Settings,
    initial: Option<PathBuf>,
) -> Result<HWND, String> {
    let cascade = (open_window_count() % 8) as i32 * 28;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: settings.width,
        bottom: settings.height,
    };
    let _ = AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, false);
    let app = Box::new(App::new(settings.clone(), initial));
    let app_pointer = Box::into_raw(app);
    let hwnd = match CreateWindowExW(
        WINDOW_EX_STYLE(0x00000010),
        CLASS_NAME,
        w!("Native Reader"),
        WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
        settings.x.unwrap_or(80) + cascade,
        settings.y.unwrap_or(60) + cascade,
        rect.right - rect.left,
        rect.bottom - rect.top,
        None,
        None,
        Some(instance),
        Some(app_pointer as *const c_void),
    ) {
        Ok(hwnd) => hwnd,
        Err(error) => {
            drop(Box::from_raw(app_pointer));
            return Err(format!("无法创建主窗口：{error}"));
        }
    };
    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);
    if let Some(app) = app_from(hwnd) {
        let _ = SetFocus(Some(app.edit));
    }
    Ok(hwnd)
}

unsafe fn send_char_message(hwnd: HWND, message: u32, text: &str) -> LRESULT {
    let value = wide(text);
    SendMessageW(
        hwnd,
        message,
        Some(WPARAM(1)),
        Some(LPARAM(value.as_ptr() as isize)),
    )
}

unsafe fn load_icon_file(path: &Path) -> Option<HICON> {
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    LoadImageW(
        None,
        PCWSTR(path.as_ptr()),
        IMAGE_ICON,
        0,
        0,
        LR_LOADFROMFILE | LR_DEFAULTSIZE,
    )
    .ok()
    .map(|handle| HICON(handle.0))
}

pub fn run(initial: Option<PathBuf>) -> Result<(), String> {
    unsafe {
        let instance =
            GetModuleHandleW(None).map_err(|error| format!("无法获取程序实例：{error}"))?;
        let cursor =
            LoadCursorW(None, IDC_ARROW).map_err(|error| format!("无法加载光标：{error}"))?;
        let icon = crate::icon::install()
            .as_deref()
            .and_then(|path| load_icon_file(path))
            .or_else(|| LoadIconW(None, IDI_APPLICATION).ok())
            .ok_or_else(|| "无法加载程序图标。".to_string())?;
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: cursor,
            hIcon: icon,
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err("无法注册主窗口类。".to_string());
        }
        let settings = Settings::load();
        if let Some(path) = initial.as_deref() {
            let no_sender = HWND(std::ptr::null_mut());
            if activate_existing_window(path, no_sender) {
                return Ok(());
            }
        }
        let association_report = associations::ensure_registered_if_needed().ok();
        let hwnd = create_app_window(instance.into(), &settings, initial)?;
        if let Some(app) = app_from(hwnd) {
            if association_report
                .as_ref()
                .is_some_and(|report| report.needs_user_confirmation())
                && !app.settings.defaults_prompted
                && !associations::defaults_prompted()
            {
                app.settings.defaults_prompted = true;
                app.settings.save();
                associations::mark_defaults_prompted();
                app.status_message =
                    "部分默认关联受 Windows 保护，请在已打开的设置窗口中确认。".to_string();
                associations::open_default_apps();
            }
        }
        let accelerators = build_accelerators();
        let accel = CreateAcceleratorTableW(&accelerators).unwrap_or(HACCEL::default());

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let target = GetAncestor(message.hwnd, GA_ROOT);
            if !accel.0.is_null()
                && !target.0.is_null()
                && TranslateAcceleratorW(target, accel, &message) != 0
            {
                continue;
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        if !accel.0.is_null() {
            let _ = DestroyAcceleratorTable(accel);
        }
        Ok(())
    }
}

fn build_accelerators() -> Vec<ACCEL> {
    let ctrl = FVIRTKEY | FCONTROL;
    let ctrl_shift = FVIRTKEY | FCONTROL | FSHIFT;
    vec![
        ACCEL {
            fVirt: ctrl,
            key: VK_O.0,
            cmd: CMD_OPEN,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_S.0,
            cmd: CMD_SAVE,
        },
        ACCEL {
            fVirt: ctrl_shift,
            key: VK_S.0,
            cmd: CMD_SAVE_AS,
        },
        ACCEL {
            fVirt: ctrl_shift,
            key: VK_V.0,
            cmd: CMD_CLIPBOARD,
        },
        ACCEL {
            fVirt: ctrl_shift,
            key: VK_C.0,
            cmd: CMD_COPY_ALL,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_F.0,
            cmd: CMD_FIND,
        },
        ACCEL {
            fVirt: FVIRTKEY,
            key: VK_F3.0,
            cmd: CMD_FIND_NEXT,
        },
        ACCEL {
            fVirt: FVIRTKEY | FSHIFT,
            key: VK_F3.0,
            cmd: CMD_FIND_PREV,
        },
        ACCEL {
            fVirt: FVIRTKEY,
            key: VK_ESCAPE.0,
            cmd: CMD_FIND_CLOSE,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_1.0,
            cmd: CMD_TOGGLE_VIEW,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_2.0,
            cmd: CMD_SOURCE,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_T.0,
            cmd: CMD_TOPMOST,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_W.0,
            cmd: CMD_WRAP,
        },
        ACCEL {
            fVirt: ctrl_shift,
            key: VK_D.0,
            cmd: CMD_THEME,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_OEM_PLUS.0,
            cmd: CMD_ZOOM_IN,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_ADD.0,
            cmd: CMD_ZOOM_IN,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_OEM_MINUS.0,
            cmd: CMD_ZOOM_OUT,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_SUBTRACT.0,
            cmd: CMD_ZOOM_OUT,
        },
        ACCEL {
            fVirt: ctrl,
            key: VK_0.0,
            cmd: CMD_ZOOM_RESET,
        },
        ACCEL {
            fVirt: FVIRTKEY,
            key: VK_F5.0,
            cmd: CMD_RELOAD,
        },
    ]
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        let pointer = create.lpCreateParams as *mut App;
        if !pointer.is_null() {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, pointer as isize);
            (*pointer).hwnd = hwnd;
            (*pointer).instance = create.hInstance;
        }
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    if message == WM_NCDESTROY {
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
        if !pointer.is_null() {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(pointer));
        }
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let Some(app) = app_from(hwnd) else {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    };
    if message == WM_COPYDATA {
        if lparam.0 == 0 {
            return LRESULT(0);
        }
        let data = &*(lparam.0 as *const COPYDATASTRUCT);
        if data.dwData != COPYDATA_ACTIVATE_PATH {
            return LRESULT(0);
        }
        if let Some(path) = copy_data_path(data) {
            if app.current_file_matches(&path) {
                bring_window_to_front(hwnd);
                app.status_message = "该文件已在此窗口中打开。".to_string();
                InvalidateRect(Some(hwnd), None, false);
                return LRESULT(1);
            }
        }
        return LRESULT(0);
    }
    match message {
        WM_CREATE => match app.create_children() {
            Ok(()) => {
                WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);
                LRESULT(0)
            }
            Err(error) => {
                let text = wide(&error);
                MessageBoxW(
                    Some(hwnd),
                    PCWSTR(text.as_ptr()),
                    w!("Native Reader"),
                    MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
                );
                LRESULT(-1)
            }
        },
        WM_SIZE => {
            app.on_size();
            LRESULT(0)
        }
        WM_PAINT => {
            app.paint();
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_SETFOCUS => {
            if GetFocus() == app.find_edit || GetFocus() == app.replace_edit {
                LRESULT(0)
            } else {
                let _ = SetFocus(Some(app.edit));
                LRESULT(0)
            }
        }
        WM_COMMAND => {
            app.on_command(wparam, lparam);
            LRESULT(0)
        }
        WM_NOTIFY => app.on_notify(lparam),
        WM_CTLCOLOREDIT => app.on_ctlcolor(wparam, lparam),
        WM_DROPFILES => {
            app.on_drop(wparam);
            LRESULT(0)
        }
        WM_TIMER => {
            app.on_timer(wparam.0);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            app.on_mouse_move(lparam);
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            app.clear_hover();
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            app.on_left_click(lparam);
            LRESULT(0)
        }
        WM_SETCURSOR => {
            if app.cursor_is_hand() {
                let hand = LoadCursorW(None, IDC_HAND);
                if let Ok(cursor) = hand {
                    SetCursor(Some(cursor));
                }
                LRESULT(1)
            } else {
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lparam.0 as *mut MINMAXINFO);
            info.ptMinTrackSize.x = app.scale(760);
            info.ptMinTrackSize.y = app.scale(480);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            app.on_dpi_changed(lparam);
            LRESULT(0)
        }
        WM_ACTIVATE => {
            app.on_activate(wparam);
            LRESULT(0)
        }
        WM_CLOSE => {
            if app.confirm_unsaved() {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            app.save_geometry();
            app.settings.save();
            KillTimer(Some(hwnd), TIMER_AUTO_RELOAD).ok();
            KillTimer(Some(hwnd), TIMER_RESIZE).ok();
            let previous = WINDOW_COUNT
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
                    Some(count.saturating_sub(1))
                })
                .unwrap_or(0);
            if previous <= 1 {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn app_from(hwnd: HWND) -> Option<&'static mut App> {
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
    if pointer.is_null() {
        None
    } else {
        Some(&mut *pointer)
    }
}

unsafe extern "system" fn edit_subclass(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    let app = &mut *(data as *mut App);
    match message {
        WM_CONTEXTMENU => {
            app.show_edit_context_menu();
            LRESULT(0)
        }
        WM_PASTE => {
            app.paste_plain_text();
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            let ctrl = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
            let shift = (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
            if ctrl && !shift && key == VK_V.0 {
                app.paste_plain_text();
                return LRESULT(0);
            }
            DefSubclassProc(hwnd, message, wparam, lparam)
        }
        WM_LBUTTONUP => {
            let point = POINT {
                x: loword_signed(lparam.0),
                y: hiword_signed(lparam.0),
            };
            if app.try_activate_link_at(point) {
                LRESULT(0)
            } else {
                DefSubclassProc(hwnd, message, wparam, lparam)
            }
        }
        WM_SETCURSOR => {
            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_ok() {
                let _ = ScreenToClient(hwnd, &mut point);
                if app.link_target_at_point(point).is_some() {
                    if let Ok(cursor) = LoadCursorW(None, IDC_HAND) {
                        SetCursor(Some(cursor));
                    }
                    return LRESULT(1);
                }
            }
            DefSubclassProc(hwnd, message, wparam, lparam)
        }
        WM_VSCROLL | WM_HSCROLL | WM_MOUSEWHEEL => {
            let result = DefSubclassProc(hwnd, message, wparam, lparam);
            app.invalidate_gutter();
            result
        }
        _ => DefSubclassProc(hwnd, message, wparam, lparam),
    }
}

unsafe extern "system" fn search_subclass(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    let app = &mut *(data as *mut App);
    if message == WM_KEYDOWN {
        let key = wparam.0 as u16;
        if key == VK_ESCAPE.0 {
            app.hide_find_bar();
            return LRESULT(0);
        }
        if key == VK_RETURN.0 {
            if hwnd == app.find_edit {
                app.find_next(true);
            } else {
                app.replace_current();
            }
            return LRESULT(0);
        }
        if key == VK_F3.0 {
            let shift = (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
            app.find_next(!shift);
            return LRESULT(0);
        }
    }
    DefSubclassProc(hwnd, message, wparam, lparam)
}
