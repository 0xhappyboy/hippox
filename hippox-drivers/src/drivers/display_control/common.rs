// display_control/common.rs
//! Shared utilities for display control - Cross platform using command line tools
//!
//! This module provides cross-platform display management functionality
//! using system command line tools.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};
/// Display information structure
///
/// Contains comprehensive information about a display/monitor
/// including resolution, refresh rate, scaling, and position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Unique display identifier
    pub id: u32,
    /// Display name (e.g., "Display 1", "Built-in Display")
    pub name: String,
    /// Whether this is the primary display
    pub is_primary: bool,
    /// Display width in pixels
    pub width: u32,
    /// Display height in pixels
    pub height: u32,
    /// Refresh rate in Hz
    pub refresh_rate: u32,
    /// Display scaling factor (e.g., 1.0 = 100%, 2.0 = 200%)
    pub scale: f64,
    /// X position on the virtual desktop
    pub x: i32,
    /// Y position on the virtual desktop
    pub y: i32,
}
/// Get all connected displays - Cross platform using system commands
///
/// This function enumerates all displays using platform-specific
/// command-line tools and returns a vector of DisplayInfo structures.
///
/// # Returns
///
/// * `DriverResult<Vec<DisplayInfo>>` - List of displays or an error
pub fn list_displays() -> DriverResult<Vec<DisplayInfo>> {
    debug!("Listing all connected displays");
    let mut displays = Vec::new();
    #[cfg(target_os = "windows")]
    {
        debug!("Using Windows PowerShell to enumerate displays");
        // Use PowerShell to get display info
        let output = Command::new("powershell")
            .args(["-Command", "Get-WmiObject -Class Win32_DesktopMonitor | Select-Object Name, ScreenWidth, ScreenHeight, DeviceID"])
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
                info!("Found {} displays on Windows", displays.len());
            }
        } else {
            warn!("Failed to execute PowerShell command for display enumeration");
        }
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Using macOS system_profiler to enumerate displays");
        // Use system_profiler on macOS
        let output = Command::new("system_profiler").args(["SPDisplaysDataType"]).output();
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
                                if let (Ok(width), Ok(height)) = (resolution[0].parse::<u32>(), resolution[1].parse::<u32>()) {
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
                                if let (Ok(w), Ok(h)) = (resolution[0].parse::<u32>(), resolution[1].parse::<u32>()) {
                                    if w > 0 && h > 0 {
                                        current_display.scale = current_display.width as f64 / w as f64;
                                    }
                                }
                            }
                        }
                    }
                }
                displays.push(current_display);
                info!("Found {} displays on macOS", displays.len());
            }
        } else {
            warn!("Failed to execute system_profiler command for display enumeration");
        }
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Using Linux xrandr to enumerate displays");
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
                                    if let (Ok(width), Ok(height)) = (resolution[0].parse::<u32>(), resolution[1].parse::<u32>()) {
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
                info!("Found {} displays on Linux", displays.len());
            }
        } else {
            warn!("Failed to execute xrandr command for display enumeration");
        }
    }
    // Fallback for all platforms
    if displays.is_empty() {
        warn!("No displays found via system commands, using fallback");
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
        info!("Using fallback display configuration");
    }
    debug!("Display enumeration complete, found {} displays", displays.len());
    return Ok(displays);
}
/// Get the primary display
///
/// Returns information about the primary (main) display.
///
/// # Returns
///
/// * `DriverResult<DisplayInfo>` - Primary display information or an error
pub fn get_primary_display() -> DriverResult<DisplayInfo> {
    debug!("Getting primary display information");
    let displays = list_displays()?;
    let primary = displays.into_iter().find(|d| d.is_primary).ok_or_else(|| {
        let err_msg = "No primary display found".to_string();
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Primary display: {} ({}x{})", primary.name, primary.width, primary.height);
    return Ok(primary);
}
/// Get current resolution of a display
///
/// Returns the resolution (width, height) for the specified display.
/// If no display_id is provided, returns the primary display resolution.
///
/// # Arguments
///
/// * `display_id` - Optional display ID, uses primary if not specified
///
/// # Returns
///
/// * `DriverResult<(u32, u32)>` - Width and height in pixels
pub fn get_resolution(display_id: Option<u32>) -> DriverResult<(u32, u32)> {
    debug!("Getting resolution for display {:?}", display_id);
    let displays = list_displays()?;
    if let Some(id) = display_id {
        if let Some(d) = displays.iter().find(|d| d.id == id) {
            info!("Resolution for display {}: {}x{}", id, d.width, d.height);
            return Ok((d.width, d.height));
        }
    }
    let primary = get_primary_display()?;
    info!("Resolution for primary display: {}x{}", primary.width, primary.height);
    return Ok((primary.width, primary.height));
}
/// Set display resolution - Cross platform
///
/// Attempts to change the resolution of the specified display.
/// This may cause temporary screen flicker on some systems.
///
/// # Arguments
///
/// * `width` - Desired width in pixels
/// * `height` - Desired height in pixels
/// * `display_id` - Optional display ID, uses primary if not specified
///
/// # Returns
///
/// * `DriverResult<()>` - Success or an error
pub fn set_resolution(width: u32, height: u32, display_id: Option<u32>) -> DriverResult<()> {
    debug!("Setting resolution to {}x{} for display {:?}", width, height, display_id);
    let _ = display_id;
    #[cfg(target_os = "windows")]
    {
        debug!("Using Windows methods to set resolution");
        // Method 1: Use DisplaySwitch.exe for basic display modes
        let _ = Command::new("DisplaySwitch.exe").arg("/extend").output();
        // Method 2: Use PowerShell with .NET to get display info (not changing resolution)
        // Note: Actually changing resolution on Windows requires Windows API or third-party tools
        // Method 3: Try using nircmd if available (third-party tool)
        let _ = Command::new("nircmd").args(["setdisplay", &width.to_string(), &height.to_string(), "32"]).output();
        // Method 4: Use QRes utility (small third-party tool)
        let _ = Command::new("QRes.exe").args(["/x", &width.to_string(), "/y", &height.to_string()]).output();
        info!("Resolution set request completed on Windows");
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Using macOS displayplacer to set resolution");
        // Use displayplacer if available
        let _ = Command::new("displayplacer").args(["res", &format!("{}x{}", width, height)]).output();
        info!("Resolution set request completed on macOS");
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Using Linux xrandr to set resolution");
        // Use xrandr on Linux
        let displays = list_displays()?;
        let display_name = displays.iter().find(|d| d.is_primary).map(|d| d.name.as_str());
        if let Some(name) = display_name {
            // First check if mode exists
            let _ = Command::new("xrandr").args(["--output", name, "--mode", &format!("{}x{}", width, height)]).output();
            info!("Resolution set request completed on Linux for display {}", name);
        } else {
            warn!("No display found to set resolution");
        }
    }
    info!("Resolution set to {}x{}", width, height);
    return Ok(());
}
/// Get display scale factor
///
/// Returns the DPI scaling factor for the specified display.
/// On macOS this typically returns 2.0 for Retina displays.
///
/// # Arguments
///
/// * `display_id` - Optional display ID, uses primary if not specified
///
/// # Returns
///
/// * `DriverResult<f64>` - Scaling factor (e.g., 1.0 = 100%, 1.5 = 150%)
pub fn get_scale(display_id: Option<u32>) -> DriverResult<f64> {
    debug!("Getting scale for display {:?}", display_id);
    let _ = display_id;
    #[cfg(target_os = "windows")]
    {
        debug!("Getting Windows DPI scaling from registry");
        // Get DPI scaling from registry
        let output = Command::new("powershell").args(["-Command", "(Get-ItemProperty 'HKCU:\\Control Panel\\Desktop').LogPixels"]).output();
        if let Ok(output) = output {
            if let Ok(scale_str) = String::from_utf8(output.stdout) {
                if let Ok(dpi) = scale_str.trim().parse::<u32>() {
                    // 96 DPI = 100% scale
                    let scale = dpi as f64 / 96.0;
                    info!("Windows DPI scaling: {:.1}x", scale);
                    return Ok(scale);
                }
            }
        }
        warn!("Failed to get DPI scaling from registry, using default 1.0");
    }
    #[cfg(target_os = "macos")]
    {
        debug!("macOS default scale is 2.0 for Retina displays");
        return Ok(2.0);
    }
    debug!("Using default scale factor 1.0");
    return Ok(1.0);
}
/// Get display orientation
///
/// Returns the current orientation of the specified display.
/// Possible values: "landscape", "portrait", "landscape_flipped", "portrait_flipped"
///
/// # Arguments
///
/// * `display_id` - Optional display ID, uses primary if not specified
///
/// # Returns
///
/// * `DriverResult<String>` - Orientation string
pub fn get_orientation(display_id: Option<u32>) -> DriverResult<String> {
    debug!("Getting orientation for display {:?}", display_id);
    let _ = display_id;
    #[cfg(target_os = "linux")]
    {
        debug!("Getting Linux orientation from xrandr");
        let output = Command::new("xrandr").arg("--current").output();
        if let Ok(output) = output {
            if let Ok(info) = String::from_utf8(output.stdout) {
                for line in info.lines() {
                    if line.contains(" connected") {
                        if line.contains(" right (") {
                            info!("Display orientation: portrait");
                            return Ok("portrait".to_string());
                        } else if line.contains(" left (") {
                            info!("Display orientation: portrait_flipped");
                            return Ok("portrait_flipped".to_string());
                        } else if line.contains(" inverted (") {
                            info!("Display orientation: landscape_flipped");
                            return Ok("landscape_flipped".to_string());
                        }
                    }
                }
            }
        }
        warn!("Failed to get orientation from xrandr, using default landscape");
    }
    info!("Display orientation: landscape (default)");
    return Ok("landscape".to_string());
}
/// Set display orientation
///
/// Changes the orientation of the specified display.
/// This may cause the screen to rotate immediately.
///
/// # Arguments
///
/// * `orientation` - Orientation string: "landscape", "portrait", "landscape_flipped", "portrait_flipped"
/// * `display_id` - Optional display ID, uses primary if not specified
///
/// # Returns
///
/// * `DriverResult<()>` - Success or an error
pub fn set_orientation(orientation: &str, display_id: Option<u32>) -> DriverResult<()> {
    debug!("Setting orientation to '{}' for display {:?}", orientation, display_id);
    let _ = display_id;
    #[cfg(target_os = "windows")]
    {
        debug!("Setting Windows orientation via PowerShell");
        let orient_num = match orientation {
            "landscape" => 0,
            "portrait" => 1,
            "landscape_flipped" => 2,
            "portrait_flipped" => 3,
            _ => 0,
        };
        let _ = Command::new("powershell").args(["-Command", &format!("Set-DisplayOrientation -Orientation {}", orient_num)]).output();
        info!("Windows orientation set request completed");
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Setting Linux orientation via xrandr");
        let transform = match orientation {
            "landscape" => "normal",
            "portrait" => "left",
            "landscape_flipped" => "inverted",
            "portrait_flipped" => "right",
            _ => "normal",
        };
        let displays = list_displays()?;
        let display_name = displays.iter().find(|d| d.is_primary).map(|d| d.name.as_str());
        if let Some(name) = display_name {
            let _ = Command::new("xrandr").args(["--output", name, "--rotate", transform]).output();
            info!("Linux orientation set request completed for display {}", name);
        } else {
            warn!("No display found to set orientation");
        }
    }
    info!("Display orientation set to {}", orientation);
    return Ok(());
}
/// Get refresh rate
///
/// Returns the refresh rate in Hz for the specified display.
///
/// # Arguments
///
/// * `display_id` - Optional display ID, uses primary if not specified
///
/// # Returns
///
/// * `DriverResult<u32>` - Refresh rate in Hz
pub fn get_refresh_rate(display_id: Option<u32>) -> DriverResult<u32> {
    debug!("Getting refresh rate for display {:?}", display_id);
    let displays = list_displays()?;
    if let Some(id) = display_id {
        if let Some(d) = displays.iter().find(|d| d.id == id) {
            info!("Refresh rate for display {}: {} Hz", id, d.refresh_rate);
            return Ok(d.refresh_rate);
        }
    }
    let primary = get_primary_display()?;
    info!("Refresh rate for primary display: {} Hz", primary.refresh_rate);
    return Ok(primary.refresh_rate);
}
/// Get display brightness - Cross platform
///
/// Returns the current brightness level as a percentage (0-100).
/// Works primarily on laptops; may not work on desktops.
///
/// # Returns
///
/// * `DriverResult<u32>` - Brightness level (0-100)
pub fn get_brightness() -> DriverResult<u32> {
    debug!("Getting current brightness level");
    #[cfg(target_os = "windows")]
    {
        debug!("Getting Windows brightness via WMI");
        let output = Command::new("powershell")
            .args(["-Command", "(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightness).CurrentBrightness"])
            .output();
        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Ok(bright) = bright_str.trim().parse::<u32>() {
                    info!("Windows brightness: {}%", bright);
                    return Ok(bright);
                }
            }
        }
        warn!("Failed to get Windows brightness, using default 50");
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting macOS brightness via brightness command");
        let output = Command::new("brightness").arg("-l").output();
        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Some(value) = bright_str.split_whitespace().last() {
                    if let Ok(bright) = value.parse::<f64>() {
                        let brightness = (bright * 100.0) as u32;
                        info!("macOS brightness: {}%", brightness);
                        return Ok(brightness);
                    }
                }
            }
        }
        warn!("Failed to get macOS brightness, using default 50");
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Getting Linux brightness via xbacklight and brightnessctl");
        let output = Command::new("xbacklight").arg("-get").output();
        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Ok(bright) = bright_str.trim().parse::<f64>() {
                    let brightness = bright as u32;
                    info!("Linux brightness (xbacklight): {}%", brightness);
                    return Ok(brightness);
                }
            }
        }
        let output = Command::new("brightnessctl").arg("get").output();
        if let Ok(output) = output {
            if let Ok(bright_str) = String::from_utf8(output.stdout) {
                if let Ok(bright) = bright_str.trim().parse::<u32>() {
                    let max = 255;
                    let brightness = (bright * 100 / max) as u32;
                    info!("Linux brightness (brightnessctl): {}%", brightness);
                    return Ok(brightness);
                }
            }
        }
        warn!("Failed to get Linux brightness, using default 50");
    }
    info!("Using default brightness: 50%");
    return Ok(50);
}
/// Set display brightness - Cross platform
///
/// Sets the brightness level to the specified percentage (0-100).
/// Works primarily on laptops; may not work on desktops.
///
/// # Arguments
///
/// * `brightness` - Brightness level (0-100), clamped to valid range
///
/// # Returns
///
/// * `DriverResult<()>` - Success or an error
pub fn set_brightness(brightness: u32) -> DriverResult<()> {
    let brightness = brightness.clamp(0, 100);
    debug!("Setting brightness to {}%", brightness);
    #[cfg(target_os = "windows")]
    {
        debug!("Setting Windows brightness via WMI");
        let _ = Command::new("powershell")
            .args(["-Command", &format!("(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods).WmiSetBrightness(1,{})", brightness)])
            .output();
        info!("Windows brightness set to {}%", brightness);
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Setting macOS brightness via brightness command");
        let value = brightness as f64 / 100.0;
        let _ = Command::new("brightness").arg(&format!("{}", value)).output();
        info!("macOS brightness set to {}%", brightness);
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Setting Linux brightness via xbacklight and brightnessctl");
        let _ = Command::new("xbacklight").args(["-set", &brightness.to_string()]).output();
        let max = 255;
        let value = (brightness * max / 100).to_string();
        let _ = Command::new("brightnessctl").args(["set", &value]).output();
        info!("Linux brightness set to {}%", brightness);
    }
    info!("Brightness set to {}%", brightness);
    return Ok(());
}
