//! OS get hostname driver
//!
//! This driver provides functionality to get or set the system hostname.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting or setting the hostname
#[derive(Debug)]
pub struct OsGetHostnameDriver;
#[async_trait::async_trait]
impl Driver for OsGetHostnameDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_hostname"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get or set the system hostname"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current hostname or set a new one (requires admin privileges)"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "set_hostname".to_string(),
            param_type: "string".to_string(),
            description: "New hostname to set (requires admin)".to_string(),
            required: false,
            default: None,
            example: Some(json!("my-server")),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_hostname"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current hostname: my-computer".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_get_hostname driver");
        let new_hostname = parameters.get("set_hostname").and_then(|v| v.as_str());
        if let Some(name) = new_hostname {
            debug!("Setting hostname to: {}", name);
            #[cfg(not(target_os = "windows"))]
            {
                let _ = exec_async("sudo", &["hostname", name], None).await;
            }
            #[cfg(target_os = "windows")]
            {
                let _ = exec_async("powershell", &["-Command", &format!("Rename-Computer -NewName '{}'", name)], None).await;
            }
            info!("Hostname changed to: {}", name);
            return Ok(format!("Hostname changed to: {}", name));
        } else {
            let hostname = System::host_name();
            info!("Current hostname retrieved");
            return Ok(format!("Current hostname: {}", hostname.unwrap_or_else(|| "unknown".to_string())));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_hostname_metadata() {
        let driver = OsGetHostnameDriver;
        assert_eq!(driver.name(), "os_get_hostname");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
