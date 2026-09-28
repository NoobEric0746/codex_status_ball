#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

type Handle = *mut c_void;
type Hwnd = Handle;
type Hdc = Handle;
type Hbrush = Handle;
type Hfont = Handle;
type Hinstance = Handle;
type Hregion = Handle;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;

const CLASS_NAME: &[u16] = &[67, 111, 100, 101, 120, 83, 116, 97, 116, 117, 115, 66, 97, 108, 108, 0];
const SETTINGS_CLASS_NAME: &[u16] = &[67, 111, 100, 101, 120, 83, 116, 97, 116, 117, 115, 83, 101, 116, 116, 105, 110, 103, 115, 0];
const WINDOW_TITLE: &[u16] = &[67, 111, 100, 101, 120, 32, 83, 116, 97, 116, 117, 115, 32, 66, 97, 108, 108, 0];
const WM_PAINT: u32 = 0x000F;
const WM_COMMAND: u32 = 0x0111;
const WM_SETFONT: u32 = 0x0030;
const WM_TIMER: u32 = 0x0113;
const WM_ALLOWANCE_UPDATED: u32 = 0x8001;
const WM_DESTROY: u32 = 0x0002;
const WM_NCHITTEST: u32 = 0x0084;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_MOUSEHOVER: u32 = 0x02A1;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_CAPTURECHANGED: u32 = 0x0215;
const TTM_ADDTOOLW: u32 = 0x0432;
const TTM_UPDATETIPTEXTW: u32 = 0x0439;
const TTM_TRACKACTIVATE: u32 = 0x0411;
const TTM_TRACKPOSITION: u32 = 0x0412;
const TTM_SETMAXTIPWIDTH: u32 = 0x0418;
const TTM_SETDELAYTIME: u32 = 0x0403;
const TTF_IDISHWND: u32 = 0x0001;
const TTF_TRACK: u32 = 0x0020;
const TTF_ABSOLUTE: u32 = 0x0080;
const TTS_USEVISUALSTYLE: u32 = 0x0100;
const HTCLIENT: Lresult = 1;
const WS_POPUP: u32 = 0x80000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_BORDER: u32 = 0x00800000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_NOACTIVATE: u32 = 0x08000000;
const WS_EX_LAYERED: u32 = 0x00080000;
const SW_SHOWNOACTIVATE: i32 = 4;
const SW_HIDE: i32 = 0;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOACTIVATE: u32 = 0x0010;
const HWND_TOPMOST: Hwnd = -1isize as Hwnd;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const TRANSPARENT: i32 = 1;
const PS_SOLID: i32 = 0;
const AD_CLOCKWISE: i32 = 2;
const HALFTONE: i32 = 4;
const SCALE: i32 = 4;
const BALL_SIZE: i32 = 122;
const PANEL_WIDTH: i32 = 236;
const TASKBAR_WIDTH: i32 = 112;
const WINDOW_WIDTH: i32 = BALL_SIZE + PANEL_WIDTH;
const WINDOW_HEIGHT: i32 = BALL_SIZE;
const TASKBAR_TOPMOST_TIMER: usize = 1;
const ALLOWANCE_REFRESH_TIMER: usize = 2;
const ALLOWANCE_REFRESH_MS: u32 = 60_000;
const DEFAULT_PREFETCH_SECONDS: u32 = 5;
const SETTINGS_MENU_ID: usize = 2;
const APPLY_BUTTON_ID: usize = 3;
const MIN_REFRESH_SECONDS: u32 = 1;
const MAX_REFRESH_SECONDS: u32 = 86_400;
const TASKBAR_ANIMATION_MS: u32 = 50;

#[repr(C)]
#[derive(Clone, Copy)]
struct Point { x: i32, y: i32 }

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect { left: i32, top: i32, right: i32, bottom: i32 }
#[repr(C)]
struct Size { width: i32, height: i32 }
#[repr(C)]
struct BitmapInfoHeader { size: u32, width: i32, height: i32, planes: u16, bit_count: u16, compression: u32, image_size: u32, x_pixels_per_meter: i32, y_pixels_per_meter: i32, colors_used: u32, colors_important: u32 }
#[repr(C)]
struct BitmapInfo { header: BitmapInfoHeader, colors: [u32; 1] }
#[repr(C)]
struct BlendFunction { operation: u8, flags: u8, constant_alpha: u8, alpha_format: u8 }

#[repr(C)]
struct Msg { hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam, time: u32, point: Point }

#[repr(C)]
struct PaintStruct {
    hdc: Hdc,
    erase: i32,
    paint: Rect,
    restore: i32,
    inc_update: i32,
    reserved: [u8; 32],
}

#[repr(C)]
struct TrackMouseEvent { size: u32, flags: u32, hwnd: Hwnd, hover_time: u32 }

#[repr(C)]
struct ToolInfoW {
    size: u32,
    flags: u32,
    hwnd: Hwnd,
    id: usize,
    rect: Rect,
    instance: Hinstance,
    text: *mut u16,
    param: Lparam,
}

#[repr(C)]
struct InitCommonControlsEx { size: u32, classes: u32 }

#[link(name = "comctl32")]
unsafe extern "system" {
    fn InitCommonControlsEx(config: *const InitCommonControlsEx) -> i32;
}

type WindowProc = unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult;

static HOVERED: AtomicBool = AtomicBool::new(false);
static DOCKED: AtomicBool = AtomicBool::new(false);
static mut BALL_WINDOW: Hwnd = null_mut();
static mut TASKBAR_WINDOW: Hwnd = null_mut();
static mut TASKBAR_TOOLTIP: Hwnd = null_mut();
static mut TASKBAR_TOOLTIP_TEXT: [u16; 128] = [0; 128];
static mut TASKBAR_TOOLTIP_VISIBLE: bool = false;
static mut TASKBAR_HOVER_POINT: Option<Point> = None;
static mut SETTINGS_WINDOW: Hwnd = null_mut();
static mut REFRESH_EDIT: Hwnd = null_mut();
static mut PREFETCH_EDIT: Hwnd = null_mut();
static mut BALL_DRAG_OFFSET: Option<Point> = None;
static mut TASKBAR_DRAG_OFFSET: Option<Point> = None;
static SNAPSHOT: OnceLock<Mutex<AllowanceSnapshot>> = OnceLock::new();
static REFRESHING: AtomicBool = AtomicBool::new(false);
static REFRESH_CYCLE: OnceLock<Mutex<RefreshCycle>> = OnceLock::new();
static REFRESH_INTERVAL_MS: AtomicU32 = AtomicU32::new(ALLOWANCE_REFRESH_MS);
static PREFETCH_SECONDS: AtomicU32 = AtomicU32::new(DEFAULT_PREFETCH_SECONDS);

struct RefreshCycle {
    started: Instant,
    generation: u64,
    prefetch_started: bool,
    awaiting_result: bool,
    pending: Option<AllowanceSnapshot>,
}

impl RefreshCycle {
    fn new() -> Self {
        Self { started: Instant::now(), generation: 0, prefetch_started: false, awaiting_result: false, pending: None }
    }

    fn reset(&mut self) {
        self.started = Instant::now();
        self.generation = self.generation.wrapping_add(1);
        self.prefetch_started = false;
        self.awaiting_result = false;
        self.pending = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AllowanceSnapshot {
    five_hour: QuotaWindow,
    weekly: Option<QuotaWindow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuotaWindow {
    remaining_percent: u8,
    resets_at: Option<i64>,
    window_duration_mins: Option<i64>,
}

#[repr(C)]
struct WndClass {
    style: u32,
    window_proc: WindowProc,
    class_extra: i32,
    window_extra: i32,
    instance: Hinstance,
    icon: Handle,
    cursor: Handle,
    background: Hbrush,
    menu_name: *const u16,
    class_name: *const u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
    fn CreateWindowExW(ex_style: u32, class_name: *const u16, title: *const u16, style: u32, x: i32, y: i32, width: i32, height: i32, parent: Hwnd, menu: Handle, instance: Hinstance, params: Handle) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn DispatchMessageW(message: *const Msg) -> Lresult;
    fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> i32;
    fn GetMessageW(message: *mut Msg, hwnd: Hwnd, min: u32, max: u32) -> i32;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn GetDC(hwnd: Hwnd) -> Hdc;
    fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
    fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: i32) -> i32;
    fn FindWindowW(class_name: *const u16, window_name: *const u16) -> Hwnd;
    fn GetModuleHandleW(name: *const u16) -> Hinstance;
    fn GetSystemMetrics(index: i32) -> i32;
    fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: i32) -> i32;
    fn PostMessageW(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> i32;
    fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Handle;
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn GetSysColorBrush(index: i32) -> Hbrush;
    fn SendMessageW(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn ReleaseCapture() -> i32;
    fn SetCapture(hwnd: Hwnd) -> Hwnd;
    fn SetWindowPos(hwnd: Hwnd, insert_after: Hwnd, x: i32, y: i32, width: i32, height: i32, flags: u32) -> i32;
    fn SetTimer(hwnd: Hwnd, timer_id: usize, interval_ms: u32, callback: Handle) -> usize;
    fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, flags: u32) -> i32;
    fn UpdateLayeredWindow(hwnd: Hwnd, screen_dc: Hdc, position: *const Point, size: *const Size, source_dc: Hdc, source_point: *const Point, color_key: u32, blend: *const BlendFunction, flags: u32) -> i32;
    fn CreatePopupMenu() -> Handle;
    fn AppendMenuW(menu: Handle, flags: u32, item: usize, text: *const u16) -> i32;
    fn TrackPopupMenu(menu: Handle, flags: u32, x: i32, y: i32, reserved: i32, hwnd: Hwnd, rect: *const Rect) -> u32;
    fn DestroyMenu(menu: Handle) -> i32;
    fn DestroyWindow(hwnd: Hwnd) -> i32;
    fn SetForegroundWindow(hwnd: Hwnd) -> i32;
    fn SetWindowRgn(hwnd: Hwnd, region: Hregion, redraw: i32) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
    fn TrackMouseEvent(event: *mut TrackMouseEvent) -> i32;
    fn TranslateMessage(message: *const Msg) -> i32;
    fn PostQuitMessage(exit_code: i32);
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn Arc(hdc: Hdc, left: i32, top: i32, right: i32, bottom: i32, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> i32;
    fn CreateEllipticRgn(left: i32, top: i32, right: i32, bottom: i32) -> Hregion;
    fn CreateFontW(height: i32, width: i32, escapement: i32, orientation: i32, weight: i32, italic: u32, underline: u32, strikeout: u32, charset: u32, out_precision: u32, clip_precision: u32, quality: u32, pitch_family: u32, face: *const u16) -> Hfont;
    fn CreatePen(style: i32, width: i32, color: u32) -> Handle;
    fn CreateSolidBrush(color: u32) -> Hbrush;
    fn DeleteObject(object: Handle) -> i32;
    fn DrawTextW(hdc: Hdc, text: *const u16, length: i32, rect: *mut Rect, format: u32) -> i32;
    fn Ellipse(hdc: Hdc, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    fn FillRect(hdc: Hdc, rect: *const Rect, brush: Hbrush) -> i32;
    fn GetTextExtentPoint32W(hdc: Hdc, text: *const u16, length: i32, size: *mut Size) -> i32;
    fn CreateCompatibleBitmap(hdc: Hdc, width: i32, height: i32) -> Handle;
    fn CreateDIBSection(hdc: Hdc, info: *const BitmapInfo, usage: u32, bits: *mut *mut c_void, section: Handle, offset: u32) -> Handle;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn DeleteDC(hdc: Hdc) -> i32;
    fn CreateRoundRectRgn(left: i32, top: i32, right: i32, bottom: i32, width: i32, height: i32) -> Hregion;
    fn SetStretchBltMode(hdc: Hdc, mode: i32) -> i32;
    fn StretchBlt(destination: Hdc, x: i32, y: i32, width: i32, height: i32, source: Hdc, source_x: i32, source_y: i32, source_width: i32, source_height: i32, operation: u32) -> i32;
    fn SelectObject(hdc: Hdc, object: Handle) -> Handle;
    fn SetArcDirection(hdc: Hdc, direction: i32) -> i32;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: u32) -> u32;
    fn GetStockObject(index: i32) -> Handle;
}

unsafe extern "system" fn window_proc(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> Lresult {
    match message {
        WM_COMMAND if hwnd == SETTINGS_WINDOW && w_param & 0xffff == APPLY_BUTTON_ID => {
            let mut buffer = [0u16; 16];
            let length = GetWindowTextW(REFRESH_EDIT, buffer.as_mut_ptr(), buffer.len() as i32);
            let seconds = String::from_utf16_lossy(&buffer[..length.max(0) as usize]).parse::<u32>();
            let length = GetWindowTextW(PREFETCH_EDIT, buffer.as_mut_ptr(), buffer.len() as i32);
            let prefetch_seconds = String::from_utf16_lossy(&buffer[..length.max(0) as usize]).parse::<u32>();
            match (seconds, prefetch_seconds) {
                (Ok(seconds), Ok(prefetch_seconds)) if (MIN_REFRESH_SECONDS..=MAX_REFRESH_SECONDS).contains(&seconds) && prefetch_seconds <= seconds => {
                    REFRESH_INTERVAL_MS.store(seconds * 1000, Ordering::Relaxed);
                    PREFETCH_SECONDS.store(prefetch_seconds, Ordering::Relaxed);
                    REFRESH_CYCLE.get().expect("refresh cycle initialized").lock().unwrap().reset();
                    save_refresh_settings(seconds, prefetch_seconds);
                    InvalidateRect(TASKBAR_WINDOW, null(), 0);
                    DestroyWindow(hwnd);
                }
                _ => {
                    let text = wide("刷新间隔需为 1 到 86400 秒；提前获取需为 0 到刷新间隔之间的秒数。");
                    let title = wide("刷新设置");
                    MessageBoxW(hwnd, text.as_ptr(), title.as_ptr(), 0x30);
                }
            }
            0
        }
        WM_LBUTTONDOWN if hwnd == TASKBAR_WINDOW => {
            hide_taskbar_tooltip();
            let mut cursor = Point { x: 0, y: 0 };
            let mut window = Rect { left: 0, top: 0, right: 0, bottom: 0 };
            if GetCursorPos(&mut cursor) != 0 && GetWindowRect(hwnd, &mut window) != 0 {
                TASKBAR_DRAG_OFFSET = Some(Point { x: cursor.x - window.left, y: cursor.y - window.top });
                SetCapture(hwnd);
            }
            0
        }
        WM_MOUSEMOVE if hwnd == TASKBAR_WINDOW => {
            let mut cursor = Point { x: 0, y: 0 };
            if GetCursorPos(&mut cursor) != 0 {
                let previous = TASKBAR_HOVER_POINT;
                if previous.is_none_or(|point| point.x != cursor.x || point.y != cursor.y) {
                    TASKBAR_HOVER_POINT = Some(cursor);
                    hide_taskbar_tooltip();
                }
            }
            let drag_offset = TASKBAR_DRAG_OFFSET;
            if drag_offset.is_none() {
                let mut tracking = TrackMouseEvent { size: std::mem::size_of::<TrackMouseEvent>() as u32, flags: 1 | 2, hwnd, hover_time: 500 };
                TrackMouseEvent(&mut tracking);
            }
            if let Some(offset) = TASKBAR_DRAG_OFFSET {
                let mut cursor = Point { x: 0, y: 0 };
                let mut window = Rect { left: 0, top: 0, right: 0, bottom: 0 };
                if GetCursorPos(&mut cursor) != 0 && GetWindowRect(hwnd, &mut window) != 0 {
                    if let Some(taskbar) = taskbar_rect() {
                        if !point_in_rect(cursor, taskbar) {
                            undock_to_ball(cursor);
                        } else {
                            let width = window.right - window.left;
                            let height = window.bottom - window.top;
                            let horizontal = taskbar.right - taskbar.left > taskbar.bottom - taskbar.top;
                            let x = if horizontal { (cursor.x - offset.x).clamp(taskbar.left, (taskbar.right - width).max(taskbar.left)) } else { taskbar.left };
                            let y = if horizontal { taskbar.top } else { (cursor.y - offset.y).clamp(taskbar.top, (taskbar.bottom - height).max(taskbar.top)) };
                            SetWindowPos(hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
                        }
                    }
                }
            }
            0
        }
        WM_MOUSEHOVER if hwnd == TASKBAR_WINDOW => {
            let drag_offset = TASKBAR_DRAG_OFFSET;
            if drag_offset.is_none() && DOCKED.load(Ordering::Relaxed) {
                update_taskbar_tooltip();
                show_taskbar_tooltip();
            }
            0
        }
        WM_MOUSELEAVE if hwnd == TASKBAR_WINDOW => {
            let mut cursor = Point { x: 0, y: 0 };
            let mut rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
            if GetCursorPos(&mut cursor) == 0 || GetWindowRect(hwnd, &mut rect) == 0 || !point_in_rect(cursor, rect) {
                hide_taskbar_tooltip();
                TASKBAR_HOVER_POINT = None;
            }
            0
        }
        WM_LBUTTONUP if hwnd == TASKBAR_WINDOW => {
            TASKBAR_DRAG_OFFSET = None;
            ReleaseCapture();
            0
        }
        WM_CAPTURECHANGED if hwnd == TASKBAR_WINDOW => { TASKBAR_DRAG_OFFSET = None; 0 }
        WM_RBUTTONUP if hwnd == BALL_WINDOW || hwnd == TASKBAR_WINDOW => {
            let mut cursor = Point { x: 0, y: 0 };
            if GetCursorPos(&mut cursor) != 0 {
                let menu = CreatePopupMenu();
                let settings_label = wide("设置");
                AppendMenuW(menu, 0, SETTINGS_MENU_ID, settings_label.as_ptr());
                let exit_label: Vec<u16> = "Exit".encode_utf16().chain(Some(0)).collect();
                AppendMenuW(menu, 0, 1, exit_label.as_ptr());
                SetForegroundWindow(hwnd);
                let selected = TrackPopupMenu(menu, 0x0100 | 0x0002, cursor.x, cursor.y, 0, hwnd, null());
                DestroyMenu(menu);
                if selected == SETTINGS_MENU_ID as u32 { show_settings(); }
                if selected == 1 { DestroyWindow(BALL_WINDOW); }
            }
            0
        }
        WM_TIMER if hwnd == TASKBAR_WINDOW && w_param == TASKBAR_TOPMOST_TIMER => {
            if DOCKED.load(Ordering::Relaxed) {
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
            0
        }
        WM_TIMER if hwnd == BALL_WINDOW && w_param == ALLOWANCE_REFRESH_TIMER => {
            let cycle = REFRESH_CYCLE.get().expect("refresh cycle initialized");
            let mut cycle = cycle.lock().unwrap();
            let interval = Duration::from_millis(REFRESH_INTERVAL_MS.load(Ordering::Relaxed) as u64);
            let elapsed = cycle.started.elapsed();
            if !cycle.prefetch_started && elapsed >= interval.saturating_sub(Duration::from_secs(PREFETCH_SECONDS.load(Ordering::Relaxed) as u64)) {
                cycle.prefetch_started = start_allowance_prefetch(hwnd, cycle.generation);
            }
            let due = elapsed >= interval;
            let ready = if due {
                let ready = cycle.pending.take();
                let awaiting_result = cycle.prefetch_started && ready.is_none();
                cycle.reset();
                cycle.awaiting_result = awaiting_result;
                ready
            } else { None };
            drop(cycle);
            if let Some(snapshot) = ready { publish_allowances(hwnd, snapshot); }
            if DOCKED.load(Ordering::Relaxed) {
                InvalidateRect(TASKBAR_WINDOW, null(), 0);
            }
            if due {
                InvalidateRect(BALL_WINDOW, null(), 0);
            }
            0
        }
        WM_ALLOWANCE_UPDATED if hwnd == BALL_WINDOW => {
            InvalidateRect(BALL_WINDOW, null(), 0);
            if DOCKED.load(Ordering::Relaxed) {
                InvalidateRect(TASKBAR_WINDOW, null(), 0);
            }
            0
        }
        WM_NCHITTEST if hwnd == BALL_WINDOW || hwnd == TASKBAR_WINDOW => HTCLIENT,
        WM_MOUSEMOVE if hwnd == BALL_WINDOW => {
            if let Some(offset) = BALL_DRAG_OFFSET {
                let mut cursor = Point { x: 0, y: 0 };
                if GetCursorPos(&mut cursor) != 0 {
                    if taskbar_rect().is_some_and(|taskbar| point_in_rect(cursor, taskbar)) {
                        dock_to_taskbar(cursor);
                    } else {
                        SetWindowPos(hwnd, HWND_TOPMOST, cursor.x - offset.x, cursor.y - offset.y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
                    }
                }
                return 0;
            }
            if !HOVERED.swap(true, Ordering::Relaxed) {
                set_window_region(hwnd, true);
                InvalidateRect(hwnd, null(), 0);
            }
            let mut tracking = TrackMouseEvent { size: std::mem::size_of::<TrackMouseEvent>() as u32, flags: 2, hwnd, hover_time: 0 };
            TrackMouseEvent(&mut tracking);
            0
        }
        WM_MOUSELEAVE if hwnd == BALL_WINDOW => {
            HOVERED.store(false, Ordering::Relaxed);
            set_window_region(hwnd, false);
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_LBUTTONDOWN if hwnd == BALL_WINDOW => {
            let mut cursor = Point { x: 0, y: 0 };
            let mut window = Rect { left: 0, top: 0, right: 0, bottom: 0 };
            if GetCursorPos(&mut cursor) != 0 && GetWindowRect(hwnd, &mut window) != 0 {
                BALL_DRAG_OFFSET = Some(Point { x: cursor.x - window.left, y: cursor.y - window.top });
                SetCapture(hwnd);
            }
            0
        }
        WM_LBUTTONUP if hwnd == BALL_WINDOW => { BALL_DRAG_OFFSET = None; ReleaseCapture(); 0 }
        WM_CAPTURECHANGED if hwnd == BALL_WINDOW => { BALL_DRAG_OFFSET = None; 0 }
        WM_PAINT if hwnd == BALL_WINDOW || hwnd == TASKBAR_WINDOW => {
            if let Some(snapshot) = SNAPSHOT.get() {
                let snapshot = snapshot.lock().unwrap();
                if hwnd == TASKBAR_WINDOW { paint_taskbar(hwnd, &snapshot); }
                else { paint_window(hwnd, &snapshot); }
            }
            0
        }
        WM_DESTROY if hwnd == SETTINGS_WINDOW => {
            SETTINGS_WINDOW = null_mut();
            REFRESH_EDIT = null_mut();
            PREFETCH_EDIT = null_mut();
            0
        }
        WM_DESTROY if hwnd == BALL_WINDOW => {
            if !SETTINGS_WINDOW.is_null() { DestroyWindow(SETTINGS_WINDOW); }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, w_param, l_param),
    }
}

fn publish_allowances(hwnd: Hwnd, snapshot: AllowanceSnapshot) {
    if let Some(current) = SNAPSHOT.get() {
        *current.lock().unwrap() = snapshot;
    }
    unsafe { PostMessageW(hwnd, WM_ALLOWANCE_UPDATED, 0, 0); }
}

fn start_allowance_prefetch(hwnd: Hwnd, generation: u64) -> bool {
    if REFRESHING.swap(true, Ordering::AcqRel) { return false; }
    let hwnd = hwnd as usize;
    std::thread::spawn(move || {
        match fetch_allowances() {
            Ok(snapshot) => {
                let mut cycle = REFRESH_CYCLE.get().expect("refresh cycle initialized").lock().unwrap();
                if cycle.generation == generation {
                    cycle.pending = Some(snapshot);
                } else if cycle.generation == generation.wrapping_add(1) && cycle.awaiting_result {
                    cycle.awaiting_result = false;
                    drop(cycle);
                    publish_allowances(hwnd as Hwnd, snapshot);
                }
            }
            Err(error) => eprintln!("Could not refresh Codex allowance: {error}"),
        }
        REFRESHING.store(false, Ordering::Release);
    });
    true
}

unsafe fn dock_to_taskbar(cursor: Point) {
    let Some(taskbar) = taskbar_rect() else { return; };
    let width = taskbar.right - taskbar.left;
    let height = taskbar.bottom - taskbar.top;
    let horizontal = width > height;
    let bar_width = if horizontal { TASKBAR_WIDTH.min(width) } else { width };
    let bar_height = if horizontal { height } else { 40.min(height) };
    let x = if horizontal { (cursor.x - bar_width / 2).clamp(taskbar.left, taskbar.right - bar_width) } else { taskbar.left };
    let y = if horizontal { taskbar.top } else { (cursor.y - bar_height / 2).clamp(taskbar.top, taskbar.bottom - bar_height) };
    BALL_DRAG_OFFSET = None;
    ReleaseCapture();
    HOVERED.store(false, Ordering::Relaxed);
    set_window_region(BALL_WINDOW, false);
    ShowWindow(BALL_WINDOW, SW_HIDE);
    SetWindowPos(TASKBAR_WINDOW, HWND_TOPMOST, x, y, bar_width, bar_height, SWP_NOACTIVATE);
    DOCKED.store(true, Ordering::Relaxed);
    ShowWindow(TASKBAR_WINDOW, SW_SHOWNOACTIVATE);
    SetWindowPos(TASKBAR_WINDOW, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    if let Some(snapshot) = SNAPSHOT.get() { paint_taskbar(TASKBAR_WINDOW, &snapshot.lock().unwrap()); }
    TASKBAR_DRAG_OFFSET = Some(Point { x: cursor.x - x, y: cursor.y - y });
    SetCapture(TASKBAR_WINDOW);
}

unsafe fn undock_to_ball(cursor: Point) {
    hide_taskbar_tooltip();
    TASKBAR_DRAG_OFFSET = None;
    ReleaseCapture();
    DOCKED.store(false, Ordering::Relaxed);
    ShowWindow(TASKBAR_WINDOW, SW_HIDE);
    HOVERED.store(false, Ordering::Relaxed);
    set_window_region(BALL_WINDOW, false);
    let x = cursor.x - BALL_SIZE / 2;
    let y = cursor.y - BALL_SIZE / 2;
    SetWindowPos(BALL_WINDOW, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
    ShowWindow(BALL_WINDOW, SW_SHOWNOACTIVATE);
    InvalidateRect(BALL_WINDOW, null(), 0);
    BALL_DRAG_OFFSET = Some(Point { x: cursor.x - x, y: cursor.y - y });
    SetCapture(BALL_WINDOW);
}

unsafe fn taskbar_rect() -> Option<Rect> {
    let shell_class: Vec<u16> = "Shell_TrayWnd".encode_utf16().chain(Some(0)).collect();
    let shell = FindWindowW(shell_class.as_ptr(), null());
    let mut rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
    (!shell.is_null() && GetWindowRect(shell, &mut rect) != 0).then_some(rect)
}

unsafe fn update_taskbar_tooltip() {
    let drag_offset = TASKBAR_DRAG_OFFSET;
    if TASKBAR_TOOLTIP.is_null() || drag_offset.is_some() { return; }
    let Some(snapshot) = SNAPSHOT.get() else { return; };
    let snapshot = snapshot.lock().unwrap();
    let weekly = snapshot.weekly.as_ref().map_or_else(|| "--".to_string(), |window| format_weekly_reset_duration(window.resets_at));
    let label = format!("5h    RESET IN  {}\r\nWeek  RESET IN  {}", format_reset_duration(snapshot.five_hour.resets_at), weekly);
    let new_text = wide(&label);
    let buffer = std::ptr::addr_of_mut!(TASKBAR_TOOLTIP_TEXT).cast::<u16>();
    std::ptr::write_bytes(buffer, 0, 128);
    std::ptr::copy_nonoverlapping(new_text.as_ptr(), buffer, new_text.len().min(128));
    let mut tool = taskbar_tool_info();
    SendMessageW(TASKBAR_TOOLTIP, TTM_UPDATETIPTEXTW, 0, &mut tool as *mut _ as Lparam);
}

unsafe fn show_taskbar_tooltip() {
    if TASKBAR_TOOLTIP.is_null() { return; }
    let mut cursor = Point { x: 0, y: 0 };
    if GetCursorPos(&mut cursor) == 0 { return; }
    let Some(taskbar) = taskbar_rect() else { return; };
    let horizontal = taskbar.right - taskbar.left > taskbar.bottom - taskbar.top;
    let initial = if horizontal {
        Point { x: cursor.x + 12, y: if taskbar.top > GetSystemMetrics(SM_CYSCREEN) / 2 { taskbar.top - 60 } else { taskbar.bottom + 8 } }
    } else {
        Point { x: if taskbar.left > GetSystemMetrics(SM_CXSCREEN) / 2 { taskbar.left - 240 } else { taskbar.right + 8 }, y: cursor.y + 8 }
    };
    position_taskbar_tooltip(initial);
    let mut tool = taskbar_tool_info();
    SendMessageW(TASKBAR_TOOLTIP, TTM_TRACKACTIVATE, 1, &mut tool as *mut _ as Lparam);
    let mut tip = Rect { left: 0, top: 0, right: 0, bottom: 0 };
    if GetWindowRect(TASKBAR_TOOLTIP, &mut tip) != 0 {
        let width = tip.right - tip.left;
        let height = tip.bottom - tip.top;
        let final_position = if horizontal {
            Point { x: (cursor.x + 12).min(taskbar.right - width), y: if taskbar.top > GetSystemMetrics(SM_CYSCREEN) / 2 { taskbar.top - height - 8 } else { taskbar.bottom + 8 } }
        } else {
            Point { x: if taskbar.left > GetSystemMetrics(SM_CXSCREEN) / 2 { taskbar.left - width - 8 } else { taskbar.right + 8 }, y: cursor.y + 8 }
        };
        position_taskbar_tooltip(final_position);
    }
    TASKBAR_TOOLTIP_VISIBLE = true;
}

unsafe fn position_taskbar_tooltip(point: Point) {
    let position = ((point.y as u16 as usize) << 16) | (point.x as u16 as usize);
    SendMessageW(TASKBAR_TOOLTIP, TTM_TRACKPOSITION, 0, position as Lparam);
}

unsafe fn hide_taskbar_tooltip() {
    if !TASKBAR_TOOLTIP_VISIBLE || TASKBAR_TOOLTIP.is_null() { return; }
    let mut tool = taskbar_tool_info();
    SendMessageW(TASKBAR_TOOLTIP, TTM_TRACKACTIVATE, 0, &mut tool as *mut _ as Lparam);
    TASKBAR_TOOLTIP_VISIBLE = false;
}

unsafe fn taskbar_tool_info() -> ToolInfoW {
    ToolInfoW {
        size: std::mem::size_of::<ToolInfoW>() as u32,
        flags: TTF_IDISHWND | TTF_TRACK | TTF_ABSOLUTE,
        hwnd: TASKBAR_WINDOW,
        id: TASKBAR_WINDOW as usize,
        rect: Rect { left: 0, top: 0, right: 0, bottom: 0 },
        instance: null_mut(),
        text: std::ptr::addr_of_mut!(TASKBAR_TOOLTIP_TEXT).cast::<u16>(),
        param: 0,
    }
}

fn point_in_rect(point: Point, rect: Rect) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

unsafe fn paint_taskbar(hwnd: Hwnd, snapshot: &AllowanceSnapshot) {
    let mut paint = PaintStruct { hdc: null_mut(), erase: 0, paint: Rect { left: 0, top: 0, right: 0, bottom: 0 }, restore: 0, inc_update: 0, reserved: [0; 32] };
    BeginPaint(hwnd, &mut paint);
    let mut window = Rect { left: 0, top: 0, right: 0, bottom: 0 };
    if GetWindowRect(hwnd, &mut window) == 0 { EndPaint(hwnd, &paint); return; }
    let bounds = Rect { left: 0, top: 0, right: window.right - window.left, bottom: window.bottom - window.top };
    let screen_dc = GetDC(null_mut());
    let hdc = CreateCompatibleDC(screen_dc);
    let info = BitmapInfo { header: BitmapInfoHeader { size: 40, width: bounds.right, height: -bounds.bottom, planes: 1, bit_count: 32, compression: 0, image_size: 0, x_pixels_per_meter: 0, y_pixels_per_meter: 0, colors_used: 0, colors_important: 0 }, colors: [0] };
    let mut bits: *mut c_void = null_mut();
    let bitmap = CreateDIBSection(screen_dc, &info, 0, &mut bits, null_mut(), 0);
    if bitmap.is_null() {
        DeleteDC(hdc);
        ReleaseDC(null_mut(), screen_dc);
        EndPaint(hwnd, &paint);
        return;
    }
    let previous_bitmap = SelectObject(hdc, bitmap);
    let background = CreateSolidBrush(rgb(255, 255, 255));
    FillRect(hdc, &bounds, background);
    DeleteObject(background);
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, rgb(0, 0, 0));
    let top = format!("5h  {}%", snapshot.five_hour.remaining_percent);
    let bottom = snapshot.weekly.as_ref().map_or_else(|| "Week  --".to_string(), |weekly| format!("Week  {}%", weekly.remaining_percent));
    let top_wide: Vec<u16> = top.encode_utf16().collect();
    let bottom_wide: Vec<u16> = bottom.encode_utf16().collect();
    let midpoint = (bounds.bottom - bounds.top) / 2;
    let row_height = midpoint.min(bounds.bottom - midpoint);
    let mut selected_font = null_mut();
    let mut previous = null_mut();
    for font_size in (12..=row_height.min(24)).rev() {
        let font = CreateFontW(-font_size, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, [83, 101, 103, 111, 101, 32, 85, 73, 0].as_ptr());
        let old = SelectObject(hdc, font);
        let mut top_size = Size { width: 0, height: 0 };
        let mut bottom_size = Size { width: 0, height: 0 };
        GetTextExtentPoint32W(hdc, top_wide.as_ptr(), top_wide.len() as i32, &mut top_size);
        GetTextExtentPoint32W(hdc, bottom_wide.as_ptr(), bottom_wide.len() as i32, &mut bottom_size);
        if top_size.width <= bounds.right - 4 && bottom_size.width <= bounds.right - 4
            && top_size.height <= row_height && bottom_size.height <= row_height {
            selected_font = font;
            previous = old;
            break;
        }
        SelectObject(hdc, old);
        DeleteObject(font);
    }
    if selected_font.is_null() {
        selected_font = CreateFontW(-12, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, [83, 101, 103, 111, 101, 32, 85, 73, 0].as_ptr());
        previous = SelectObject(hdc, selected_font);
    }
    for (wide, mut rect) in [(top_wide, Rect { left: 2, top: 0, right: bounds.right - 2, bottom: midpoint }), (bottom_wide, Rect { left: 2, top: midpoint, right: bounds.right - 2, bottom: bounds.bottom })] {
        DrawTextW(hdc, wide.as_ptr(), wide.len() as i32, &mut rect, 0x00000001 | 0x00000004 | 0x00000020);
    }
    SelectObject(hdc, previous);
    DeleteObject(selected_font);
    if !bits.is_null() {
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, (bounds.right * bounds.bottom) as usize);
        let progress_color = taskbar_progress_color(
            snapshot.five_hour.remaining_percent,
            snapshot.weekly.as_ref().map(|window| window.remaining_percent),
        );
        let elapsed = REFRESH_CYCLE.get().expect("refresh cycle initialized").lock().unwrap().started.elapsed();
        let interval = REFRESH_INTERVAL_MS.load(Ordering::Relaxed) as u128;
        let erased = ((elapsed.as_millis().min(interval) * bounds.right as u128)
            / interval) as usize;
        for (index, pixel) in pixels.iter_mut().enumerate() {
            let text_alpha = 255 - (*pixel & 255);
            let progress_alpha = if progress_color.is_some() && index % bounds.right as usize >= erased { 52u32 } else { 0 };
            let background_alpha = progress_alpha * (255 - text_alpha) / 255;
            let alpha = (text_alpha + background_alpha).max(1);
            // UpdateLayeredWindow expects premultiplied BGRA pixels.
            let (red, green, blue) = progress_color.unwrap_or((0, 0, 0));
            let blue = blue * background_alpha / 255;
            let green = green * background_alpha / 255;
            let red = red * background_alpha / 255;
            *pixel = (alpha << 24) | (red << 16) | (green << 8) | blue;
        }
        let size = Size { width: bounds.right, height: bounds.bottom };
        let source = Point { x: 0, y: 0 };
        let position = Point { x: window.left, y: window.top };
        let blend = BlendFunction { operation: 0, flags: 0, constant_alpha: 255, alpha_format: 1 };
        UpdateLayeredWindow(hwnd, screen_dc, &position, &size, hdc, &source, 0, &blend, 2);
    }
    SelectObject(hdc, previous_bitmap);
    DeleteObject(bitmap);
    DeleteDC(hdc);
    ReleaseDC(null_mut(), screen_dc);
    EndPaint(hwnd, &paint);
}

fn taskbar_progress_color(remaining_percent: u8, weekly_remaining_percent: Option<u8>) -> Option<(u32, u32, u32)> {
    if weekly_remaining_percent == Some(0) { return None; }
    match remaining_percent {
        0 => None,
        1..=5 => Some((230, 65, 65)),
        6..=20 => Some((235, 195, 45)),
        _ => Some((55, 120, 220)),
    }
}

unsafe fn set_window_region(hwnd: Hwnd, expanded: bool) {
    let region = if expanded {
        CreateRoundRectRgn(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_HEIGHT, WINDOW_HEIGHT)
    } else {
        CreateEllipticRgn(0, 0, BALL_SIZE, BALL_SIZE)
    };
    SetWindowRgn(hwnd, region, 1);
}

unsafe fn paint_window(hwnd: Hwnd, snapshot: &AllowanceSnapshot) {
    let allowance = snapshot.five_hour.remaining_percent;
    let hovered = HOVERED.load(Ordering::Relaxed);
    let mut paint = PaintStruct { hdc: null_mut(), erase: 0, paint: Rect { left: 0, top: 0, right: 0, bottom: 0 }, restore: 0, inc_update: 0, reserved: [0; 32] };
    let hdc = BeginPaint(hwnd, &mut paint);
    let large_width = WINDOW_WIDTH * SCALE;
    let large_height = WINDOW_HEIGHT * SCALE;
    let buffer = CreateCompatibleDC(hdc);
    let bitmap = CreateCompatibleBitmap(hdc, large_width, large_height);
    let previous_bitmap = SelectObject(buffer, bitmap);
    let bounds = Rect { left: 0, top: 0, right: large_width, bottom: large_height };
    let background = CreateSolidBrush(rgb(27, 42, 41));
    FillRect(buffer, &bounds, background);
    DeleteObject(background);

    if hovered {
        let panel_brush = CreateSolidBrush(rgb(23, 47, 48));
        let panel = Rect { left: BALL_SIZE * SCALE, top: 0, right: WINDOW_WIDTH * SCALE, bottom: WINDOW_HEIGHT * SCALE };
        FillRect(buffer, &panel, panel_brush);
        DeleteObject(panel_brush);
    }

    let ball_brush = CreateSolidBrush(rgb(41, 71, 67));
    SelectObject(buffer, ball_brush);
    Ellipse(buffer, 6 * SCALE, 6 * SCALE, 116 * SCALE, 116 * SCALE);
    DeleteObject(ball_brush);

    let track_pen = CreatePen(PS_SOLID, 5 * SCALE, rgb(80, 108, 102));
    SelectObject(buffer, track_pen);
    SetArcDirection(buffer, AD_CLOCKWISE);
    Arc(buffer, 5 * SCALE, 5 * SCALE, 117 * SCALE, 117 * SCALE, 61 * SCALE, 5 * SCALE, 61 * SCALE, 5 * SCALE);
    DeleteObject(track_pen);

    let progress_pen = CreatePen(PS_SOLID, 5 * SCALE, rgb(154, 226, 180));
    SelectObject(buffer, progress_pen);
    let end_angle = 2.0 * std::f64::consts::PI * (allowance as f64 / 100.0) - std::f64::consts::FRAC_PI_2;
    let end_x = 61.0 + 56.0 * end_angle.cos();
    let end_y = 61.0 + 56.0 * end_angle.sin();
    Arc(buffer, 5 * SCALE, 5 * SCALE, 117 * SCALE, 117 * SCALE, 61 * SCALE, 5 * SCALE, (end_x * SCALE as f64).round() as i32, (end_y * SCALE as f64).round() as i32);
    DeleteObject(progress_pen);

    SetBkMode(buffer, TRANSPARENT);
    SetTextColor(buffer, rgb(243, 247, 245));
    let font = CreateFontW(51 * SCALE, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, [77, 97, 110, 114, 111, 112, 101, 0].as_ptr());
    SelectObject(buffer, font);
    let mut number_rect = Rect { left: 8 * SCALE, top: 25 * SCALE, right: 114 * SCALE, bottom: 78 * SCALE };
    let number_text: Vec<u16> = allowance.to_string().encode_utf16().collect();
    DrawTextW(buffer, number_text.as_ptr(), number_text.len() as i32, &mut number_rect, 0x00000001 | 0x00000004);
    DeleteObject(font);

    let label_font = CreateFontW(14 * SCALE, 0, 0, 0, 500, 0, 0, 0, 1, 0, 0, 5, 0, [68, 77, 32, 77, 111, 110, 111, 0].as_ptr());
    SelectObject(buffer, label_font);
    SetTextColor(buffer, rgb(168, 202, 184));
    let mut label_rect = Rect { left: 8 * SCALE, top: 81 * SCALE, right: 114 * SCALE, bottom: 108 * SCALE };
    let reset_text: Vec<u16> = format_reset_duration(snapshot.five_hour.resets_at)
        .encode_utf16()
        .collect();
    DrawTextW(buffer, reset_text.as_ptr(), reset_text.len() as i32, &mut label_rect, 0x00000001 | 0x00000004);
    DeleteObject(label_font);

    if hovered {
        let panel_font = CreateFontW(25 * SCALE, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, [77, 97, 110, 114, 111, 112, 101, 0].as_ptr());
        SelectObject(buffer, panel_font);
        SetTextColor(buffer, rgb(172, 208, 188));
        let weekly = snapshot.weekly.as_ref().map_or_else(|| "--".to_string(), |window| format!("WEEKLY  {}%", window.remaining_percent));
        let weekly_text: Vec<u16> = weekly.encode_utf16().collect();
        let mut weekly_rect = Rect { left: BALL_SIZE * SCALE, top: 14 * SCALE, right: (WINDOW_WIDTH - 12) * SCALE, bottom: 59 * SCALE };
        DrawTextW(buffer, weekly_text.as_ptr(), weekly_text.len() as i32, &mut weekly_rect, 0x00000004);

        let reset_text: Vec<u16> = snapshot.weekly.as_ref().map_or_else(|| "RESET IN  --".to_string(), |window| format!("RESET IN  {}", format_reset_countdown(window.resets_at))).encode_utf16().collect();
        let mut reset_rect = Rect { left: BALL_SIZE * SCALE, top: 77 * SCALE, right: (WINDOW_WIDTH - 12) * SCALE, bottom: 122 * SCALE };
        DrawTextW(buffer, reset_text.as_ptr(), reset_text.len() as i32, &mut reset_rect, 0x00000004);
        DeleteObject(panel_font);
    }

    SetStretchBltMode(hdc, HALFTONE);
    StretchBlt(hdc, 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, buffer, 0, 0, large_width, large_height, 0x00CC0020);
    SelectObject(buffer, previous_bitmap);
    DeleteObject(bitmap);
    DeleteDC(buffer);
    EndPaint(hwnd, &paint);
}

const fn rgb(red: u32, green: u32, blue: u32) -> u32 {
    red | (green << 8) | (blue << 16)
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn settings_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|path| PathBuf::from(path).join("CodexStatusBall").join("settings.txt"))
}

fn parse_refresh_settings(text: &str) -> (u32, u32) {
    // Older settings files contain only the refresh interval.
    if let Ok(interval) = text.trim().parse::<u32>() {
        let interval = if (MIN_REFRESH_SECONDS..=MAX_REFRESH_SECONDS).contains(&interval) { interval } else { 60 };
        return (interval, DEFAULT_PREFETCH_SECONDS.min(interval));
    }
    let mut interval = None;
    let mut prefetch = None;
    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "refresh_seconds" => interval = value.trim().parse::<u32>().ok(),
                "prefetch_seconds" => prefetch = value.trim().parse::<u32>().ok(),
                _ => {}
            }
        }
    }
    let interval = interval.filter(|seconds| (MIN_REFRESH_SECONDS..=MAX_REFRESH_SECONDS).contains(seconds)).unwrap_or(60);
    let prefetch = prefetch.filter(|seconds| *seconds <= interval).unwrap_or(DEFAULT_PREFETCH_SECONDS.min(interval));
    (interval, prefetch)
}

fn load_refresh_settings() -> (u32, u32) {
    settings_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map_or((60, DEFAULT_PREFETCH_SECONDS), |text| parse_refresh_settings(&text))
}

fn save_refresh_settings(seconds: u32, prefetch_seconds: u32) {
    if let Some(path) = settings_path() {
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("Could not create settings directory: {error}");
                return;
            }
        }
        if let Err(error) = fs::write(path, format!("refresh_seconds={seconds}\nprefetch_seconds={prefetch_seconds}\n")) {
            eprintln!("Could not save refresh settings: {error}");
        }
    }
}

unsafe fn show_settings() {
    if !SETTINGS_WINDOW.is_null() {
        SetForegroundWindow(SETTINGS_WINDOW);
        return;
    }
    let instance = GetModuleHandleW(null());
    let title = wide("设置");
    let x = (GetSystemMetrics(SM_CXSCREEN) - 360) / 2;
    let y = (GetSystemMetrics(SM_CYSCREEN) - 190) / 2;
    let hwnd = CreateWindowExW(0, SETTINGS_CLASS_NAME.as_ptr(), title.as_ptr(), WS_CAPTION | WS_SYSMENU, x, y, 360, 190, null_mut(), null_mut(), instance, null_mut());
    if hwnd.is_null() { return; }
    SETTINGS_WINDOW = hwnd;
    let static_class = wide("STATIC");
    let edit_class = wide("EDIT");
    let button_class = wide("BUTTON");
    let label = wide("刷新间隔（秒）:");
    let font = GetStockObject(17) as usize; // DEFAULT_GUI_FONT
    let refresh_label = CreateWindowExW(0, static_class.as_ptr(), label.as_ptr(), WS_CHILD | WS_VISIBLE, 24, 24, 155, 25, hwnd, null_mut(), instance, null_mut());
    SendMessageW(refresh_label, WM_SETFONT, font, 1);
    let current = wide(&(REFRESH_INTERVAL_MS.load(Ordering::Relaxed) / 1000).to_string());
    REFRESH_EDIT = CreateWindowExW(0, edit_class.as_ptr(), current.as_ptr(), WS_CHILD | WS_VISIBLE | WS_BORDER | 0x2000, 190, 20, 135, 25, hwnd, null_mut(), instance, null_mut());
    SendMessageW(REFRESH_EDIT, WM_SETFONT, font, 1);
    let label = wide("提前获取（秒）:");
    let prefetch_label = CreateWindowExW(0, static_class.as_ptr(), label.as_ptr(), WS_CHILD | WS_VISIBLE, 24, 64, 155, 25, hwnd, null_mut(), instance, null_mut());
    SendMessageW(prefetch_label, WM_SETFONT, font, 1);
    let current = wide(&PREFETCH_SECONDS.load(Ordering::Relaxed).to_string());
    PREFETCH_EDIT = CreateWindowExW(0, edit_class.as_ptr(), current.as_ptr(), WS_CHILD | WS_VISIBLE | WS_BORDER | 0x2000, 190, 60, 135, 25, hwnd, null_mut(), instance, null_mut());
    SendMessageW(PREFETCH_EDIT, WM_SETFONT, font, 1);
    let apply = wide("保存");
    let button = CreateWindowExW(0, button_class.as_ptr(), apply.as_ptr(), WS_CHILD | WS_VISIBLE | 0x0001, 235, 106, 90, 30, hwnd, APPLY_BUTTON_ID as Handle, instance, null_mut());
    SendMessageW(button, WM_SETFONT, font, 1);
    ShowWindow(hwnd, 5);
    SetForegroundWindow(hwnd);
}

pub fn run() {
    unsafe {
        let (interval, prefetch) = load_refresh_settings();
        REFRESH_INTERVAL_MS.store(interval * 1000, Ordering::Relaxed);
        PREFETCH_SECONDS.store(prefetch, Ordering::Relaxed);
        let snapshot = fetch_allowances().unwrap_or_else(|error| {
            eprintln!("Could not read Codex allowance: {error}");
            AllowanceSnapshot {
                five_hour: QuotaWindow {
                    remaining_percent: 0,
                    resets_at: None,
                    window_duration_mins: None,
                },
                weekly: None,
            }
        });
        SNAPSHOT.set(Mutex::new(snapshot)).expect("allowance snapshot initialized once");
        assert!(REFRESH_CYCLE.set(Mutex::new(RefreshCycle::new())).is_ok(), "refresh cycle initialized once");
        SetProcessDPIAware();
        let controls = InitCommonControlsEx { size: std::mem::size_of::<InitCommonControlsEx>() as u32, classes: 0x0000_00ff };
        InitCommonControlsEx(&controls);
        let instance = GetModuleHandleW(null());
        let class = WndClass { style: 0, window_proc, class_extra: 0, window_extra: 0, instance, icon: null_mut(), cursor: LoadCursorW(null_mut(), 32512usize as *const u16), background: null_mut(), menu_name: null(), class_name: CLASS_NAME.as_ptr() };
        RegisterClassW(&class);
        let settings_class = WndClass { style: 0, window_proc, class_extra: 0, window_extra: 0, instance, icon: null_mut(), cursor: LoadCursorW(null_mut(), 32512usize as *const u16), background: GetSysColorBrush(15), menu_name: null(), class_name: SETTINGS_CLASS_NAME.as_ptr() };
        RegisterClassW(&settings_class);
        let x = (GetSystemMetrics(SM_CXSCREEN) - BALL_SIZE) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - WINDOW_HEIGHT) / 2;
        let hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, CLASS_NAME.as_ptr(), WINDOW_TITLE.as_ptr(), WS_POPUP, x, y, WINDOW_WIDTH, WINDOW_HEIGHT, null_mut(), null_mut(), instance, null_mut());
        BALL_WINDOW = hwnd;
        TASKBAR_WINDOW = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED, CLASS_NAME.as_ptr(), WINDOW_TITLE.as_ptr(), WS_POPUP, 0, 0, TASKBAR_WIDTH, 40, null_mut(), null_mut(), instance, null_mut());
        let tooltip_class = wide("tooltips_class32");
        TASKBAR_TOOLTIP = CreateWindowExW(WS_EX_TOPMOST, tooltip_class.as_ptr(), null(), WS_POPUP | TTS_USEVISUALSTYLE, 0, 0, 0, 0, TASKBAR_WINDOW, null_mut(), instance, null_mut());
        if !TASKBAR_TOOLTIP.is_null() {
            let mut tool = taskbar_tool_info();
            SendMessageW(TASKBAR_TOOLTIP, TTM_ADDTOOLW, 0, &mut tool as *mut _ as Lparam);
            SendMessageW(TASKBAR_TOOLTIP, TTM_SETMAXTIPWIDTH, 0, 300);
            SendMessageW(TASKBAR_TOOLTIP, TTM_SETDELAYTIME, 2, 60_000); // TTDT_AUTOPOP
        }
        SetTimer(TASKBAR_WINDOW, TASKBAR_TOPMOST_TIMER, 250, null_mut());
        SetTimer(BALL_WINDOW, ALLOWANCE_REFRESH_TIMER, TASKBAR_ANIMATION_MS, null_mut());
        set_window_region(hwnd, false);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let mut message = Msg { hwnd: null_mut(), message: 0, w_param: 0, l_param: 0, time: 0, point: Point { x: 0, y: 0 } };
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

fn fetch_allowances() -> Result<AllowanceSnapshot, String> {
    let executable = find_codex_executable()?;
    let mut process = Command::new(executable);
    process
        .args(["app-server", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x0800_0000);
    }

    let mut child = process
        .spawn()
        .map_err(|error| format!("could not start Codex app-server: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Codex app-server stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Codex app-server stdout unavailable".to_string())?;
    let mut reader = BufReader::new(stdout);

    let result = (|| {
        send_request(
            &mut stdin,
            &mut reader,
            1,
            "initialize",
            json!({
                "clientInfo": {
                    "name": "codex-status-ball",
                    "title": "Codex Status Ball",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "experimentalApi": false,
                    "requestAttestation": false
                }
            }),
        )?;
        write_message(&mut stdin, &json!({"method": "initialized"}))?;

        let account = send_request(
            &mut stdin,
            &mut reader,
            2,
            "account/read",
            json!({"refreshToken": false}),
        )?;
        if account.get("account").is_none() || account.get("account").is_some_and(Value::is_null) {
            return Err("Codex account is not signed in".to_string());
        }

        let limits = send_request(&mut stdin, &mut reader, 3, "account/rateLimits/read", Value::Null)?;
        parse_allowance_snapshot(&limits)
    })();

    let _ = child.kill();
    let _ = child.wait();
    result
}

fn format_reset_countdown(resets_at: Option<i64>) -> String {
    let Some(resets_at) = resets_at else {
        return "--".to_string();
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);
    let seconds = (resets_at - now).max(0);
    format_reset_seconds(seconds)
}

fn format_reset_duration(resets_at: Option<i64>) -> String {
    let Some(resets_at) = resets_at else {
        return "--".to_string();
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);
    let seconds = (resets_at - now).max(0);
    format_duration_seconds(seconds)
}

fn format_weekly_reset_duration(resets_at: Option<i64>) -> String {
    let Some(resets_at) = resets_at else {
        return "--".to_string();
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);
    format_day_hour_seconds((resets_at - now).max(0))
}

fn format_day_hour_seconds(seconds: i64) -> String {
    if seconds == 0 {
        return "now".to_string();
    }
    format!("{}d {}h", seconds / 86_400, (seconds % 86_400) / 3_600)
}

fn format_reset_seconds(seconds: i64) -> String {
    if seconds == 0 {
        return "now".to_string();
    }
    if seconds >= 86_400 {
        return format!("{}d", seconds / 86_400);
    }
    if seconds >= 3_600 {
        return format!("{}h", seconds / 3_600);
    }
    format!("{}m", (seconds / 60).max(1))
}

fn format_duration_seconds(seconds: i64) -> String {
    if seconds == 0 {
        return "now".to_string();
    }
    format!("{}h {:02}m", seconds / 3_600, (seconds % 3_600) / 60)
}

fn parse_allowance_snapshot(limits: &Value) -> Result<AllowanceSnapshot, String> {
    let bucket = limits
        .get("rateLimitsByLimitId")
        .and_then(|buckets| buckets.get("codex"))
        .or_else(|| limits.get("rateLimits"))
        .ok_or_else(|| "Codex did not return rate limits".to_string())?;

    let five_hour = parse_quota_window(
        bucket
            .get("primary")
            .ok_or_else(|| "Codex did not return the 5-hour allowance".to_string())?,
        "5-hour",
    )?;
    let weekly = bucket
        .get("secondary")
        .map(|window| parse_quota_window(window, "weekly"))
        .transpose()?;

    Ok(AllowanceSnapshot { five_hour, weekly })
}

fn parse_quota_window(window: &Value, name: &str) -> Result<QuotaWindow, String> {
    let used_percent = window
        .get("usedPercent")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Codex did not return the {name} used percentage"))?;

    Ok(QuotaWindow {
        remaining_percent: (100 - used_percent.clamp(0, 100)) as u8,
        resets_at: window.get("resetsAt").and_then(Value::as_i64),
        window_duration_mins: window.get("windowDurationMins").and_then(Value::as_i64),
    })
}

fn find_codex_executable() -> Result<PathBuf, String> {
    // where.exe is a console program too; hide its window just like app-server.
    let mut where_command = Command::new("where.exe");
    where_command
        .arg("codex.exe")
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        where_command.creation_flags(0x0800_0000);
    }
    if let Ok(output) = where_command.output()
    {
        if let Some(path) = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .find(|path| !path.is_empty())
        {
            return Ok(PathBuf::from(path));
        }
    }

    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let path = PathBuf::from(user_profile)
            .join(".codex")
            .join(".sandbox-bin")
            .join("codex.exe");
        if path.is_file() {
            return Ok(path);
        }
    }

    Err("Codex CLI was not found in PATH or the standard Windows install location".to_string())
}

fn send_request(
    stdin: &mut ChildStdin,
    reader: &mut BufReader<ChildStdout>,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    write_message(stdin, &json!({"method": method, "id": id, "params": params}))?;
    let mut line = String::new();
    loop {
        line.clear();
        if reader
            .read_line(&mut line)
            .map_err(|error| format!("could not read Codex response: {error}"))?
            == 0
        {
            return Err("Codex app-server disconnected".to_string());
        }
        let message: Value = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(_) => continue,
        };
        if message.get("id").and_then(Value::as_u64) != Some(id) {
            continue;
        }
        if let Some(error) = message.get("error") {
            return Err(error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Codex request failed")
                .to_string());
        }
        return Ok(message.get("result").cloned().unwrap_or(Value::Null));
    }
}

fn write_message(stdin: &mut ChildStdin, message: &Value) -> Result<(), String> {
    writeln!(stdin, "{message}")
        .map_err(|error| format!("could not write to Codex app-server: {error}"))?;
    stdin
        .flush()
        .map_err(|error| format!("could not flush Codex request: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{format_day_hour_seconds, format_duration_seconds, format_reset_seconds, parse_allowance_snapshot, parse_refresh_settings, taskbar_progress_color, AllowanceSnapshot, QuotaWindow};
    use serde_json::json;

    #[test]
    fn parses_remaining_percentages_and_reset_times() {
        let limits = json!({
            "rateLimitsByLimitId": {
                "codex": {
                    "primary": {
                        "usedPercent": 28,
                        "windowDurationMins": 300,
                        "resetsAt": 1_800
                    },
                    "secondary": {
                        "usedPercent": 43,
                        "windowDurationMins": 10_080,
                        "resetsAt": 2_400
                    }
                }
            }
        });

        assert_eq!(
            parse_allowance_snapshot(&limits).unwrap(),
            AllowanceSnapshot {
                five_hour: QuotaWindow {
                    remaining_percent: 72,
                    resets_at: Some(1_800),
                    window_duration_mins: Some(300),
                },
                weekly: Some(QuotaWindow {
                    remaining_percent: 57,
                    resets_at: Some(2_400),
                    window_duration_mins: Some(10_080),
                }),
            }
        );
    }

    #[test]
    fn clamps_percentages_and_supports_missing_weekly_window() {
        let limits = json!({
            "rateLimits": {
                "primary": { "usedPercent": 140, "resetsAt": null }
            }
        });

        assert_eq!(
            parse_allowance_snapshot(&limits).unwrap(),
            AllowanceSnapshot {
                five_hour: QuotaWindow {
                    remaining_percent: 0,
                    resets_at: None,
                    window_duration_mins: None,
                },
                weekly: None,
            }
        );
    }

    #[test]
    fn formats_only_the_largest_countdown_unit() {
        assert_eq!(format_reset_seconds(2 * 86_400 + 3_600), "2d");
        assert_eq!(format_reset_seconds(5 * 3_600 + 30 * 60), "5h");
        assert_eq!(format_reset_seconds(45 * 60 + 20), "45m");
    }

    #[test]
    fn formats_five_hour_countdown_with_hours_and_minutes() {
        assert_eq!(format_duration_seconds(5 * 3_600 + 3 * 60), "5h 03m");
        assert_eq!(format_duration_seconds(45 * 60), "0h 45m");
    }

    #[test]
    fn formats_weekly_countdown_with_days_and_hours() {
        assert_eq!(format_day_hour_seconds(2 * 86_400 + 3 * 3_600 + 35 * 60), "2d 3h");
        assert_eq!(format_day_hour_seconds(5 * 3_600), "0d 5h");
        assert_eq!(format_day_hour_seconds(0), "now");
    }

    #[test]
    fn loads_legacy_and_new_refresh_settings() {
        assert_eq!(parse_refresh_settings("60"), (60, 5));
        assert_eq!(parse_refresh_settings("3"), (3, 3));
        assert_eq!(parse_refresh_settings("refresh_seconds=30\nprefetch_seconds=8\n"), (30, 8));
        assert_eq!(parse_refresh_settings("refresh_seconds=30\nprefetch_seconds=0\n"), (30, 0));
        assert_eq!(parse_refresh_settings("refresh_seconds=30\nprefetch_seconds=31\n"), (30, 5));
    }

    #[test]
    fn selects_taskbar_progress_color_at_quota_boundaries() {
        assert_eq!(taskbar_progress_color(0, Some(50)), None);
        assert_eq!(taskbar_progress_color(1, Some(50)), Some((230, 65, 65)));
        assert_eq!(taskbar_progress_color(5, Some(50)), Some((230, 65, 65)));
        assert_eq!(taskbar_progress_color(6, Some(50)), Some((235, 195, 45)));
        assert_eq!(taskbar_progress_color(19, Some(50)), Some((235, 195, 45)));
        assert_eq!(taskbar_progress_color(20, Some(50)), Some((235, 195, 45)));
        assert_eq!(taskbar_progress_color(21, Some(50)), Some((55, 120, 220)));
        assert_eq!(taskbar_progress_color(100, Some(0)), None);
        assert_eq!(taskbar_progress_color(100, None), Some((55, 120, 220)));
    }
}
