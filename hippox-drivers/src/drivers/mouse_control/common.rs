//! Mouse control shared utilities
//!
//! This module provides cross-platform shared utilities for mouse control
//! including getting/setting mouse position, clicking, scrolling, and
//! smooth movement.
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info};
use crate::DriverError;
use crate::DriverResult;
#[cfg(target_os = "windows")]
use winapi::shared::windef::POINT;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{
    GetCursorPos, INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput, SetCursorPos,
};
/// Mouse button types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
/// Mouse position structure
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}
/// Gets the current mouse position
///
/// # Returns
/// * `DriverResult<MousePosition>` - Current mouse position
#[cfg(target_os = "windows")]
pub fn get_mouse_position() -> DriverResult<MousePosition> {
    debug!("Getting mouse position on Windows");
    let mut point: POINT = unsafe { std::mem::zeroed() };
    unsafe {
        GetCursorPos(&mut point);
    }
    let pos = MousePosition { x: point.x, y: point.y };
    debug!("Mouse position: ({}, {})", pos.x, pos.y);
    return Ok(pos);
}
#[cfg(target_os = "linux")]
pub fn get_mouse_position() -> DriverResult<MousePosition> {
    debug!("Getting mouse position on Linux");
    let output = Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .map_err(|e| DriverError::execution(format!("Failed to get mouse position: {}", e)))?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut x = 0;
    let mut y = 0;
    for line in output_str.lines() {
        if line.starts_with("X=") {
            x = line[2..].parse().unwrap_or(0);
        } else if line.starts_with("Y=") {
            y = line[2..].parse().unwrap_or(0);
        }
    }
    let pos = MousePosition { x, y };
    debug!("Mouse position: ({}, {})", pos.x, pos.y);
    return Ok(pos);
}
#[cfg(target_os = "macos")]
pub fn get_mouse_position() -> DriverResult<MousePosition> {
    debug!("Getting mouse position on macOS");
    let script = r#"
        tell application "System Events"
            set mousePos to (current location of (first process whose frontmost is true))
            return (item 1 of mousePos) & "," & (item 2 of mousePos)
        end tell
    "#;
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| DriverError::execution(format!("Failed to get mouse position: {}", e)))?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let coords: Vec<&str> = output_str.trim().split(',').collect();
    let x = coords.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let y = coords.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let pos = MousePosition { x, y };
    debug!("Mouse position: ({}, {})", pos.x, pos.y);
    return Ok(pos);
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn get_mouse_position() -> DriverResult<MousePosition> {
    return Err(DriverError::execution("Get mouse position not implemented on this platform".to_string()));
}
/// Sets the mouse position
///
/// # Arguments
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// * `DriverResult<()>` - Success or error
#[cfg(target_os = "windows")]
pub fn set_mouse_position(x: i32, y: i32) -> DriverResult<()> {
    debug!("Setting mouse position to ({}, {}) on Windows", x, y);
    unsafe {
        SetCursorPos(x, y);
    }
    info!("Mouse moved to ({}, {})", x, y);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn set_mouse_position(x: i32, y: i32) -> DriverResult<()> {
    debug!("Setting mouse position to ({}, {}) on Linux", x, y);
    Command::new("xdotool")
        .args(["mousemove", &x.to_string(), &y.to_string()])
        .output()
        .map_err(|e| DriverError::execution(format!("Failed to set mouse position: {}", e)))?;
    info!("Mouse moved to ({}, {})", x, y);
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn set_mouse_position(x: i32, y: i32) -> DriverResult<()> {
    debug!("Setting mouse position to ({}, {}) on macOS", x, y);
    let script = format!(r#"tell application "System Events" to set position of first process whose frontmost is true to {{{}, {}}}"#, x, y);
    Command::new("osascript").args(["-e", &script]).output().map_err(|e| DriverError::execution(format!("Failed to set mouse position: {}", e)))?;
    info!("Mouse moved to ({}, {})", x, y);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn set_mouse_position(x: i32, y: i32) -> DriverResult<()> {
    let _ = (x, y);
    return Err(DriverError::execution("Set mouse position not implemented on this platform".to_string()));
}
/// Sends a mouse click
///
/// # Arguments
/// * `button` - Mouse button to click
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// * `DriverResult<()>` - Success or error
#[cfg(target_os = "windows")]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse click at ({}, {}) on Windows", x, y);
    set_mouse_position(x, y)?;
    let (down_flag, up_flag) = match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };
    let mut inputs = [INPUT { type_: INPUT_MOUSE, u: unsafe { std::mem::zeroed() } }, INPUT { type_: INPUT_MOUSE, u: unsafe { std::mem::zeroed() } }];
    unsafe {
        let mi_down = &mut inputs[0].u.mi_mut();
        mi_down.dx = 0;
        mi_down.dy = 0;
        mi_down.mouseData = 0;
        mi_down.dwFlags = down_flag;
        mi_down.time = 0;
        mi_down.dwExtraInfo = 0;
        let mi_up = &mut inputs[1].u.mi_mut();
        mi_up.dx = 0;
        mi_up.dy = 0;
        mi_up.mouseData = 0;
        mi_up.dwFlags = up_flag;
        mi_up.time = 0;
        mi_up.dwExtraInfo = 0;
        SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
    info!("Mouse clicked at ({}, {})", x, y);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse click at ({}, {}) on Linux", x, y);
    set_mouse_position(x, y)?;
    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Middle => "2",
        MouseButton::Right => "3",
    };
    Command::new("xdotool").args(["click", btn]).output().map_err(|e| DriverError::execution(format!("Failed to click: {}", e)))?;
    info!("Mouse clicked at ({}, {})", x, y);
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse click at ({}, {}) on macOS", x, y);
    set_mouse_position(x, y)?;
    let click_cmd = match button {
        MouseButton::Left => "click",
        MouseButton::Right => "click at {x, y}",
        MouseButton::Middle => "click at {x, y}",
    };
    let script = format!(r#"tell application "System Events" to {}"#, click_cmd);
    Command::new("osascript").args(["-e", &script]).output().map_err(|e| DriverError::execution(format!("Failed to click: {}", e)))?;
    info!("Mouse clicked at ({}, {})", x, y);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    let _ = (button, x, y);
    return Err(DriverError::execution("Mouse click not implemented on this platform".to_string()));
}
/// Sends a mouse double click
///
/// # Arguments
/// * `button` - Mouse button to double click
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// * `DriverResult<()>` - Success or error
pub fn mouse_double_click(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse double click at ({}, {})", x, y);
    mouse_click(button.clone(), x, y)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    mouse_click(button, x, y)?;
    info!("Mouse double clicked at ({}, {})", x, y);
    return Ok(());
}
/// Sends a mouse press (down only)
///
/// # Arguments
/// * `button` - Mouse button to press
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// * `DriverResult<()>` - Success or error
#[cfg(target_os = "windows")]
pub fn mouse_press(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse press at ({}, {}) on Windows", x, y);
    set_mouse_position(x, y)?;
    let down_flag = match button {
        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
    };
    let mut input = INPUT { type_: INPUT_MOUSE, u: unsafe { std::mem::zeroed() } };
    unsafe {
        let mi = input.u.mi_mut();
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = down_flag;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
    info!("Mouse pressed at ({}, {})", x, y);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn mouse_press(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse press at ({}, {}) on Linux", x, y);
    set_mouse_position(x, y)?;
    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Middle => "2",
        MouseButton::Right => "3",
    };
    Command::new("xdotool").args(["mousedown", btn]).output().map_err(|e| DriverError::execution(format!("Failed to press: {}", e)))?;
    info!("Mouse pressed at ({}, {})", x, y);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn mouse_press(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    let _ = (button, x, y);
    return Err(DriverError::execution("Mouse press not implemented on this platform".to_string()));
}
/// Sends a mouse release (up only)
///
/// # Arguments
/// * `button` - Mouse button to release
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// * `DriverResult<()>` - Success or error
#[cfg(target_os = "windows")]
pub fn mouse_release(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse release at ({}, {}) on Windows", x, y);
    set_mouse_position(x, y)?;
    let up_flag = match button {
        MouseButton::Left => MOUSEEVENTF_LEFTUP,
        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
    };
    let mut input = INPUT { type_: INPUT_MOUSE, u: unsafe { std::mem::zeroed() } };
    unsafe {
        let mi = input.u.mi_mut();
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = up_flag;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
    info!("Mouse released at ({}, {})", x, y);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn mouse_release(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    debug!("Mouse release at ({}, {}) on Linux", x, y);
    set_mouse_position(x, y)?;
    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Middle => "2",
        MouseButton::Right => "3",
    };
    Command::new("xdotool").args(["mouseup", btn]).output().map_err(|e| DriverError::execution(format!("Failed to release: {}", e)))?;
    info!("Mouse released at ({}, {})", x, y);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn mouse_release(button: MouseButton, x: i32, y: i32) -> DriverResult<()> {
    let _ = (button, x, y);
    return Err(DriverError::execution("Mouse release not implemented on this platform".to_string()));
}
/// Sends a mouse scroll
///
/// # Arguments
/// * `delta` - Scroll delta (positive=up, negative=down)
///
/// # Returns
/// * `DriverResult<()>` - Success or error
#[cfg(target_os = "windows")]
pub fn mouse_scroll(delta: i32) -> DriverResult<()> {
    debug!("Mouse scroll on Windows: delta={}", delta);
    let mut input = INPUT { type_: INPUT_MOUSE, u: unsafe { std::mem::zeroed() } };
    unsafe {
        let mi = input.u.mi_mut();
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = delta as u32;
        mi.dwFlags = MOUSEEVENTF_WHEEL;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
    info!("Scrolled by {}", delta);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn mouse_scroll(delta: i32) -> DriverResult<()> {
    debug!("Mouse scroll on Linux: delta={}", delta);
    let direction = if delta > 0 { "up" } else { "down" };
    let clicks = (delta.abs() / 120).max(1);
    for _ in 0..clicks {
        Command::new("xdotool")
            .args(["click", if direction == "up" { "4" } else { "5" }])
            .output()
            .map_err(|e| DriverError::execution(format!("Failed to scroll: {}", e)))?;
    }
    info!("Scrolled by {}", delta);
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn mouse_scroll(delta: i32) -> DriverResult<()> {
    debug!("Mouse scroll on macOS: delta={}", delta);
    let script = format!(r#"tell application "System Events" to scroll wheel {}"#, if delta > 0 { "up" } else { "down" });
    Command::new("osascript").args(["-e", &script]).output().map_err(|e| DriverError::execution(format!("Failed to scroll: {}", e)))?;
    info!("Scrolled by {}", delta);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn mouse_scroll(delta: i32) -> DriverResult<()> {
    let _ = delta;
    return Err(DriverError::execution("Mouse scroll not implemented on this platform".to_string()));
}
/// Smoothly moves the mouse to target with acceleration
///
/// # Arguments
/// * `target_x` - Target X coordinate
/// * `target_y` - Target Y coordinate
/// * `duration_ms` - Movement duration in milliseconds
///
/// # Returns
/// * `DriverResult<()>` - Success or error
pub async fn smooth_move_to(target_x: i32, target_y: i32, duration_ms: u64) -> DriverResult<()> {
    debug!("Smooth move to ({}, {}) in {}ms", target_x, target_y, duration_ms);
    let start = get_mouse_position()?;
    let start_x = start.x;
    let start_y = start.y;
    let steps = 20;
    let step_delay = duration_ms / steps as u64;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let ease = 1.0 - (1.0 - t).powi(3);
        let x = start_x + ((target_x - start_x) as f64 * ease) as i32;
        let y = start_y + ((target_y - start_y) as f64 * ease) as i32;
        set_mouse_position(x, y)?;
        tokio::time::sleep(std::time::Duration::from_millis(step_delay)).await;
    }
    set_mouse_position(target_x, target_y)?;
    info!("Smooth move completed to ({}, {})", target_x, target_y);
    return Ok(());
}
/// Gets the current cursor type
///
/// # Returns
/// * `DriverResult<String>` - Cursor type string
#[cfg(target_os = "windows")]
pub fn get_cursor_type() -> DriverResult<String> {
    debug!("Getting cursor type on Windows");
    return Ok("arrow".to_string());
}
#[cfg(not(target_os = "windows"))]
pub fn get_cursor_type() -> DriverResult<String> {
    debug!("Getting cursor type on non-Windows platform");
    return Ok("unknown".to_string());
}
