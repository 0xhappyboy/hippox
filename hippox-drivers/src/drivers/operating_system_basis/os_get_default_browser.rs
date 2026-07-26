//! OS get default browser driver
//!
//! This driver provides functionality to get the default web browser path and name.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting the default browser
#[derive(Debug)]
pub struct OsGetDefaultBrowserDriver;
#[async_trait::async_trait]
impl Driver for OsGetDefaultBrowserDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_default_browser"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the default web browser path and name"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to find out which browser is set as the default"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_default_browser"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Default browser: Google Chrome (/Applications/Google Chrome.app)".to_string();
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
        debug!("Executing os_get_default_browser driver");
        let (name, path) = get_default_browser()?;
        info!("Default browser retrieved: {} ({})", name, path);
        return Ok(format!("Default browser: {} ({})", name, path));
    }
}
/// Gets the default browser name and path
fn get_default_browser() -> DriverResult<(String, String)> {
    #[cfg(target_os = "windows")]
    {
        debug!("Getting default browser on Windows");
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-ItemProperty 'HKCU:\\SOFTWARE\\Microsoft\\Windows\\Shell\\Associations\\UrlAssociations\\http\\UserChoice' | Select-Object -ExpandProperty Progid"
            ])
            .output();
        if let Ok(output) = output {
            if let Ok(progid_str) = String::from_utf8(output.stdout) {
                let progid = progid_str.trim();
                if !progid.is_empty() {
                    let output2 = Command::new("powershell")
                        .args(["-Command", &format!("(Get-ItemProperty 'HKCR:\\{}\\shell\\open\\command').'(Default)'", progid)])
                        .output();
                    if let Ok(output2) = output2 {
                        if let Ok(cmd_str) = String::from_utf8(output2.stdout) {
                            let cmd = cmd_str.trim().trim_matches('"');
                            if !cmd.is_empty() {
                                info!("Default browser found on Windows: {}", progid);
                                return Ok((progid.to_string(), cmd.to_string()));
                            }
                        }
                    }
                }
            }
        }
        info!("Default browser not found on Windows, returning Unknown");
        return Ok(("Unknown".to_string(), "Unknown".to_string()));
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Getting default browser on Linux");
        let output = Command::new("xdg-settings").args(["get", "default-web-browser"]).output();
        if let Ok(output) = output {
            if let Ok(browser_str) = String::from_utf8(output.stdout) {
                let browser = browser_str.trim();
                if !browser.is_empty() {
                    let name = browser.to_string();
                    let output2 = Command::new("which").args([browser]).output();
                    if let Ok(output2) = output2 {
                        if let Ok(path_str) = String::from_utf8(output2.stdout) {
                            let path = path_str.trim();
                            if !path.is_empty() {
                                info!("Default browser found on Linux: {}", name);
                                return Ok((name, path.to_string()));
                            }
                        }
                    }
                    info!("Default browser found on Linux but path not found: {}", name);
                    return Ok((name, browser.to_string()));
                }
            }
        }
        let browsers = ["google-chrome", "chromium", "firefox", "brave", "opera"];
        for browser in browsers {
            let output = Command::new("which").args([browser]).output();
            if let Ok(output) = output {
                if let Ok(path_str) = String::from_utf8(output.stdout) {
                    let path = path_str.trim();
                    if !path.is_empty() {
                        info!("Default browser found by scanning on Linux: {}", browser);
                        return Ok((browser.to_string(), path.to_string()));
                    }
                }
            }
        }
        info!("Default browser not found on Linux");
        return Ok(("Unknown".to_string(), "Unknown".to_string()));
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting default browser on macOS");
        let output = Command::new("defaults").args(["read", "com.apple.LaunchServices", "LSHandlers"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("LSHandlerURLScheme") && line.contains("http") {
                        if let Some(bundle_start) = line.find("LSHandlerRoleAll") {
                            if let Some(bundle_end) = line[bundle_start..].find(';') {
                                let bundle = line[bundle_start + 18..bundle_start + bundle_end].trim();
                                if !bundle.is_empty() {
                                    let name = bundle.replace('"', "");
                                    info!("Default browser found via LSHandlers on macOS: {}", name);
                                    return Ok((name.clone(), format!("{}", name)));
                                }
                            }
                        }
                    }
                }
            }
        }
        let output = Command::new("open").args(["-R", "http://"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stderr) {
                for line in output_str.lines() {
                    if line.contains("/Applications") && line.contains(".app") {
                        if let Some(path) = line.split_whitespace().find(|s| s.contains("/Applications")) {
                            let name = Path::new(path).file_name().unwrap_or_default().to_string_lossy().to_string();
                            info!("Default browser found via open on macOS: {}", name);
                            return Ok((name, path.to_string()));
                        }
                    }
                }
            }
        }
        let browsers = [
            ("Google Chrome", "/Applications/Google Chrome.app"),
            ("Chromium", "/Applications/Chromium.app"),
            ("Firefox", "/Applications/Firefox.app"),
            ("Safari", "/Applications/Safari.app"),
            ("Brave Browser", "/Applications/Brave Browser.app"),
            ("Opera", "/Applications/Opera.app"),
        ];
        for (name, path) in browsers {
            if Path::new(path).exists() {
                info!("Default browser found by scanning on macOS: {}", name);
                return Ok((name.to_string(), path.to_string()));
            }
        }
        info!("Default browser not found on macOS");
        return Ok(("Unknown".to_string(), "Unknown".to_string()));
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        debug!("Platform not supported for default browser detection");
        return Ok(("Unknown".to_string(), "Unknown".to_string()));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_default_browser_metadata() {
        let driver = OsGetDefaultBrowserDriver;
        assert_eq!(driver.name(), "os_get_default_browser");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
