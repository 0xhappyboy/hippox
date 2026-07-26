//! OS get domain driver
//!
//! This driver provides functionality to get system domain or workgroup information.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting domain information
#[derive(Debug)]
pub struct OsGetDomainDriver;
#[async_trait::async_trait]
impl Driver for OsGetDomainDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_domain"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get system domain or workgroup information"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the domain or workgroup name of the system"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_domain"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Domain: WORKGROUP".to_string();
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
        debug!("Executing os_get_domain driver");
        let domain = get_domain()?;
        info!("Domain retrieved: {}", domain);
        return Ok(format!("Domain: {}", domain));
    }
}
/// Gets the system domain
fn get_domain() -> DriverResult<String> {
    #[cfg(target_os = "windows")]
    {
        debug!("Getting domain on Windows");
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -ExpandProperty Domain"])
            .output();
        if let Ok(output) = output {
            if let Ok(domain_str) = String::from_utf8(output.stdout) {
                let domain = domain_str.trim();
                if !domain.is_empty() {
                    info!("Domain found on Windows: {}", domain);
                    return Ok(domain.to_string());
                }
            }
        }
        info!("Domain not found on Windows, returning WORKGROUP");
        return Ok("WORKGROUP".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        debug!("Getting domain on Linux");
        if let Ok(content) = std::fs::read_to_string("/etc/hostname") {
            let hostname = content.trim();
            if let Ok(content) = std::fs::read_to_string("/etc/hosts") {
                for line in content.lines() {
                    if line.contains(hostname) && line.contains('.') {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let fqdn = parts[1];
                            if let Some(domain) = fqdn.split('.').nth(1) {
                                if !domain.is_empty() {
                                    info!("Domain found in /etc/hosts on Linux: {}", domain);
                                    return Ok(domain.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Ok(output) = Command::new("hostname").arg("-d").output() {
            if let Ok(domain_str) = String::from_utf8(output.stdout) {
                let domain = domain_str.trim();
                if !domain.is_empty() {
                    info!("Domain found via hostname on Linux: {}", domain);
                    return Ok(domain.to_string());
                }
            }
        }
        info!("Domain not found on Linux");
        return Ok("Unknown".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting domain on macOS");
        let output = Command::new("dsconfigad").args(["-show"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Active Directory Domain") {
                        if let Some(domain) = line.split(':').nth(1) {
                            let domain = domain.trim();
                            if !domain.is_empty() {
                                info!("Domain found via dsconfigad on macOS: {}", domain);
                                return Ok(domain.to_string());
                            }
                        }
                    }
                }
            }
        }
        let output = Command::new("hostname").arg("-f").output();
        if let Ok(output) = output {
            if let Ok(hostname_str) = String::from_utf8(output.stdout) {
                let fqdn = hostname_str.trim();
                if let Some(domain) = fqdn.split('.').nth(1) {
                    if !domain.is_empty() {
                        info!("Domain found via hostname on macOS: {}", domain);
                        return Ok(domain.to_string());
                    }
                }
            }
        }
        info!("Domain not found on macOS");
        return Ok("Unknown".to_string());
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        debug!("Platform not supported for domain detection");
        return Ok("Unknown".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_domain_metadata() {
        let driver = OsGetDomainDriver;
        assert_eq!(driver.name(), "os_get_domain");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
