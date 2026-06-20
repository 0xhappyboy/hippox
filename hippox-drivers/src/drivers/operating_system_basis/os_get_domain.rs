//! OS get domain driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsGetDomainDriver;
#[async_trait::async_trait]
impl Driver for OsGetDomainDriver {
    fn name(&self) -> &str {
        "os_get_domain"
    }
    fn description(&self) -> &str {
        "Get system domain or workgroup information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the domain or workgroup name of the system"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_domain"
        })
    }
    fn example_output(&self) -> String {
        "Domain: WORKGROUP".to_string()
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
        let domain = get_domain()?;
        Ok(format!("Domain: {}", domain))
    }
}
fn get_domain() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -ExpandProperty Domain"
            ])
            .output();
        if let Ok(output) = output {
            if let Ok(domain_str) = String::from_utf8(output.stdout) {
                let domain = domain_str.trim();
                if !domain.is_empty() {
                    return Ok(domain.to_string());
                }
            }
        }
        Ok("WORKGROUP".to_string())
    }
    #[cfg(target_os = "linux")]
    {
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
                    return Ok(domain.to_string());
                }
            }
        }
        Ok("Unknown".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("dsconfigad").args(["-show"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Active Directory Domain") {
                        if let Some(domain) = line.split(':').nth(1) {
                            let domain = domain.trim();
                            if !domain.is_empty() {
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
                        return Ok(domain.to_string());
                    }
                }
            }
        }
        Ok("Unknown".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok("Unknown".to_string())
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
