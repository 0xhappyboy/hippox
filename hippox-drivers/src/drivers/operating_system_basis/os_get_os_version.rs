//! OS get version driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
#[derive(Debug)]
pub struct OsGetOsVersionDriver;
#[async_trait::async_trait]
impl Driver for OsGetOsVersionDriver {
    fn name(&self) -> &str {
        "os_get_os_version"
    }
    fn description(&self) -> &str {
        "Get detailed operating system version information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get detailed OS version, kernel, and build information"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_os_version"
        })
    }
    fn example_output(&self) -> String {
        "OS: Windows 11 Pro (23H2)\nKernel: 10.0.22631\nBuild: 22631".to_string()
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
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let arch = std::env::consts::ARCH;
        let os_type = std::env::consts::OS;
        #[cfg(target_os = "windows")]
        {
            let build = get_windows_build();
            return Ok(format!(
                "OS: {} {}\nKernel: {}\nBuild: {}\nArchitecture: {}\nHostname: {}",
                os_name, os_version, kernel, build, arch, hostname
            ));
        }
        #[cfg(target_os = "linux")]
        {
            let pretty_name = get_linux_pretty_name();
            return Ok(format!(
                "OS: {}\nKernel: {}\nDistribution: {}\nArchitecture: {}\nHostname: {}",
                pretty_name, kernel, os_name, arch, hostname
            ));
        }
        #[cfg(target_os = "macos")]
        {
            let product_version = get_macos_version();
            return Ok(format!(
                "OS: macOS {}\nKernel: {}\nArchitecture: {}\nHostname: {}",
                product_version, kernel, arch, hostname
            ));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Ok(format!(
                "OS: {} ({} {})\nKernel: {}\nArchitecture: {}\nHostname: {}",
                os_type, os_name, os_version, kernel, arch, hostname
            ))
        }
    }
}
#[cfg(target_os = "windows")]
fn get_windows_build() -> String {
    use std::process::Command;
    let output = Command::new("powershell")
        .args([
            "-Command",
            "(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').CurrentBuild"
        ])
        .output();
    if let Ok(output) = output {
        if let Ok(build_str) = String::from_utf8(output.stdout) {
            let build = build_str.trim();
            if !build.is_empty() {
                return build.to_string();
            }
        }
    }
    "Unknown".to_string()
}
#[cfg(target_os = "linux")]
fn get_linux_pretty_name() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                    return name.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string("/etc/lsb-release") {
        for line in content.lines() {
            if line.starts_with("DISTRIB_DESCRIPTION=") {
                if let Some(name) = line.strip_prefix("DISTRIB_DESCRIPTION=") {
                    return name.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}
#[cfg(target_os = "macos")]
fn get_macos_version() -> String {
    use std::process::Command;
    let output = Command::new("sw_vers").args(["-productVersion"]).output();
    if let Ok(output) = output {
        if let Ok(version_str) = String::from_utf8(output.stdout) {
            let version = version_str.trim();
            if !version.is_empty() {
                return version.to_string();
            }
        }
    }
    "Unknown".to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_os_version_metadata() {
        let driver = OsGetOsVersionDriver;
        assert_eq!(driver.name(), "os_get_os_version");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
