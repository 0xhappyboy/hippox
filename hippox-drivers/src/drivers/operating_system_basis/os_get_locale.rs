//! OS get locale driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsGetLocaleDriver;
#[async_trait::async_trait]
impl Driver for OsGetLocaleDriver {
    fn name(&self) -> &str {
        "os_get_locale"
    }
    fn description(&self) -> &str {
        "Get system language and locale settings"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the system language, country, and encoding settings"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_locale"
        })
    }
    fn example_output(&self) -> String {
        "Language: en-US\nLocale: en_US.UTF-8\nSystem Language: English (United States)".to_string()
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
        let locale = get_locale()?;
        Ok(format!(
            "Language: {}\nLocale: {}\nSystem Language: {}",
            locale.language, locale.locale, locale.display_name
        ))
    }
}
#[derive(Debug)]
struct LocaleInfo {
    language: String,
    locale: String,
    display_name: String,
}
fn get_locale() -> Result<LocaleInfo> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-Culture | Select-Object Name, DisplayName, LCID",
            ])
            .output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Name") {
                        if let Some(name) = line.split(':').nth(1) {
                            let name = name.trim();
                            return Ok(LocaleInfo {
                                language: name.to_string(),
                                locale: name.to_string(),
                                display_name: name.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en-US".to_string(),
            display_name: "English (United States)".to_string(),
        })
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/default/locale") {
            for line in content.lines() {
                if line.starts_with("LANG=") {
                    if let Some(lang) = line.strip_prefix("LANG=") {
                        let lang = lang.trim().trim_matches('"');
                        return Ok(LocaleInfo {
                            language: lang.to_string(),
                            locale: lang.to_string(),
                            display_name: lang.to_string(),
                        });
                    }
                }
            }
        }
        if let Ok(output) = Command::new("locale").arg("-a").output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("en_US.utf8") || line.contains("en_US.UTF-8") {
                        return Ok(LocaleInfo {
                            language: "en-US".to_string(),
                            locale: line.to_string(),
                            display_name: "English (United States)".to_string(),
                        });
                    }
                    if line.contains("zh_CN.utf8") || line.contains("zh_CN.UTF-8") {
                        return Ok(LocaleInfo {
                            language: "zh-CN".to_string(),
                            locale: line.to_string(),
                            display_name: "Chinese (China)".to_string(),
                        });
                    }
                }
            }
        }
        Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en_US.UTF-8".to_string(),
            display_name: "English (United States)".to_string(),
        })
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let lang = output_str.trim().trim_matches('"');
                if !lang.is_empty() {
                    return Ok(LocaleInfo {
                        language: lang.to_string(),
                        locale: lang.to_string(),
                        display_name: lang.to_string(),
                    });
                }
            }
        }
        let output = Command::new("system_profiler")
            .args(["SPSoftwareDataType"])
            .output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("System Language") {
                        if let Some(lang) = line.split(':').nth(1) {
                            let lang = lang.trim();
                            return Ok(LocaleInfo {
                                language: lang.to_string(),
                                locale: lang.to_string(),
                                display_name: lang.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en_US.UTF-8".to_string(),
            display_name: "English (United States)".to_string(),
        })
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(LocaleInfo {
            language: "en-US".to_string(),
            locale: "en_US.UTF-8".to_string(),
            display_name: "English (United States)".to_string(),
        })
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
