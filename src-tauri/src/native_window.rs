#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::ptr::{null, null_mut};

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
const WINDOW_TITLE: &[u16] = &[67, 111, 100, 101, 120, 32, 83, 116, 97, 116, 117, 115, 32, 66, 97, 108, 108, 0];
const WM_PAINT: u32 = 0x000F;
const WM_DESTROY: u32 = 0x0002;
const WM_NCHITTEST: u32 = 0x0084;
const HTCAPTION: Lresult = 2;
const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_NOACTIVATE: u32 = 0x08000000;
const SW_SHOWNOACTIVATE: i32 = 4;
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

#[repr(C)]
struct Point { x: i32, y: i32 }

#[repr(C)]
struct Rect { left: i32, top: i32, right: i32, bottom: i32 }

#[repr(C)]
struct Msg { hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam, time: u32, point: Point }

#[repr(C)]
struct PaintStruct { hdc: Hdc, erase: i32, paint: Rect, reserved: i32 }

type WindowProc = unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult;

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
    fn GetModuleHandleW(name: *const u16) -> Hinstance;
    fn GetSystemMetrics(index: i32) -> i32;
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn SetWindowPos(hwnd: Hwnd, insert_after: Hwnd, x: i32, y: i32, width: i32, height: i32, flags: u32) -> i32;
    fn SetWindowRgn(hwnd: Hwnd, region: Hregion, redraw: i32) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
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
    fn CreateCompatibleBitmap(hdc: Hdc, width: i32, height: i32) -> Handle;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn DeleteDC(hdc: Hdc) -> i32;
    fn SetStretchBltMode(hdc: Hdc, mode: i32) -> i32;
    fn StretchBlt(destination: Hdc, x: i32, y: i32, width: i32, height: i32, source: Hdc, source_x: i32, source_y: i32, source_width: i32, source_height: i32, operation: u32) -> i32;
    fn SelectObject(hdc: Hdc, object: Handle) -> Handle;
    fn SetArcDirection(hdc: Hdc, direction: i32) -> i32;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: u32) -> u32;
}

unsafe extern "system" fn window_proc(hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam) -> Lresult {
    match message {
        WM_NCHITTEST => HTCAPTION,
        WM_PAINT => {
            paint_window(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, w_param, l_param),
    }
}

unsafe fn paint_window(hwnd: Hwnd) {
    let mut paint = PaintStruct { hdc: null_mut(), erase: 0, paint: Rect { left: 0, top: 0, right: 0, bottom: 0 }, reserved: 0 };
    let hdc = BeginPaint(hwnd, &mut paint);
    let large_size = 174 * SCALE;
    let buffer = CreateCompatibleDC(hdc);
    let bitmap = CreateCompatibleBitmap(hdc, large_size, large_size);
    let previous_bitmap = SelectObject(buffer, bitmap);
    let bounds = Rect { left: 0, top: 0, right: large_size, bottom: large_size };
    let background = CreateSolidBrush(rgb(27, 42, 41));
    FillRect(buffer, &bounds, background);
    DeleteObject(background);

    let ball_brush = CreateSolidBrush(rgb(36, 63, 60));
    SelectObject(buffer, ball_brush);
    Ellipse(buffer, 8 * SCALE, 8 * SCALE, 166 * SCALE, 166 * SCALE);
    DeleteObject(ball_brush);

    let track_pen = CreatePen(PS_SOLID, 5 * SCALE, rgb(80, 108, 102));
    SelectObject(buffer, track_pen);
    SetArcDirection(buffer, AD_CLOCKWISE);
    Arc(buffer, 15 * SCALE, 15 * SCALE, 159 * SCALE, 159 * SCALE, 87 * SCALE, 15 * SCALE, 87 * SCALE, 15 * SCALE);
    DeleteObject(track_pen);

    let progress_pen = CreatePen(PS_SOLID, 5 * SCALE, rgb(154, 226, 180));
    SelectObject(buffer, progress_pen);
    let end_angle = 2.0 * std::f64::consts::PI * 0.78 - std::f64::consts::FRAC_PI_2;
    let end_x = 87.0 + 72.0 * end_angle.cos();
    let end_y = 87.0 + 72.0 * end_angle.sin();
    Arc(buffer, 15 * SCALE, 15 * SCALE, 159 * SCALE, 159 * SCALE, 87 * SCALE, 15 * SCALE, (end_x * SCALE as f64).round() as i32, (end_y * SCALE as f64).round() as i32);
    DeleteObject(progress_pen);

    SetBkMode(buffer, TRANSPARENT);
    SetTextColor(buffer, rgb(243, 247, 245));
    let font = CreateFontW(52 * SCALE, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, [77, 97, 110, 111, 112, 111, 108, 0].as_ptr());
    SelectObject(buffer, font);
    let mut number_rect = Rect { left: 20 * SCALE, top: 47 * SCALE, right: 154 * SCALE, bottom: 111 * SCALE };
    DrawTextW(buffer, [55, 56, 0].as_ptr(), 2, &mut number_rect, 0x00000001 | 0x00000004);
    DeleteObject(font);

    let label_font = CreateFontW(13 * SCALE, 0, 0, 0, 500, 0, 0, 0, 1, 0, 0, 5, 0, [67, 111, 110, 115, 111, 108, 97, 115, 0].as_ptr());
    SelectObject(buffer, label_font);
    SetTextColor(buffer, rgb(168, 202, 184));
    let mut label_rect = Rect { left: 20 * SCALE, top: 108 * SCALE, right: 154 * SCALE, bottom: 137 * SCALE };
    DrawTextW(buffer, [53, 72, 0].as_ptr(), 2, &mut label_rect, 0x00000001 | 0x00000004);
    DeleteObject(label_font);

    SetStretchBltMode(hdc, HALFTONE);
    StretchBlt(hdc, 0, 0, 174, 174, buffer, 0, 0, large_size, large_size, 0x00CC0020);
    SelectObject(buffer, previous_bitmap);
    DeleteObject(bitmap);
    DeleteDC(buffer);
    EndPaint(hwnd, &paint);
}

const fn rgb(red: u32, green: u32, blue: u32) -> u32 {
    red | (green << 8) | (blue << 16)
}

pub fn run() {
    unsafe {
        SetProcessDPIAware();
        let instance = GetModuleHandleW(null());
        let class = WndClass { style: 0, window_proc, class_extra: 0, window_extra: 0, instance, icon: null_mut(), cursor: null_mut(), background: null_mut(), menu_name: null(), class_name: CLASS_NAME.as_ptr() };
        RegisterClassW(&class);
        let size = 174;
        let x = (GetSystemMetrics(SM_CXSCREEN) - size) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - size) / 2;
        let hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, CLASS_NAME.as_ptr(), WINDOW_TITLE.as_ptr(), WS_POPUP, x, y, size, size, null_mut(), null_mut(), instance, null_mut());
        SetWindowRgn(hwnd, CreateEllipticRgn(0, 0, size, size), 1);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let mut message = Msg { hwnd: null_mut(), message: 0, w_param: 0, l_param: 0, time: 0, point: Point { x: 0, y: 0 } };
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
