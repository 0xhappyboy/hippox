//! OS get locale driver
//!
//! This driver provides functionality to get system language and locale settings.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting locale information
#[derive(Debug)]
pub struct OsGetLocaleDriver;
#[async_trait::async_trait]
impl Driver for OsGetLocaleDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_locale"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get system language and locale settings"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the system language, country, and encoding settings"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_locale"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Language: en-US\nLocale: en_US.UTF-8\nSystem Language: English (United States)".to_string();
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
        debug!("Executing os_get_locale driver");
        let locale = get_locale()?;
        info!("Locale retrieved: {}", locale.locale);
        return Ok(format!("Language: {}\nLocale: {}\nSystem Language: {}", locale.language, locale.locale, locale.display_name));
    }
}
/// Locale information structure
#[derive(Debug)]
struct LocaleInfo {
    language: String,
    locale: String,
    display_name: String,
}
/// Gets the system locale information
fn get_locale() -> DriverResult<LocaleInfo> {
    #[cfg(target_os = "windows")]
    {
        debug!("Getting locale on Windows");
        let output = Command::new("powershell").args(["-Command", "Get-Culture | Select-Object Name, DisplayName, LCID"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Name") {
                        if let Some(name) = line.split(':').nth(1) {
                            let name = name.trim();
                            info!("Locale found on Windows: {}", name);
                            return Ok(LocaleInfo { language: name.to_string(), locale: name.to_string(), display_name: name.to_string() });
                        }
                    }
                }
            }
        }
        info!("Locale not found on Windows, using default en-US");
        return Ok(LocaleInfo { language: "en-US".to_string(), locale: "en-US".to_string(), display_name: "English (United States)".to_string() });
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Getting locale on Linux");
        if let Ok(content) = std::fs::read_to_string("/etc/default/locale") {
            for line in content.lines() {
                if line.starts_with("LANG=") {
                    if let Some(lang) = line.strip_prefix("LANG=") {
                        let lang = lang.trim().trim_matches('"');
                        info!("Locale found in /etc/default/locale on Linux: {}", lang);
                        return Ok(LocaleInfo { language: lang.to_string(), locale: lang.to_string(), display_name: lang.to_string() });
                    }
                }
            }
        }
        if let Ok(output) = Command::new("locale").arg("-a").output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("en_US.utf8") || line.contains("en_US.UTF-8") {
                        info!("Locale found via locale command on Linux: {}", line);
                        return Ok(LocaleInfo {
                            language: "en-US".to_string(),
                            locale: line.to_string(),
                            display_name: "English (United States)".to_string(),
                        });
                    }
                    if line.contains("zh_CN.utf8") || line.contains("zh_CN.UTF-8") {
                        info!("Locale found via locale command on Linux: {}", line);
                        return Ok(LocaleInfo {
                            language: "zh-CN".to_string(),
                            locale: line.to_string(),
                            display_name: "Chinese (China)".to_string(),
                        });
                    }
                }
            }
        }
        info!("Locale not found on Linux, using default en_US.UTF-8");
        return Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en_US.UTF-8".to_string(),
            display_name: "English (United States)".to_string(),
        });
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting locale on macOS");
        let output = Command::new("defaults").args(["read", "-g", "AppleLocale"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let lang = output_str.trim().trim_matches('"');
                if !lang.is_empty() {
                    info!("Locale found via defaults on macOS: {}", lang);
                    return Ok(LocaleInfo { language: lang.to_string(), locale: lang.to_string(), display_name: lang.to_string() });
                }
            }
        }
        let output = Command::new("system_profiler").args(["SPSoftwareDataType"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("System Language") {
                        if let Some(lang) = line.split(':').nth(1) {
                            let lang = lang.trim();
                            info!("Locale found via system_profiler on macOS: {}", lang);
                            return Ok(LocaleInfo { language: lang.to_string(), locale: lang.to_string(), display_name: lang.to_string() });
                        }
                    }
                }
            }
        }
        info!("Locale not found on macOS, using default en_US.UTF-8");
        return Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en_US.UTF-8".to_string(),
            display_name: "English (United States)".to_string(),
        });
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        debug!("Platform not supported for locale detection");
        return Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en_US.UTF-8".to_string(),
            display_name: "English (United States)".to_string(),
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_locale_metadata() {
        let driver = OsGetLocaleDriver;
        assert_eq!(driver.name(), "os_get_locale");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
