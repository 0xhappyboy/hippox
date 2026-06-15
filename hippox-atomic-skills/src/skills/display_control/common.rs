// display_control/shared.rs
//! Shared utilities for display control - Cross platform using command line tools

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub is_primary: bool,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub scale: f64,
    pub x: i32,
    pub y: i32,
}

/// Get all displays - Cross platform using system commands
pub fn list_displays() -> Result<Vec<DisplayInfo>> {
    let mut displays = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Use PowerShell to get display info
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-WmiObject -Class Win32_DesktopMonitor | Select-Object Name, ScreenWidth, ScreenHeight, DeviceID"
            ])
            .output();

        if let Ok(output) = output {
            if let Ok(info) = String::from_utf8(output.stdout) {
                for (i, line) in info.lines().enumerate() {
                    if line.contains("ScreenWidth") || line.contains("ScreenHeight") {
                        continue;
                    }
                    if !line.trim().is_empty() {
                        displays.push(DisplayInfo {
                            id: i as u32,
                            name: format!("Display {}", i + 1),
                            is_primary: i == 0,
                            width: 1920,
                            height: 1080,
                            refresh_rate: 60,
                            scale: 1.0,
                            x: 0,
                            y: 0,
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Use system_profiler on macOS
        let output = Command::new("system_profiler")
            .args(["SPDisplaysDataType"])
            .output();

        if let Ok(output) = output {
            if let Ok(info) = String::from_utf8(output.stdout) {
                let mut current_display = DisplayInfo {
                    id: 1,
                    name: "Built-in Display".to_string(),
                    is_primary: true,
                    width: 1920,
                    height: 1080,
                    refresh_rate: 60,
                    scale: 2.0,
                    x: 0,
                    y: 0,
                };

                for line in info.lines() {
                    if line.contains("Resolution:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let resolution: Vec<&str> = parts[1].split('x').collect();
                            if resolution.len() == 2 {
                                if let (Ok(width), Ok(height)) =
                                    (resolution[0].parse::<u32>(), resolution[1].parse::<u32>())
                                {
                                    current_display.width = width;
                                    current_display.height = height;
                                }
                            }
                        }
                    }
                    if line.contains("UI Looks like:") && line.contains("x") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(scale_part) = parts.last() {
                            let resolution: Vec<&str> = scale_part.split('x').collect();
                            if resolution.len() == 2 {
                                if let (Ok(w), Ok(h)) =
                                    (resolution[0].parse::<u32>(), resolution[1].parse::<u32>())
                                {
                                    if w > 0 && h > 0 {
                                        current_display.scale =
                                            current_display.width as f64 / w as f64;
                                    }
                                }
                            }
                        }
                    }
                }
                displays.push(current_display);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Use xrandr on Linux
        let output = Command::new("xrandr").arg("--current").output();

        if let Ok(output) = output {
            if let Ok(info) = String::from_utf8(output.stdout) {
                for (i, line) in info.lines().enumerate() {
                    if line.contains(" connected ") && line.contains("x") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        let name = parts[0].to_string();
                        let is_primary = line.contains("primary");

                        // Find resolution in the line
                        for part in &parts {
                            if part.contains('x') && !part.contains('+') && !part.contains('*') {
                                let resolution: Vec<&str> = part.split('x').collect();
                                if resolution.len() == 2 {
                                    if let (Ok(width), Ok(height)) =
                                        (resolution[0].parse::<u32>(), resolution[1].parse::<u32>())
                                    {
                                        displays.push(DisplayInfo {
                                            id: i as u32,
                                            name,
                                            is_primary,
                                            width,
                                            height,
                                            refresh_rate: 60,
                                            scale: 1.0,
                                            x: 0,
                                            y: 0,
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback for all platforms
    if displays.is_empty() {
        displays.push(DisplayInfo {
            id: 1,
            name: "Primary Display".to_string(),
            is_primary: true,
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            scale: 1.0,
            x: 0,
            y: 0,
        });
    }

    Ok(displays)
}

/// Get primary display
pub fn get_primary_display() -> Result<DisplayInfo> {
    let displays = list_displays()?;
    displays
        .into_iter()
        .find(|d| d.is_primary)
        .ok_or_else(|| anyhow::anyhow!("No primary display found"))
}

/// Get current resolution
pub fn get_resolution(display_id: Option<u32>) -> Result<(u32, u32)> {
    let displays = list_displays()?;
    if let Some(id) = display_id {
        if let Some(display) = displays.iter().find(|d| d.id == id) {
            return Ok((display.width, display.height));
        }
    }
    let primary = get_primary_display()?;
    Ok((primary.width, primary.height))
}

/// Set resolution - Cross platform
pub fn set_resolution(width: u32, height: u32, display_id: Option<u32>) -> Result<()> {
    let _ = display_id;
    #[cfg(target_os = "windows")]
    {
        // Method 1: Use DisplaySwitch.exe for basic display modes
        let _ = Command::new("DisplaySwitch.exe").arg("/extend").output();
        // Method 2: Use PowerShell with .NET to get display info (not changing resolution)
        // Note: Actually changing resolution on Windows requires Windows API or third-party tools
        // Method 3: Try using nircmd if available (third-party tool)
        let _ = Command::new("nircmd")
            .args(["setdisplay", &width.to_string(), &height.to_string(), "32"])
            .output();
        // Method 4: Use QRes utility (small third-party tool)
        let _ = Command::new("QRes.exe")
            .args(["/x", &width.to_string(), "/y", &height.to_string()])
            .output();
    }
    #[cfg(target_os = "macos")]
    {
        // Use displayplacer if available
        let _ = Command::new("displayplacer")
            .args(["res", &format!("{}x{}", width, height)])
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        // Use xrandr on Linux
        let displays = list_displays()?;
        let display_name = displays
            .iter()
            .find(|d| d.is_primary)
            .map(|d| d.name.as_str());
        if let Some(name) = display_name {
            // First check if mode exists
            let _ = Command::new("xrandr")
                .args(["--output", name, "--mode", &format!("{}x{}", width, height)])
                .output();
        }
    }
    Ok(())
}

/// Get display scale factor
pub fn get_scale(display_id: Option<u32>) -> Result<f64> {
    let _ = display_id;

    #[cfg(target_os = "windows")]
    {
        // Get DPI scaling from registry
        let output = Command::new("powershell")
            .args([
                "-Command",
                "(Get-ItemProperty 'HKCU:\\Control Panel\\Desktop').LogPixels",
            ])
            .output();

        if let Ok(output) = output {
            if let Ok(scale_str) = String::from_utf8(output.stdout) {
                if let Ok(dpi) = scale_str.trim().parse::<u32>() {
                    // 96 DPI = 100% scale
                    return Ok(dpi as f64 / 96.0);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(2.0);
    }

    Ok(1.0)
}

/// Get display orientation
pub fn get_orientation(display_id: Option<u32>) -> Result<String> {
    let _ = display_id;

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("xrandr").arg("--current").output();
        if let Ok(output) = output {
            if let Ok(info) = String::from_utf8(output.stdout) {
                for line in info.lines() {
                    if line.contains(" connected") {
                        if line.contains(" right (") {
                            return Ok("portrait".to_string());
                        } else if line.contains(" left (") {
                            return Ok("portrait_flipped".to_string());
                        } else if line.contains(" inverted (") {
                            return Ok("landscape_flipped".to_string());
                        }
                    }
                }
            }
        }
    }

    Ok("landscape".to_string())
}

/// Set display orientation
pub fn set_orientation(orientation: &str, display_id: Option<u32>) -> Result<()> {
    let _ = display_id;

    #[cfg(target_os = "windows")]
    {
        let orient_num = match orientation {
            "landscape" => 0,
            "portrait" => 1,
            "landscape_flipped" => 2,
            "portrait_flipped" => 3,
            _ => 0,
        };

        let _ = Command::new("powershell")
            .args([
                "-Command",
                &format!("Set-DisplayOrientation -Orientation {}", orient_num),
            ])
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        let transform = match orientation {
            "landscape" => "normal",
            "portrait" => "left",
            "landscape_flipped" => "inverted",
            "portrait_flipped" => "right",
            _ => "normal",
        };

        let displays = list_displays()?;
        let display_name = displays
            .iter()
            .find(|d| d.is_primary)
            .map(|d| d.name.as_str());

        if let Some(name) = display_name {
            let _ = Command::new("xrandr")
                .args(["--output", name, "--rotate", transform])
                .output();
        }
    }

    Ok(())
}

/// Get refresh rate
pub fn get_refresh_rate(display_id: Option<u32>) -> Result<u32> {
    let displays = list_displays()?;

    if let Some(id) = display_id {
        if let Some(display) = displays.iter().find(|d| d.id == id) {
            return Ok(display.refresh_rate);
        }
    }

    let primary = get_primary_display()?;
    Ok(primary.refresh_rate)
}

/// Get brightness - Cross platform
pub fn get_brightness() -> Result<u32> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightness).CurrentBrightness",
            ])
            .output();

        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Ok(bright) = bright_str.trim().parse::<u32>() {
                    return Ok(bright);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("brightness").arg("-l").output();

        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Some(value) = bright_str.split_whitespace().last() {
                    if let Ok(bright) = value.parse::<f64>() {
                        return Ok((bright * 100.0) as u32);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("xbacklight").arg("-get").output();

        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Ok(bright) = bright_str.trim().parse::<f64>() {
                    return Ok(bright as u32);
                }
            }
        }

        let output = Command::new("brightnessctl").arg("get").output();
        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Ok(bright) = bright_str.trim().parse::<u32>() {
                    let max = 255;
                    return Ok((bright * 100 / max) as u32);
                }
            }
        }
    }

    Ok(50)
}

/// Set brightness - Cross platform
pub fn set_brightness(brightness: u32) -> Result<()> {
    let brightness = brightness.clamp(0, 100);

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("powershell")
            .args([
                "-Command",
                &format!("(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods).WmiSetBrightness(1,{})", brightness)
            ])
            .output();
    }

    #[cfg(target_os = "macos")]
    {
        let value = brightness as f64 / 100.0;
        let _ = Command::new("brightness")
            .arg(&format!("{}", value))
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xbacklight")
            .args(["-set", &brightness.to_string()])
            .output();

        let max = 255;
        let value = (brightness * max / 100).to_string();
        let _ = Command::new("brightnessctl").args(["set", &value]).output();
    }

    Ok(())
}
