//! OS wallpaper set driver
//!
//! This driver provides functionality to set the desktop wallpaper.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info};
/// Driver for setting the wallpaper
#[derive(Debug)]
pub struct OsWallpaperSetDriver;
#[async_trait::async_trait]
impl Driver for OsWallpaperSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_wallpaper_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the desktop wallpaper"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to change the desktop wallpaper. Provide a path to an image file."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file (jpg, png, etc.)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/home/user/wallpaper.jpg".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_wallpaper_set",
            "parameters": {
                "path": "/home/user/wallpaper.jpg"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Wallpaper set to /home/user/wallpaper.jpg".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_wallpaper_set driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if !Path::new(path).exists() {
            debug!("File not found: {}", path);
            return Err(DriverError::execution(format!("File not found: {}", path)));
        }
        info!("Setting wallpaper to: {}", path);
        set_wallpaper(path)?;
        info!("Wallpaper set successfully");
        return Ok(format!("Wallpaper set to {}", path));
    }
}
/// Sets the wallpaper on Windows
#[cfg(target_os = "windows")]
fn set_wallpaper(path: &str) -> DriverResult<()> {
    debug!("Setting wallpaper on Windows");
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!("Set-ItemProperty -Path 'HKCU:\\Control Panel\\Desktop' -Name Wallpaper -Value '{}'; RUNDLL32.EXE user32.dll,UpdatePerUserSystemParameters", path)
        ])
        .output();
    return Ok(());
}
/// Sets the wallpaper on Linux
#[cfg(target_os = "linux")]
fn set_wallpaper(path: &str) -> DriverResult<()> {
    debug!("Setting wallpaper on Linux");
    let _ = Command::new("gsettings").args(["set", "org.gnome.desktop.background", "picture-uri", &format!("file://{}", path)]).output();
    let _ = Command::new("gsettings").args(["set", "org.gnome.desktop.background", "picture-uri-dark", &format!("file://{}", path)]).output();
    let _ = Command::new("feh").args(["--bg-scale", path]).output();
    let _ = Command::new("nitrogen").args(["--set-scaled", path]).output();
    return Ok(());
}
/// Sets the wallpaper on macOS
#[cfg(target_os = "macos")]
fn set_wallpaper(path: &str) -> DriverResult<()> {
    debug!("Setting wallpaper on macOS");
    let _ =
        Command::new("osascript").args(["-e", &format!("tell application \"Finder\" to set desktop picture to POSIX file \"{}\"", path)]).output();
    return Ok(());
}
/// Sets the wallpaper on unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn set_wallpaper(_path: &str) -> DriverResult<()> {
    debug!("Setting wallpaper not supported on this platform");
    return Err(DriverError::execution("Setting wallpaper is not supported on this platform"));
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
