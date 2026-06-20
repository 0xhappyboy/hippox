//! OS wallpaper get driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsWallpaperGetDriver;
#[async_trait::async_trait]
impl Driver for OsWallpaperGetDriver {
    fn name(&self) -> &str {
        "os_wallpaper_get"
    }
    fn description(&self) -> &str {
        "Get the current desktop wallpaper path"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the file path of the current desktop wallpaper"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_wallpaper_get"
        })
    }
    fn example_output(&self) -> String {
        "Current wallpaper: /Users/username/Pictures/wallpaper.jpg".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let path = get_wallpaper_path()?;
        Ok(format!("Current wallpaper: {}", path))
    }
}
fn get_wallpaper_path() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "(Get-ItemProperty 'HKCU:\\Control Panel\\Desktop').Wallpaper",
            ])
            .output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok(path.to_string());
                }
            }
        }
        Ok("Unknown".to_string())
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.background", "picture-uri"])
            .output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim().trim_matches('\'').trim_matches('"');
                if !path.is_empty() && path != "''" {
                    if let Some(stripped) = path.strip_prefix("file://") {
                        return Ok(stripped.to_string());
                    }
                    return Ok(path.to_string());
                }
            }
        }
        let output = Command::new("xfconf-query")
            .args([
                "-c",
                "xfdesktop",
                "-p",
                "/backdrop/screen0/monitor0/image-path",
            ])
            .output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() && path != "''" {
                    return Ok(path.to_string());
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let candidates = [
                format!("{}/.config/feh/fehbg", home),
                format!("{}/.config/i3/config", home),
                format!("{}/.config/sway/config", home),
            ];
            for candidate in candidates {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    for line in content.lines() {
                        if line.contains("wallpaper") || line.contains("bg") {
                            if let Some(path) = line.split_whitespace().find(|s| {
                                s.contains('/')
                                    && (s.contains(".jpg")
                                        || s.contains(".png")
                                        || s.contains(".jpeg"))
                            }) {
                                return Ok(path.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok("Unknown".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .args([
                "-e",
                "tell application \"Finder\" to get desktop picture as POSIX file",
            ])
            .output();
        if let Ok(output) = output {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok(path.to_string());
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let plist_path = format!(
                "{}/Library/Application Support/Dock/desktoppicture.db",
                home
            );
            if let Ok(content) = std::fs::read_to_string(&plist_path) {
                for line in content.lines() {
                    if line.contains(".jpg") || line.contains(".png") {
                        if let Some(start) = line.find('/') {
                            let end = line
                                .rfind('"')
                                .or_else(|| line.rfind('\''))
                                .unwrap_or(line.len());
                            if start < end {
                                return Ok(line[start..end].to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok("Unknown".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok("Unknown".to_string())
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
