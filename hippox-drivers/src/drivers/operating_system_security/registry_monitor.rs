//! Registry monitor driver (Windows)
//!
//! This driver provides functionality to monitor Windows registry keys for security issues.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
#[cfg(target_os = "windows")]
use crate::operating_system_security::common::monitor_registry_key;
/// Driver for monitoring Windows registry keys
#[derive(Debug)]
pub struct RegistryMonitorDriver;
#[async_trait::async_trait]
impl Driver for RegistryMonitorDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_registry_monitor"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Monitor Windows registry keys for security issues"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to audit Windows registry keys for security issues like startup persistence and service configurations"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "key".to_string(),
            param_type: "string".to_string(),
            description: "Registry key path to monitor (default: HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run)".to_string(),
            required: false,
            default: Some(Value::String("HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string())),
            example: Some(Value::String("HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_registry_monitor",
            "parameters": {
                "key": "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Registry Monitor Results:\n\nKey: HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\nName: startup\nValue: C:\\Program Files\\App\\app.exe\nValue Type: REG_SZ\nSecurity Issues:\n  - Startup registry key - potential persistence".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemSecurity;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing security_registry_monitor driver");
        #[cfg(not(target_os = "windows"))]
        {
            info!("Registry monitor is only supported on Windows systems");
            return Ok("Registry monitor is only supported on Windows systems".to_string());
        }
        #[cfg(target_os = "windows")]
        {
            let key =
                parameters.get("key").and_then(|v| v.as_str()).unwrap_or("HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run");
            info!("Monitoring registry key: {}", key);
            let info = monitor_registry_key(key);
            let mut output = String::new();
            output.push_str(&format!("Registry Monitor Results:\n\nKey: {}\n", info.path));
            output.push_str(&format!("Name: {}\n", info.name));
            output.push_str(&format!("Value: {}\n", info.value));
            output.push_str(&format!("Value Type: {}\n", info.value_type));
            if !info.security_issues.is_empty() {
                info!("Found {} security issues in registry key", info.security_issues.len());
                output.push_str("\nSecurity Issues:\n");
                for issue in &info.security_issues {
                    output.push_str(&format!("  - {}\n", issue));
                }
            } else {
                output.push_str("\nNo security issues found.");
                info!("No security issues found in registry key");
            }
            return Ok(output);
        }
    }
}
