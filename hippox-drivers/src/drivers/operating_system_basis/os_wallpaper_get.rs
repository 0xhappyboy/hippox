//! OS wallpaper get driver
//!
//! This driver provides functionality to get the current desktop wallpaper path.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting the wallpaper path
#[derive(Debug)]
pub struct OsWallpaperGetDriver;
#[async_trait::async_trait]
impl Driver for OsWallpaperGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_wallpaper_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current desktop wallpaper path"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the file path of the current desktop wallpaper"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_wallpaper_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current wallpaper: /Users/username/Pictures/wallpaper.jpg".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_wallpaper_get driver");
        let path = get_wallpaper_path()?;
        info!("Wallpaper path retrieved: {}", path);
        return Ok(format!("Current wallpaper: {}", path));
    }
}
/// Gets the current wallpaper path
fn get_wallpaper_path() -> DriverResult<String> {
    #[cfg(target_os = "windows")]
    {
        debug!("Getting wallpaper path on Windows");
        let output = Command::new("powershell").args(["-Command", "(Get-ItemProperty 'HKCU:\\Control Panel\\Desktop').Wallpaper"]).output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() {
                    info!("Wallpaper path found on Windows: {}", path);
                    return Ok(path.to_string());
                }
            }
        }
        info!("Wallpaper path not found on Windows");
        return Ok("Unknown".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Getting wallpaper path on Linux");
        let output = Command::new("gsettings").args(["get", "org.gnome.desktop.background", "picture-uri"]).output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim().trim_matches('\'').trim_matches('"');
                if !path.is_empty() && path != "''" {
                    if let Some(stripped) = path.strip_prefix("file://") {
                        info!("Wallpaper path found via gsettings on Linux: {}", stripped);
                        return Ok(stripped.to_string());
                    }
                    info!("Wallpaper path found via gsettings on Linux: {}", path);
                    return Ok(path.to_string());
                }
            }
        }
        let output = Command::new("xfconf-query").args(["-c", "xfdesktop", "-p", "/backdrop/screen0/monitor0/image-path"]).output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() && path != "''" {
                    info!("Wallpaper path found via xfconf-query on Linux: {}", path);
                    return Ok(path.to_string());
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let candidates = [format!("{}/.config/feh/fehbg", home), format!("{}/.config/i3/config", home), format!("{}/.config/sway/config", home)];
            for candidate in candidates {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    for line in content.lines() {
                        if line.contains("wallpaper") || line.contains("bg") {
                            if let Some(path) =
                                line.split_whitespace().find(|s| s.contains('/') && (s.contains(".jpg") || s.contains(".png") || s.contains(".jpeg")))
                            {
                                info!("Wallpaper path found in config file on Linux: {}", path);
                                return Ok(path.to_string());
                            }
                        }
                    }
                }
            }
        }
        info!("Wallpaper path not found on Linux");
        return Ok("Unknown".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting wallpaper path on macOS");
        let output = Command::new("osascript").args(["-e", "tell application \"Finder\" to get desktop picture as POSIX file"]).output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() {
                    info!("Wallpaper path found via osascript on macOS: {}", path);
                    return Ok(path.to_string());
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let plist_path = format!("{}/Library/Application Support/Dock/desktoppicture.db", home);
            if let Ok(content) = std::fs::read_to_string(&plist_path) {
                for line in content.lines() {
                    if line.contains(".jpg") || line.contains(".png") {
                        if let Some(start) = line.find('/') {
                            let end = line.rfind('"').or_else(|| line.rfind('\'')).unwrap_or(line.len());
                            if start < end {
                                let path = line[start..end].to_string();
                                info!("Wallpaper path found in plist on macOS: {}", path);
                                return Ok(path);
                            }
                        }
                    }
                }
            }
        }
        info!("Wallpaper path not found on macOS");
        return Ok("Unknown".to_string());
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        debug!("Platform not supported for wallpaper detection");
        return Ok("Unknown".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_wallpaper_get_metadata() {
        let driver = OsWallpaperGetDriver;
        assert_eq!(driver.name(), "os_wallpaper_get");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
