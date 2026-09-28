#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

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
const DESIGN_SIZE: i32 = 174;
const WINDOW_SIZE: i32 = 122;

#[repr(C)]
struct Point { x: i32, y: i32 }

#[repr(C)]
struct Rect { left: i32, top: i32, right: i32, bottom: i32 }

#[repr(C)]
struct Msg { hwnd: Hwnd, message: u32, w_param: Wparam, l_param: Lparam, time: u32, point: Point }

#[repr(C)]
struct PaintStruct { hdc: Hdc, erase: i32, paint: Rect, reserved: i32 }

type WindowProc = unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult;

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
        WM_PAINT => 0,
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, w_param, l_param),
    }
}

unsafe fn paint_window(hwnd: Hwnd, snapshot: &AllowanceSnapshot) {
    let allowance = snapshot.five_hour.remaining_percent;
    let mut paint = PaintStruct { hdc: null_mut(), erase: 0, paint: Rect { left: 0, top: 0, right: 0, bottom: 0 }, reserved: 0 };
    let hdc = BeginPaint(hwnd, &mut paint);
    let large_size = DESIGN_SIZE * SCALE;
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
    let end_angle = 2.0 * std::f64::consts::PI * (allowance as f64 / 100.0) - std::f64::consts::FRAC_PI_2;
    let end_x = 87.0 + 72.0 * end_angle.cos();
    let end_y = 87.0 + 72.0 * end_angle.sin();
    Arc(buffer, 15 * SCALE, 15 * SCALE, 159 * SCALE, 159 * SCALE, 87 * SCALE, 15 * SCALE, (end_x * SCALE as f64).round() as i32, (end_y * SCALE as f64).round() as i32);
    DeleteObject(progress_pen);

    SetBkMode(buffer, TRANSPARENT);
    SetTextColor(buffer, rgb(243, 247, 245));
    let font = CreateFontW(52 * SCALE, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, [77, 97, 110, 111, 112, 111, 108, 0].as_ptr());
    SelectObject(buffer, font);
    let mut number_rect = Rect { left: 20 * SCALE, top: 47 * SCALE, right: 154 * SCALE, bottom: 111 * SCALE };
    let number_text: Vec<u16> = allowance.to_string().encode_utf16().collect();
    DrawTextW(buffer, number_text.as_ptr(), number_text.len() as i32, &mut number_rect, 0x00000001 | 0x00000004);
    DeleteObject(font);

    let label_font = CreateFontW(13 * SCALE, 0, 0, 0, 500, 0, 0, 0, 1, 0, 0, 5, 0, [67, 111, 110, 115, 111, 108, 97, 115, 0].as_ptr());
    SelectObject(buffer, label_font);
    SetTextColor(buffer, rgb(168, 202, 184));
    let mut label_rect = Rect { left: 20 * SCALE, top: 108 * SCALE, right: 154 * SCALE, bottom: 137 * SCALE };
    let reset_text: Vec<u16> = format_reset_countdown(snapshot.five_hour.resets_at)
        .encode_utf16()
        .collect();
    DrawTextW(buffer, reset_text.as_ptr(), reset_text.len() as i32, &mut label_rect, 0x00000001 | 0x00000004);
    DeleteObject(label_font);

    SetStretchBltMode(hdc, HALFTONE);
    StretchBlt(hdc, 0, 0, WINDOW_SIZE, WINDOW_SIZE, buffer, 0, 0, large_size, large_size, 0x00CC0020);
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
        SetProcessDPIAware();
        let instance = GetModuleHandleW(null());
        let class = WndClass { style: 0, window_proc, class_extra: 0, window_extra: 0, instance, icon: null_mut(), cursor: null_mut(), background: null_mut(), menu_name: null(), class_name: CLASS_NAME.as_ptr() };
        RegisterClassW(&class);
        let size = WINDOW_SIZE;
        let x = (GetSystemMetrics(SM_CXSCREEN) - size) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - size) / 2;
        let hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, CLASS_NAME.as_ptr(), WINDOW_TITLE.as_ptr(), WS_POPUP, x, y, size, size, null_mut(), null_mut(), instance, null_mut());
        SetWindowRgn(hwnd, CreateEllipticRgn(0, 0, size, size), 1);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let mut message = Msg { hwnd: null_mut(), message: 0, w_param: 0, l_param: 0, time: 0, point: Point { x: 0, y: 0 } };
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            if message.message == WM_PAINT {
                paint_window(hwnd, &snapshot);
            } else {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
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
    if seconds == 0 {
        return "now".to_string();
    }
    if seconds >= 3600 {
        return format!("{}h {:02}m", seconds / 3600, (seconds % 3600) / 60);
    }
    format!("{}m", (seconds / 60).max(1))
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
    if let Ok(output) = Command::new("where.exe")
        .arg("codex.exe")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
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
    use super::{parse_allowance_snapshot, AllowanceSnapshot, QuotaWindow};
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
}
