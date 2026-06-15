//! Shared utilities for window control across platforms

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ffi::c_void;

/// Window information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub process_name: String,
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_visible: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
}

/// Rectangle structure
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// Find window by title or process name
pub fn find_window(title: Option<&str>, process: Option<&str>) -> Result<u64> {
    let windows = list_windows()?;

    if let Some(title_match) = title {
        let title_lower = title_match.to_lowercase();
        if let Some(w) = windows
            .iter()
            .find(|w| w.title.to_lowercase().contains(&title_lower))
        {
            return Ok(w.id);
        }
    }

    if let Some(process_match) = process {
        let process_lower = process_match.to_lowercase();
        if let Some(w) = windows
            .iter()
            .find(|w| w.process_name.to_lowercase().contains(&process_lower))
        {
            return Ok(w.id);
        }
    }

    anyhow::bail!("Window not found: title={:?}, process={:?}", title, process)
}

// Helper function to convert u64 to HWND (Windows only)
#[cfg(target_os = "windows")]
pub fn u64_to_hwnd(id: u64) -> windows::Win32::Foundation::HWND {
    windows::Win32::Foundation::HWND(id as *mut c_void)
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::Win32::Foundation::{BOOL, CloseHandle, HWND, LPARAM, RECT};
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;
    pub fn list_windows() -> Result<Vec<WindowInfo>> {
        let mut windows: Vec<WindowInfo> = Vec::new();
        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let windows_ptr = lparam.0 as *mut Vec<WindowInfo>;
            let windows = &mut *windows_ptr;
            if IsWindowVisible(hwnd).as_bool() {
                let mut title_buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut title_buf);
                let title = String::from_utf16_lossy(&title_buf[..len as usize]);
                let mut pid: u32 = 0;
                let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
                let mut rect: RECT = std::mem::zeroed();
                let _ = GetWindowRect(hwnd, &mut rect);
                windows.push(WindowInfo {
                    id: hwnd.0 as u64,
                    title,
                    process_name: get_process_name(pid),
                    pid,
                    x: rect.left,
                    y: rect.top,
                    width: (rect.right - rect.left) as u32,
                    height: (rect.bottom - rect.top) as u32,
                    is_visible: true,
                    is_minimized: IsIconic(hwnd).as_bool(),
                    is_maximized: IsZoomed(hwnd).as_bool(),
                });
            }
            BOOL::from(true)
        }
        let mut windows_vec = windows;
        let lparam = LPARAM(&mut windows_vec as *mut Vec<WindowInfo> as isize);
        unsafe {
            let _ = EnumWindows(Some(enum_callback), lparam);
        }
        Ok(windows_vec)
    }

    fn get_process_name(pid: u32) -> String {
        unsafe {
            let handle_result =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
            match handle_result {
                Ok(handle) => {
                    let mut name_buf = [0u16; 260];
                    let len = GetModuleBaseNameW(handle, None, &mut name_buf);
                    let _ = CloseHandle(handle);
                    String::from_utf16_lossy(&name_buf[..len as usize])
                }
                Err(_) => format!("[{}]", pid),
            }
        }
    }

    pub fn get_window_rect(window_id: u64) -> Result<Rect> {
        unsafe {
            let mut rect: RECT = std::mem::zeroed();
            let hwnd = u64_to_hwnd(window_id);
            let _ = GetWindowRect(hwnd, &mut rect);
            Ok(Rect {
                x: rect.left,
                y: rect.top,
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            })
        }
    }

    pub fn set_window_pos(window_id: u64, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
        unsafe {
            let hwnd = u64_to_hwnd(window_id);
            let _ = SetWindowPos(hwnd, None, x, y, width as i32, height as i32, SWP_NOZORDER);
        }
        Ok(())
    }

    pub fn show_window(window_id: u64, cmd: SHOW_WINDOW_CMD) -> Result<()> {
        unsafe {
            let hwnd = u64_to_hwnd(window_id);
            let _ = ShowWindow(hwnd, cmd);
        }
        Ok(())
    }

    pub fn close_window(window_id: u64) -> Result<()> {
        unsafe {
            let hwnd = u64_to_hwnd(window_id);
            let _ = PostMessageW(hwnd, WM_CLOSE, None, None);
        }
        Ok(())
    }

    pub fn kill_window(window_id: u64) -> Result<()> {
        unsafe {
            let hwnd = u64_to_hwnd(window_id);
            let mut pid: u32 = 0;
            let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));

            use windows::Win32::System::Threading::{
                OpenProcess, PROCESS_TERMINATE, TerminateProcess,
            };
            if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                let _ = TerminateProcess(handle, 1);
                let _ = CloseHandle(handle);
            }
        }
        Ok(())
    }

    pub fn set_foreground_window(window_id: u64) -> Result<()> {
        unsafe {
            let hwnd = u64_to_hwnd(window_id);
            let _ = SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    pub fn get_focus_window() -> Result<u64> {
        unsafe { Ok(GetForegroundWindow().0 as u64) }
    }
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use core_graphics::window::{
        CGWindow, CGWindowID, CGWindowListCopyWindowInfo, CGWindowListOption,
    };
    use std::collections::HashMap;

    pub fn list_windows() -> Result<Vec<WindowInfo>> {
        let mut windows = Vec::new();
        let window_info = unsafe {
            CGWindowListCopyWindowInfo(CGWindowListOption::kCGWindowListOptionAll, 0)
        };
        if let Some(info_array) = window_info {
            for window in info_array.iter() {
                let dict = window;
                let window_id: Option<u64> = dict.get("kCGWindowNumber").and_then(|v| v.as_u64());
                let title: String = dict
                    .get("kCGWindowName")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                let pid: u32 = dict
                    .get("kCGWindowOwnerPID")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let process_name: String = dict
                    .get("kCGWindowOwnerName")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                let bounds = dict.get("kCGWindowBounds").and_then(|v| v.as_dictionary());
                let x: i32 = bounds
                    .and_then(|b| b.get("X"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as i32;
                let y: i32 = bounds
                    .and_then(|b| b.get("Y"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as i32;
                let width: u32 = bounds
                    .and_then(|b| b.get("Width"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as u32;
                let height: u32 = bounds
                    .and_then(|b| b.get("Height"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as u32;
                let is_visible: bool = dict
                    .get("kCGWindowIsOnscreen")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) == 1;
                if let Some(id) = window_id {
                    windows.push(WindowInfo {
                        id,
                        title,
                        process_name,
                        pid,
                        x,
                        y,
                        width,
                        height,
                        is_visible,
                        is_minimized: false, // macOS: check via other API
                        is_maximized: false,
                    });
                }
            }
        }
        Ok(windows)
    }

    pub fn get_window_rect(window_id: u64) -> Result<Rect> {
        let windows = list_windows()?;
        let window = windows.iter().find(|w| w.id == window_id)
            .ok_or_else(|| anyhow::anyhow!("Window {} not found", window_id))?;
        Ok(Rect {
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
        })
    }

    pub fn set_window_pos(_window_id: u64, _x: i32, _y: i32, _width: u32, _height: u32) -> Result<()> {
        // TODO: Implement using AXUIElement
        anyhow::bail!("Window resize/move not implemented on macOS")
    }

    pub fn show_window(_window_id: u64, _cmd: u32) -> Result<()> {
        // TODO: Implement using AXUIElement
        anyhow::bail!("Show window not implemented on macOS")
    }

    pub fn close_window(_window_id: u64) -> Result<()> {
        // TODO: Implement using AXUIElement
        anyhow::bail!("Close window not implemented on macOS")
    }

    pub fn kill_window(window_id: u64) -> Result<()> {
        let windows = list_windows()?;
        let window = windows.iter().find(|w| w.id == window_id)
            .ok_or_else(|| anyhow::anyhow!("Window {} not found", window_id))?;
        
        use std::process::Command;
        Command::new("kill")
            .arg("-9")
            .arg(window.pid.to_string())
            .status()?;
        Ok(())
    }

    pub fn set_foreground_window(_window_id: u64) -> Result<()> {
        // TODO: Implement using AXUIElement
        anyhow::bail!("Set foreground window not implemented on macOS")
    }

    pub fn get_focus_window() -> Result<u64> {
        // Get frontmost window using NSWorkspace
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send, sel};
        
        let workspace = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
        let front_app: *mut AnyObject = unsafe { msg_send![workspace, frontmostApplication] };
        let pid: i32 = unsafe { msg_send![front_app, processIdentifier] };
        
        let windows = list_windows()?;
        if let Some(w) = windows.iter().find(|w| w.pid == pid as u32) {
            Ok(w.id)
        } else {
            Err(anyhow::anyhow!("No focused window found"))
        }
    }
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;
    use std::process::Command;

    pub fn list_windows() -> Result<Vec<WindowInfo>> {
        // Use xdotool to list windows
        let output = Command::new("xdotool")
            .args(["search", "--name", ".*"])
            .output()?;
        
        let window_ids_str = String::from_utf8_lossy(&output.stdout);
        let window_ids: Vec<&str> = window_ids_str.trim().split('\n').collect();
        
        let mut windows = Vec::new();
        
        for (i, id_str) in window_ids.iter().enumerate() {
            if id_str.is_empty() {
                continue;
            }
            
            let id = id_str.parse::<u64>().unwrap_or(0);
            
            // Get window title
            let title_output = Command::new("xdotool")
                .args(["getwindowname", id_str])
                .output()
                .ok();
            let title = title_output
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string();
            
            // Get window geometry
            let geom_output = Command::new("xdotool")
                .args(["getwindowgeometry", id_str])
                .output()
                .ok();
            
            let mut x = 0;
            let mut y = 0;
            let mut width: u32 = 0;
            let mut height: u32 = 0;
            
            if let Some(output) = geom_output {
                let geom = String::from_utf8_lossy(&output.stdout);
                for line in geom.lines() {
                    if line.starts_with("Position:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let pos: Vec<&str> = parts[1].split(',').collect();
                            if pos.len() == 2 {
                                x = pos[0].parse().unwrap_or(0);
                                y = pos[1].parse().unwrap_or(0);
                            }
                        }
                    } else if line.starts_with("Geometry:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let geom: Vec<&str> = parts[1].split('x').collect();
                            if geom.len() == 2 {
                                width = geom[0].parse().unwrap_or(0);
                                height = geom[1].parse().unwrap_or(0);
                            }
                        }
                    }
                }
            }
            
            windows.push(WindowInfo {
                id,
                title,
                process_name: format!("window_{}", i),
                pid: 0,
                x,
                y,
                width,
                height,
                is_visible: true,
                is_minimized: false,
                is_maximized: false,
            });
        }
        
        Ok(windows)
    }

    pub fn get_window_rect(window_id: u64) -> Result<Rect> {
        let output = Command::new("xdotool")
            .args(["getwindowgeometry", &window_id.to_string()])
            .output()?;
        
        let geom = String::from_utf8_lossy(&output.stdout);
        let mut x = 0;
        let mut y = 0;
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        
        for line in geom.lines() {
            if line.starts_with("Position:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let pos: Vec<&str> = parts[1].split(',').collect();
                    if pos.len() == 2 {
                        x = pos[0].parse().unwrap_or(0);
                        y = pos[1].parse().unwrap_or(0);
                    }
                }
            } else if line.starts_with("Geometry:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let geom: Vec<&str> = parts[1].split('x').collect();
                    if geom.len() == 2 {
                        width = geom[0].parse().unwrap_or(0);
                        height = geom[1].parse().unwrap_or(0);
                    }
                }
            }
        }
        
        Ok(Rect { x, y, width, height })
    }

    pub fn set_window_pos(window_id: u64, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
        Command::new("xdotool")
            .args(["windowmove", &window_id.to_string(), &x.to_string(), &y.to_string()])
            .status()?;
        Command::new("xdotool")
            .args(["windowsize", &window_id.to_string(), &width.to_string(), &height.to_string()])
            .status()?;
        Ok(())
    }

    pub fn show_window(window_id: u64, cmd: u32) -> Result<()> {
        match cmd {
            3 | 9 => { // Maximize or Restore
                Command::new("xdotool")
                    .args(["windowactivate", &window_id.to_string()])
                    .status()?;
                // Try to maximize
                let _ = Command::new("xdotool")
                    .args(["windowsize", &window_id.to_string(), "100%", "100%"])
                    .status();
            }
            6 => { // Minimize
                Command::new("xdotool")
                    .args(["windowminimize", &window_id.to_string()])
                    .status()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn close_window(window_id: u64) -> Result<()> {
        Command::new("xdotool")
            .args(["windowclose", &window_id.to_string()])
            .status()?;
        Ok(())
    }

    pub fn kill_window(window_id: u64) -> Result<()> {
        Command::new("xdotool")
            .args(["windowkill", &window_id.to_string()])
            .status()?;
        Ok(())
    }

    pub fn set_foreground_window(window_id: u64) -> Result<()> {
        Command::new("xdotool")
            .args(["windowactivate", &window_id.to_string()])
            .status()?;
        Ok(())
    }

    pub fn get_focus_window() -> Result<u64> {
        let output = Command::new("xdotool")
            .args(["getactivewindow"])
            .output()?;
        let id_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(id_str.parse::<u64>().unwrap_or(0))
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::*;
#[cfg(target_os = "macos")]
pub use macos_impl::*;
#[cfg(target_os = "linux")]
pub use linux_impl::*;

pub fn list_windows() -> Result<Vec<WindowInfo>> {
    #[cfg(target_os = "windows")]
    return windows_impl::list_windows();
    #[cfg(target_os = "macos")]
    return macos_impl::list_windows();
    #[cfg(target_os = "linux")]
    return linux_impl::list_windows();
}

pub fn get_window_rect(window_id: u64) -> Result<Rect> {
    #[cfg(target_os = "windows")]
    return windows_impl::get_window_rect(window_id);
    #[cfg(target_os = "macos")]
    return macos_impl::get_window_rect(window_id);
    #[cfg(target_os = "linux")]
    return linux_impl::get_window_rect(window_id);
}

pub fn set_window_pos(window_id: u64, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows_impl::set_window_pos(window_id, x, y, width, height);
    #[cfg(target_os = "macos")]
    return macos_impl::set_window_pos(window_id, x, y, width, height);
    #[cfg(target_os = "linux")]
    return linux_impl::set_window_pos(window_id, x, y, width, height);
}

pub fn show_window(window_id: u64, cmd: u32) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;
        let cmd_i32 = cmd.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid SHOW_WINDOW_CMD value: {}", cmd))?;
        return windows_impl::show_window(window_id, SHOW_WINDOW_CMD(cmd_i32));
    }
    #[cfg(target_os = "macos")]
    return macos_impl::show_window(window_id, cmd);
    #[cfg(target_os = "linux")]
    return linux_impl::show_window(window_id, cmd);
}

pub fn close_window(window_id: u64) -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows_impl::close_window(window_id);
    #[cfg(target_os = "macos")]
    return macos_impl::close_window(window_id);
    #[cfg(target_os = "linux")]
    return linux_impl::close_window(window_id);
}

pub fn kill_window(window_id: u64) -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows_impl::kill_window(window_id);
    #[cfg(target_os = "macos")]
    return macos_impl::kill_window(window_id);
    #[cfg(target_os = "linux")]
    return linux_impl::kill_window(window_id);
}

pub fn set_foreground_window(window_id: u64) -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows_impl::set_foreground_window(window_id);
    #[cfg(target_os = "macos")]
    return macos_impl::set_foreground_window(window_id);
    #[cfg(target_os = "linux")]
    return linux_impl::set_foreground_window(window_id);
}

pub fn get_focus_window() -> Result<u64> {
    #[cfg(target_os = "windows")]
    return windows_impl::get_focus_window();
    #[cfg(target_os = "macos")]
    return macos_impl::get_focus_window();
    #[cfg(target_os = "linux")]
    return linux_impl::get_focus_window();
}