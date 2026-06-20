//! OS wallpaper set driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
#[derive(Debug)]
pub struct OsWallpaperSetDriver;
#[async_trait::async_trait]
impl Driver for OsWallpaperSetDriver {
    fn name(&self) -> &str {
        "os_wallpaper_set"
    }
    fn description(&self) -> &str {
        "Set the desktop wallpaper"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to change the desktop wallpaper. Provide a path to an image file."
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file (jpg, png, etc.)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/home/user/wallpaper.jpg".to_string())),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_wallpaper_set",
            "parameters": {
                "path": "/home/user/wallpaper.jpg"
            }
        })
    }
    fn example_output(&self) -> String {
        "Wallpaper set to /home/user/wallpaper.jpg".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        if !Path::new(path).exists() {
            return Err(anyhow::anyhow!("File not found: {}", path));
        }
        set_wallpaper(path)?;
        Ok(format!("Wallpaper set to {}", path))
    }
}
#[cfg(target_os = "windows")]
fn set_wallpaper(path: &str) -> Result<()> {
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!("Set-ItemProperty -Path 'HKCU:\\Control Panel\\Desktop' -Name Wallpaper -Value '{}'; RUNDLL32.EXE user32.dll,UpdatePerUserSystemParameters", path)
        ])
        .output();
    Ok(())
}
#[cfg(target_os = "linux")]
fn set_wallpaper(path: &str) -> Result<()> {
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", &format!("file://{}", path)])
        .output();
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri-dark", &format!("file://{}", path)])
        .output();
    let _ = Command::new("feh")
        .args(["--bg-scale", path])
        .output();
    let _ = Command::new("nitrogen")
        .args(["--set-scaled", path])
        .output();
    Ok(())
}
#[cfg(target_os = "macos")]
fn set_wallpaper(path: &str) -> Result<()> {
    let _ = Command::new("osascript")
        .args([
            "-e",
            &format!("tell application \"Finder\" to set desktop picture to POSIX file \"{}\"", path)
        ])
        .output();
    Ok(())
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn set_wallpaper(_path: &str) -> Result<()> {
    Err(anyhow::anyhow!("Setting wallpaper is not supported on this platform"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_wallpaper_set_metadata() {
        let driver = OsWallpaperSetDriver;
        assert_eq!(driver.name(), "os_wallpaper_set");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}