//! OS get version driver
//!
//! This driver provides functionality to get detailed operating system version information.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting OS version information
#[derive(Debug)]
pub struct OsGetOsVersionDriver;
#[async_trait::async_trait]
impl Driver for OsGetOsVersionDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_os_version"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get detailed operating system version information"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get detailed OS version, kernel, and build information"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_os_version"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "OS: Windows 11 Pro (23H2)\nKernel: 10.0.22631\nBuild: 22631".to_string();
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
        debug!("Executing os_get_os_version driver");
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let arch = std::env::consts::ARCH;
        let os_type = std::env::consts::OS;
        #[cfg(target_os = "windows")]
        {
            debug!("Getting OS version on Windows");
            let build = get_windows_build();
            info!("OS version retrieved on Windows");
            return Ok(format!(
                "OS: {} {}\nKernel: {}\nBuild: {}\nArchitecture: {}\nHostname: {}",
                os_name, os_version, kernel, build, arch, hostname
            ));
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Getting OS version on Linux");
            let pretty_name = get_linux_pretty_name();
            info!("OS version retrieved on Linux");
            return Ok(format!("OS: {}\nKernel: {}\nDistribution: {}\nArchitecture: {}\nHostname: {}", pretty_name, kernel, os_name, arch, hostname));
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Getting OS version on macOS");
            let product_version = get_macos_version();
            info!("OS version retrieved on macOS");
            return Ok(format!("OS: macOS {}\nKernel: {}\nArchitecture: {}\nHostname: {}", product_version, kernel, arch, hostname));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            debug!("Getting OS version on unsupported platform");
            info!("OS version retrieved on unsupported platform");
            return Ok(format!("OS: {} ({} {})\nKernel: {}\nArchitecture: {}\nHostname: {}", os_type, os_name, os_version, kernel, arch, hostname));
        }
    }
}
/// Gets the Windows build number
#[cfg(target_os = "windows")]
fn get_windows_build() -> String {
    use std::process::Command;
    debug!("Getting Windows build number");
    let output = Command::new("powershell")
        .args(["-Command", "(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').CurrentBuild"])
        .output();
    if let Ok(output) = output {
        if let Ok(build_str) = String::from_utf8(output.stdout) {
            let build = build_str.trim();
            if !build.is_empty() {
                info!("Windows build number: {}", build);
                return build.to_string();
            }
        }
    }
    info!("Windows build number not found");
    return "Unknown".to_string();
}
/// Gets the Linux pretty name from os-release
#[cfg(target_os = "linux")]
fn get_linux_pretty_name() -> String {
    debug!("Getting Linux pretty name");
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                    let name = name.trim().trim_matches('"').to_string();
                    info!("Linux pretty name: {}", name);
                    return name;
                }
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string("/etc/lsb-release") {
        for line in content.lines() {
            if line.starts_with("DISTRIB_DESCRIPTION=") {
                if let Some(name) = line.strip_prefix("DISTRIB_DESCRIPTION=") {
                    let name = name.trim().trim_matches('"').to_string();
                    info!("Linux pretty name from lsb-release: {}", name);
                    return name;
                }
            }
        }
    }
    info!("Linux pretty name not found");
    return "Unknown".to_string();
}
/// Gets the macOS version
#[cfg(target_os = "macos")]
fn get_macos_version() -> String {
    use std::process::Command;
    debug!("Getting macOS version");
    let output = Command::new("sw_vers").args(["-productVersion"]).output();
    if let Ok(output) = output {
        if let Ok(version_str) = String::from_utf8(output.stdout) {
            let version = version_str.trim();
            if !version.is_empty() {
                info!("macOS version: {}", version);
                return version.to_string();
            }
        }
    }
    info!("macOS version not found");
    return "Unknown".to_string();
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
