//! OS reboot driver
//!
//! This driver provides functionality to reboot the system.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for rebooting the system
#[derive(Debug)]
pub struct OsRebootDriver;
#[async_trait::async_trait]
impl Driver for OsRebootDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_reboot"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Reboot the system"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to restart the system for updates or troubleshooting"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "delay".to_string(),
                param_type: "integer".to_string(),
                description: "Delay in seconds before reboot (default: 0)".to_string(),
                required: false,
                default: Some(json!(0)),
                example: Some(json!(60)),
                enum_values: None,
            },
            DriverParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force reboot without asking (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_reboot",
            "parameters": {
                "delay": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System will reboot in 10 seconds".to_string();
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
        debug!("Executing os_reboot driver");
        let delay = parameters.get("delay").and_then(|v| v.as_u64()).unwrap_or(0);
        let force = parameters.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Rebooting system with delay={}s, force={}", delay, force);
        #[cfg(target_os = "windows")]
        {
            debug!("Rebooting on Windows");
            let mut args: Vec<String> = vec!["/r".to_string()];
            if delay > 0 {
                args.push("/t".to_string());
                args.push(delay.to_string());
            }
            if force {
                args.push("/f".to_string());
            }
            let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            exec_async("shutdown", &args_ref, None).await.map_err(|e| DriverError::execution(format!("Failed to reboot on Windows: {}", e)))?;
            info!("System will reboot on Windows in {} seconds", delay);
        }
        #[cfg(not(target_os = "windows"))]
        {
            debug!("Rebooting on Unix-like system");
            let mut args = vec!["shutdown"];
            if delay > 0 {
                args.push("-h");
                args.push(&format!("+{}", delay / 60));
            } else {
                args.push("-r");
                args.push("now");
            }
            if force {
                args.push("-f");
            }
            let _ = exec_async("sudo", &args, None).await;
            info!("System will reboot on Unix-like system in {} seconds", delay);
        }
        return Ok(format!("System will reboot in {} seconds", delay));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_reboot_metadata() {
        let driver = OsRebootDriver;
        assert_eq!(driver.name(), "os_reboot");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
