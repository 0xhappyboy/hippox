//! OS get timezone driver
//!
//! This driver provides functionality to get the current system timezone.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use chrono::Local;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting timezone information
#[derive(Debug)]
pub struct OsGetTimezoneDriver;
#[async_trait::async_trait]
impl Driver for OsGetTimezoneDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_timezone"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current system timezone"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current timezone (e.g., Asia/Shanghai, America/New_York)"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_timezone"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current timezone: Asia/Shanghai (UTC+8)".to_string();
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
        debug!("Executing os_get_timezone driver");
        let tz = get_timezone()?;
        let offset = Local::now().offset().to_owned();
        info!("Timezone retrieved: {} (UTC{})", tz, offset);
        return Ok(format!("Current timezone: {} (UTC{})", tz, offset));
    }
}
/// Gets the system timezone
fn get_timezone() -> DriverResult<String> {
    #[cfg(target_os = "windows")]
    {
        debug!("Getting timezone on Windows");
        let output = Command::new("powershell").args(["-Command", "(Get-TimeZone).Id"]).output();
        if let Ok(output) = output {
            if let Ok(tz_str) = String::from_utf8(output.stdout) {
                let tz = tz_str.trim();
                if !tz.is_empty() {
                    info!("Timezone found on Windows: {}", tz);
                    return Ok(tz.to_string());
                }
            }
        }
        info!("Timezone not found on Windows, defaulting to UTC");
        return Ok("UTC".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Getting timezone on Linux");
        if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
            let tz = content.trim();
            if !tz.is_empty() {
                info!("Timezone found in /etc/timezone on Linux: {}", tz);
                return Ok(tz.to_string());
            }
        }
        if let Ok(link) = std::fs::read_link("/etc/localtime") {
            if let Some(path) = link.to_str() {
                if let Some(tz) = path.strip_prefix("/usr/share/zoneinfo/") {
                    info!("Timezone found via /etc/localtime symlink on Linux: {}", tz);
                    return Ok(tz.to_string());
                }
            }
        }
        info!("Timezone not found on Linux, defaulting to UTC");
        return Ok("UTC".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting timezone on macOS");
        let output = Command::new("systemsetup").args(["-gettimezone"]).output();
        if let Ok(output) = output {
            if let Ok(tz_str) = String::from_utf8(output.stdout) {
                if let Some(tz) = tz_str.split(':').nth(1) {
                    let tz = tz.trim();
                    if !tz.is_empty() {
                        info!("Timezone found on macOS: {}", tz);
                        return Ok(tz.to_string());
                    }
                }
            }
        }
        info!("Timezone not found on macOS, defaulting to UTC");
        return Ok("UTC".to_string());
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        debug!("Platform not supported for timezone detection");
        return Ok("UTC".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_timezone_metadata() {
        let driver = OsGetTimezoneDriver;
        assert_eq!(driver.name(), "os_get_timezone");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
