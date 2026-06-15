// keyboard_control/shared.rs
//! Shared utilities for keyboard control across platforms

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{KEYEVENTF_KEYUP, keybd_event};

/// Keyboard key representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCode {
    pub name: String,
    pub virtual_key: u16,
}

/// Common key mappings for Windows using virtual key codes
#[cfg(target_os = "windows")]
pub fn get_key_code(key_name: &str) -> Option<u16> {
    match key_name.to_lowercase().as_str() {
        // Letters
        "a" => Some(0x41),
        "b" => Some(0x42),
        "c" => Some(0x43),
        "d" => Some(0x44),
        "e" => Some(0x45),
        "f" => Some(0x46),
        "g" => Some(0x47),
        "h" => Some(0x48),
        "i" => Some(0x49),
        "j" => Some(0x4A),
        "k" => Some(0x4B),
        "l" => Some(0x4C),
        "m" => Some(0x4D),
        "n" => Some(0x4E),
        "o" => Some(0x4F),
        "p" => Some(0x50),
        "q" => Some(0x51),
        "r" => Some(0x52),
        "s" => Some(0x53),
        "t" => Some(0x54),
        "u" => Some(0x55),
        "v" => Some(0x56),
        "w" => Some(0x57),
        "x" => Some(0x58),
        "y" => Some(0x59),
        "z" => Some(0x5A),
        // Numbers (top row)
        "0" => Some(0x30),
        "1" => Some(0x31),
        "2" => Some(0x32),
        "3" => Some(0x33),
        "4" => Some(0x34),
        "5" => Some(0x35),
        "6" => Some(0x36),
        "7" => Some(0x37),
        "8" => Some(0x38),
        "9" => Some(0x39),
        // Numpad
        "numpad0" => Some(0x60),
        "numpad1" => Some(0x61),
        "numpad2" => Some(0x62),
        "numpad3" => Some(0x63),
        "numpad4" => Some(0x64),
        "numpad5" => Some(0x65),
        "numpad6" => Some(0x66),
        "numpad7" => Some(0x67),
        "numpad8" => Some(0x68),
        "numpad9" => Some(0x69),
        // Modifiers
        "ctrl" | "control" => Some(0x11),
        "alt" => Some(0x12),
        "shift" => Some(0x10),
        "win" | "windows" => Some(0x5B),
        // Function keys
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        "f3" => Some(0x72),
        "f4" => Some(0x73),
        "f5" => Some(0x74),
        "f6" => Some(0x75),
        "f7" => Some(0x76),
        "f8" => Some(0x77),
        "f9" => Some(0x78),
        "f10" => Some(0x79),
        "f11" => Some(0x7A),
        "f12" => Some(0x7B),
        // Navigation
        "enter" | "return" => Some(0x0D),
        "space" => Some(0x20),
        "tab" => Some(0x09),
        "esc" | "escape" => Some(0x1B),
        "backspace" => Some(0x08),
        "delete" => Some(0x2E),
        "insert" => Some(0x2D),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "pageup" => Some(0x21),
        "pagedown" => Some(0x22),
        // Arrows
        "up" => Some(0x26),
        "down" => Some(0x28),
        "left" => Some(0x25),
        "right" => Some(0x27),
        _ => None,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_key_code(key_name: &str) -> Option<u16> {
    // Linux key codes (X11)
    match key_name.to_lowercase().as_str() {
        "a" => Some(38),
        "b" => Some(56),
        "c" => Some(54),
        "d" => Some(40),
        "e" => Some(26),
        "f" => Some(41),
        "g" => Some(42),
        "h" => Some(43),
        "i" => Some(31),
        "j" => Some(44),
        "k" => Some(45),
        "l" => Some(46),
        "m" => Some(58),
        "n" => Some(57),
        "o" => Some(32),
        "p" => Some(33),
        "q" => Some(24),
        "r" => Some(27),
        "s" => Some(39),
        "t" => Some(28),
        "u" => Some(30),
        "v" => Some(55),
        "w" => Some(25),
        "x" => Some(53),
        "y" => Some(29),
        "z" => Some(52),
        "enter" | "return" => Some(36),
        "space" => Some(65),
        "tab" => Some(23),
        "esc" | "escape" => Some(9),
        "ctrl" | "control" => Some(37),
        "alt" => Some(64),
        "shift" => Some(50),
        "up" => Some(111),
        "down" => Some(116),
        "left" => Some(113),
        "right" => Some(114),
        _ => None,
    }
}

/// Send a key press using Windows keybd_event (winapi)
#[cfg(target_os = "windows")]
pub fn send_key_press(key_code: u16) -> Result<()> {
    unsafe {
        keybd_event(key_code as u8, 0, 0, 0);
        keybd_event(key_code as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    Ok(())
}

/// Send key down event
#[cfg(target_os = "windows")]
pub fn send_key_down(key_code: u16) -> Result<()> {
    unsafe {
        keybd_event(key_code as u8, 0, 0, 0);
    }
    Ok(())
}

/// Send key up event
#[cfg(target_os = "windows")]
pub fn send_key_up(key_code: u16) -> Result<()> {
    unsafe {
        keybd_event(key_code as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    Ok(())
}

/// Linux implementation using xdotool
#[cfg(target_os = "linux")]
pub fn send_key_press(key_code: u16) -> Result<()> {
    let key_name = match key_code {
        38 => "a",
        56 => "b",
        54 => "c",
        40 => "d",
        26 => "e",
        41 => "f",
        42 => "g",
        43 => "h",
        31 => "i",
        44 => "j",
        45 => "k",
        46 => "l",
        58 => "m",
        57 => "n",
        32 => "o",
        33 => "p",
        24 => "q",
        27 => "r",
        39 => "s",
        28 => "t",
        30 => "u",
        55 => "v",
        25 => "w",
        53 => "x",
        29 => "y",
        52 => "z",
        36 => "Return",
        65 => "space",
        23 => "Tab",
        9 => "Escape",
        _ => return Ok(()),
    };

    Command::new("xdotool").args(["key", key_name]).output()?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn send_key_down(key_code: u16) -> Result<()> {
    let key_name = match key_code {
        37 => "ctrl",
        64 => "alt",
        50 => "shift",
        _ => return Ok(()),
    };
    Command::new("xdotool")
        .args(["keydown", key_name])
        .output()?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn send_key_up(key_code: u16) -> Result<()> {
    let key_name = match key_code {
        37 => "ctrl",
        64 => "alt",
        50 => "shift",
        _ => return Ok(()),
    };
    Command::new("xdotool").args(["keyup", key_name]).output()?;
    Ok(())
}

/// macOS implementation using osascript
#[cfg(target_os = "macos")]
pub fn send_key_press(key_code: u16) -> Result<()> {
    let key_name = match key_code {
        0x41 => "a",
        0x42 => "b",
        0x43 => "c",
        0x44 => "d",
        0x45 => "e",
        0x46 => "f",
        0x47 => "g",
        0x48 => "h",
        0x49 => "i",
        0x4A => "j",
        0x4B => "k",
        0x4C => "l",
        0x4D => "m",
        0x4E => "n",
        0x4F => "o",
        0x50 => "p",
        0x51 => "q",
        0x52 => "r",
        0x53 => "s",
        0x54 => "t",
        0x55 => "u",
        0x56 => "v",
        0x57 => "w",
        0x58 => "x",
        0x59 => "y",
        0x5A => "z",
        0x0D => "return",
        0x20 => "space",
        0x09 => "tab",
        0x1B => "escape",
        _ => return Ok(()),
    };

    let script = format!(
        r#"tell application "System Events" to keystroke "{}""#,
        key_name
    );
    Command::new("osascript").args(["-e", &script]).output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn send_key_down(key_code: u16) -> Result<()> {
    let _ = key_code;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn send_key_up(key_code: u16) -> Result<()> {
    let _ = key_code;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn send_key_press(key_code: u16) -> Result<()> {
    let _ = key_code;
    anyhow::bail!("Key press not implemented on this platform")
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn send_key_down(key_code: u16) -> Result<()> {
    let _ = key_code;
    anyhow::bail!("Key down not implemented on this platform")
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn send_key_up(key_code: u16) -> Result<()> {
    let _ = key_code;
    anyhow::bail!("Key up not implemented on this platform")
}

/// Parse shortcut string like "Ctrl+C" or "Ctrl+Shift+S"
pub fn parse_shortcut(shortcut: &str) -> Result<Vec<(String, bool)>> {
    let parts: Vec<&str> = shortcut.split('+').collect();
    let mut keys = Vec::new();

    for part in parts {
        let key = part.to_lowercase();
        let is_modifier = matches!(
            key.as_str(),
            "ctrl" | "control" | "alt" | "shift" | "win" | "windows" | "cmd" | "command"
        );
        keys.push((key, is_modifier));
    }

    Ok(keys)
}

/// Send modifier key state
#[cfg(target_os = "windows")]
pub fn set_modifier_state(modifier: &str, down: bool) -> Result<()> {
    let key_code = match modifier.to_lowercase().as_str() {
        "ctrl" | "control" => get_key_code("ctrl"),
        "alt" => get_key_code("alt"),
        "shift" => get_key_code("shift"),
        "win" | "windows" => get_key_code("win"),
        _ => None,
    };

    if let Some(code) = key_code {
        if down {
            send_key_down(code)?;
        } else {
            send_key_up(code)?;
        }
        Ok(())
    } else {
        anyhow::bail!("Unknown modifier: {}", modifier)
    }
}

#[cfg(target_os = "linux")]
pub fn set_modifier_state(modifier: &str, down: bool) -> Result<()> {
    let key_name = match modifier.to_lowercase().as_str() {
        "ctrl" | "control" => "ctrl",
        "alt" => "alt",
        "shift" => "shift",
        "win" | "windows" => "super",
        _ => "",
    };

    if !key_name.is_empty() {
        let cmd = if down { "keydown" } else { "keyup" };
        Command::new("xdotool").args([cmd, key_name]).output()?;
    }
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_modifier_state(modifier: &str, down: bool) -> Result<()> {
    let _ = (modifier, down);
    Ok(())
}

/// Send a shortcut (combination of keys)
pub fn send_shortcut(shortcut: &str) -> Result<()> {
    let keys = parse_shortcut(shortcut)?;

    // Press all modifiers in order
    for (key, is_modifier) in &keys {
        if *is_modifier {
            let _ = set_modifier_state(key, true);
        }
    }

    // Find and press the main key
    let main_key = keys.last().unwrap();
    if let Some(code) = get_key_code(&main_key.0) {
        send_key_press(code)?;
    }

    // Release modifiers in reverse order
    for (key, is_modifier) in keys.iter().rev() {
        if *is_modifier {
            let _ = set_modifier_state(key, false);
        }
    }

    Ok(())
}

/// Type text as keyboard input
#[cfg(target_os = "windows")]
pub fn type_text(text: &str) -> Result<()> {
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            let vk = get_key_code(&c.to_lowercase().to_string()).unwrap_or(0);
            let is_upper = c.is_ascii_uppercase();

            if is_upper {
                let shift = get_key_code("shift").unwrap();
                unsafe {
                    keybd_event(shift as u8, 0, 0, 0);
                    keybd_event(vk as u8, 0, 0, 0);
                    keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
                    keybd_event(shift as u8, 0, KEYEVENTF_KEYUP, 0);
                }
            } else {
                unsafe {
                    keybd_event(vk as u8, 0, 0, 0);
                    keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
                }
            }
        } else if c.is_ascii_digit() {
            if let Some(vk) = get_key_code(&c.to_string()) {
                unsafe {
                    keybd_event(vk as u8, 0, 0, 0);
                    keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
                }
            }
        } else if c == ' ' {
            if let Some(vk) = get_key_code("space") {
                unsafe {
                    keybd_event(vk as u8, 0, 0, 0);
                    keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
                }
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn type_text(text: &str) -> Result<()> {
    Command::new("xdotool")
        .args(["type", "--clearmodifiers", text])
        .output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn type_text(text: &str) -> Result<()> {
    let escaped = text.replace("\\", "\\\\").replace("\"", "\\\"");
    let script = format!(
        r#"tell application "System Events" to keystroke "{}""#,
        escaped
    );
    Command::new("osascript").args(["-e", &script]).output()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn type_text(text: &str) -> Result<()> {
    let _ = text;
    anyhow::bail!("Type text not implemented on this platform")
}
