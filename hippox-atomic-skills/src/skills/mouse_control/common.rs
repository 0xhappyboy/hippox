// mouse_control/shared.rs
//! Shared utilities for mouse control across platforms

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use winapi::shared::windef::POINT;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{
    GetCursorPos, INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput, SetCursorPos,
};

/// Mouse button types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

/// Get current mouse position
#[cfg(target_os = "windows")]
pub fn get_mouse_position() -> Result<MousePosition> {
    let mut point: POINT = unsafe { std::mem::zeroed() };
    unsafe {
        GetCursorPos(&mut point);
    }
    Ok(MousePosition {
        x: point.x,
        y: point.y,
    })
}

#[cfg(target_os = "linux")]
pub fn get_mouse_position() -> Result<MousePosition> {
    let output = Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()?;
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
    Ok(MousePosition { x, y })
}

#[cfg(target_os = "macos")]
pub fn get_mouse_position() -> Result<MousePosition> {
    let script = r#"
        tell application "System Events"
            set mousePos to (current location of (first process whose frontmost is true))
            return (item 1 of mousePos) & "," & (item 2 of mousePos)
        end tell
    "#;
    let output = Command::new("osascript").args(["-e", script]).output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let coords: Vec<&str> = output_str.trim().split(',').collect();
    let x = coords.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let y = coords.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Ok(MousePosition { x, y })
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn get_mouse_position() -> Result<MousePosition> {
    anyhow::bail!("Get mouse position not implemented on this platform")
}

/// Set mouse position
#[cfg(target_os = "windows")]
pub fn set_mouse_position(x: i32, y: i32) -> Result<()> {
    unsafe {
        SetCursorPos(x, y);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn set_mouse_position(x: i32, y: i32) -> Result<()> {
    Command::new("xdotool")
        .args(["mousemove", &x.to_string(), &y.to_string()])
        .output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_mouse_position(x: i32, y: i32) -> Result<()> {
    let script = format!(
        r#"tell application "System Events" to set position of first process whose frontmost is true to {{{}, {}}}"#,
        x, y
    );
    Command::new("osascript").args(["-e", &script]).output()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn set_mouse_position(x: i32, y: i32) -> Result<()> {
    let _ = (x, y);
    anyhow::bail!("Set mouse position not implemented on this platform")
}

/// Send mouse click
#[cfg(target_os = "windows")]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> Result<()> {
    // Move to position first
    set_mouse_position(x, y)?;

    let (down_flag, up_flag) = match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };

    let mut inputs = [
        INPUT {
            type_: INPUT_MOUSE,
            u: unsafe { std::mem::zeroed() },
        },
        INPUT {
            type_: INPUT_MOUSE,
            u: unsafe { std::mem::zeroed() },
        },
    ];

    unsafe {
        // First input - mouse down
        let mi_down = &mut inputs[0].u.mi_mut();
        mi_down.dx = 0;
        mi_down.dy = 0;
        mi_down.mouseData = 0;
        mi_down.dwFlags = down_flag;
        mi_down.time = 0;
        mi_down.dwExtraInfo = 0;

        // Second input - mouse up
        let mi_up = &mut inputs[1].u.mi_mut();
        mi_up.dx = 0;
        mi_up.dy = 0;
        mi_up.mouseData = 0;
        mi_up.dwFlags = up_flag;
        mi_up.time = 0;
        mi_up.dwExtraInfo = 0;

        SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> Result<()> {
    set_mouse_position(x, y)?;

    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Middle => "2",
        MouseButton::Right => "3",
    };

    Command::new("xdotool").args(["click", btn]).output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> Result<()> {
    set_mouse_position(x, y)?;

    let click_cmd = match button {
        MouseButton::Left => "click",
        MouseButton::Right => "click at {x, y}",
        MouseButton::Middle => "click at {x, y}",
    };

    let script = format!(r#"tell application "System Events" to {}"#, click_cmd);
    Command::new("osascript").args(["-e", &script]).output()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn mouse_click(button: MouseButton, x: i32, y: i32) -> Result<()> {
    let _ = (button, x, y);
    anyhow::bail!("Mouse click not implemented on this platform")
}

/// Send mouse double click
pub fn mouse_double_click(button: MouseButton, x: i32, y: i32) -> Result<()> {
    mouse_click(button.clone(), x, y)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    mouse_click(button, x, y)?;
    Ok(())
}

/// Send mouse press (down only)
#[cfg(target_os = "windows")]
pub fn mouse_press(button: MouseButton, x: i32, y: i32) -> Result<()> {
    set_mouse_position(x, y)?;
    let down_flag = match button {
        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
    };
    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe { std::mem::zeroed() },
    };
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
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn mouse_press(button: MouseButton, x: i32, y: i32) -> Result<()> {
    set_mouse_position(x, y)?;

    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Middle => "2",
        MouseButton::Right => "3",
    };

    Command::new("xdotool").args(["mousedown", btn]).output()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn mouse_press(button: MouseButton, x: i32, y: i32) -> Result<()> {
    let _ = (button, x, y);
    anyhow::bail!("Mouse press not implemented on this platform")
}

/// Send mouse release (up only)
#[cfg(target_os = "windows")]
pub fn mouse_release(button: MouseButton, x: i32, y: i32) -> Result<()> {
    set_mouse_position(x, y)?;

    let up_flag = match button {
        MouseButton::Left => MOUSEEVENTF_LEFTUP,
        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
    };

    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe { std::mem::zeroed() },
    };

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

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn mouse_release(button: MouseButton, x: i32, y: i32) -> Result<()> {
    set_mouse_position(x, y)?;

    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Middle => "2",
        MouseButton::Right => "3",
    };

    Command::new("xdotool").args(["mouseup", btn]).output()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn mouse_release(button: MouseButton, x: i32, y: i32) -> Result<()> {
    let _ = (button, x, y);
    anyhow::bail!("Mouse release not implemented on this platform")
}

/// Send mouse scroll
#[cfg(target_os = "windows")]
pub fn mouse_scroll(delta: i32) -> Result<()> {
    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe { std::mem::zeroed() },
    };

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

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn mouse_scroll(delta: i32) -> Result<()> {
    let direction = if delta > 0 { "up" } else { "down" };
    let clicks = (delta.abs() / 120).max(1);

    for _ in 0..clicks {
        Command::new("xdotool")
            .args(["click", if direction == "up" { "4" } else { "5" }])
            .output()?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn mouse_scroll(delta: i32) -> Result<()> {
    let script = format!(
        r#"tell application "System Events" to scroll wheel {}"#,
        if delta > 0 { "up" } else { "down" }
    );
    Command::new("osascript").args(["-e", &script]).output()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn mouse_scroll(delta: i32) -> Result<()> {
    let _ = delta;
    anyhow::bail!("Mouse scroll not implemented on this platform")
}

/// Smooth move to target with acceleration
pub async fn smooth_move_to(target_x: i32, target_y: i32, duration_ms: u64) -> Result<()> {
    let start = get_mouse_position()?;
    let start_x = start.x;
    let start_y = start.y;

    let steps = 20;
    let step_delay = duration_ms / steps as u64;

    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        // Ease out cubic
        let ease = 1.0 - (1.0 - t).powi(3);

        let x = start_x + ((target_x - start_x) as f64 * ease) as i32;
        let y = start_y + ((target_y - start_y) as f64 * ease) as i32;

        set_mouse_position(x, y)?;
        tokio::time::sleep(std::time::Duration::from_millis(step_delay)).await;
    }

    set_mouse_position(target_x, target_y)?;
    Ok(())
}

/// Get cursor type
#[cfg(target_os = "windows")]
pub fn get_cursor_type() -> Result<String> {
    Ok("arrow".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn get_cursor_type() -> Result<String> {
    Ok("unknown".to_string())
}
