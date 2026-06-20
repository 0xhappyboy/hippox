//! OS get timezone driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use chrono::Local;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsGetTimezoneDriver;
#[async_trait::async_trait]
impl Driver for OsGetTimezoneDriver {
    fn name(&self) -> &str {
        "os_get_timezone"
    }
    fn description(&self) -> &str {
        "Get the current system timezone"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current timezone (e.g., Asia/Shanghai, America/New_York)"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_timezone"
        })
    }
    fn example_output(&self) -> String {
        "Current timezone: Asia/Shanghai (UTC+8)".to_string()
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
        let tz = get_timezone()?;
        let offset = Local::now().offset().to_owned();
        Ok(format!("Current timezone: {} (UTC{})", tz, offset))
    }
}
fn get_timezone() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-Command", "(Get-TimeZone).Id"])
            .output();
        if let Ok(output) = output {
            if let Ok(tz_str) = String::from_utf8(output.stdout) {
                let tz = tz_str.trim();
                if !tz.is_empty() {
                    return Ok(tz.to_string());
                }
            }
        }
        Ok("UTC".to_string())
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
            let tz = content.trim();
            if !tz.is_empty() {
                return Ok(tz.to_string());
            }
        }
        if let Ok(link) = std::fs::read_link("/etc/localtime") {
            if let Some(path) = link.to_str() {
                if let Some(tz) = path.strip_prefix("/usr/share/zoneinfo/") {
                    return Ok(tz.to_string());
                }
            }
        }
        Ok("UTC".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("systemsetup").args(["-gettimezone"]).output();
        if let Ok(output) = output {
            if let Ok(tz_str) = String::from_utf8(output.stdout) {
                if let Some(tz) = tz_str.split(':').nth(1) {
                    let tz = tz.trim();
                    if !tz.is_empty() {
                        return Ok(tz.to_string());
                    }
                }
            }
        }
        Ok("UTC".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok("UTC".to_string())
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
