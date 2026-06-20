//! OS get hostname driver
use crate::{
    DriverCallback, DriverCategory, DriverContext, exec_async,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
#[derive(Debug)]
pub struct OsGetHostnameDriver;
#[async_trait::async_trait]
impl Driver for OsGetHostnameDriver {
    fn name(&self) -> &str {
        "os_get_hostname"
    }
    fn description(&self) -> &str {
        "Get or set the system hostname"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current hostname or set a new one (requires admin privileges)"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "set_hostname".to_string(),
            param_type: "string".to_string(),
            description: "New hostname to set (requires admin)".to_string(),
            required: false,
            default: None,
            example: Some(json!("my-server")),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_hostname"
        })
    }
    fn example_output(&self) -> String {
        "Current hostname: my-computer".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let new_hostname = parameters.get("set_hostname").and_then(|v| v.as_str());
        if let Some(name) = new_hostname {
            #[cfg(not(target_os = "windows"))]
            {
                let _ = exec_async("sudo", &["hostname", name], None).await;
            }
            #[cfg(target_os = "windows")]
            {
                let _ = exec_async(
                    "powershell",
                    &["-Command", &format!("Rename-Computer -NewName '{}'", name)],
                    None,
                )
                .await;
            }
            Ok(format!("Hostname changed to: {}", name))
        } else {
            let hostname = System::host_name();
            Ok(format!(
                "Current hostname: {}",
                hostname.unwrap_or_else(|| "unknown".to_string())
            ))
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
