//! OS reboot driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsRebootDriver;
#[async_trait::async_trait]
impl Driver for OsRebootDriver {
    fn name(&self) -> &str {
        "os_reboot"
    }
    fn description(&self) -> &str {
        "Reboot the system"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to restart the system for updates or troubleshooting"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_reboot",
            "parameters": {
                "delay": 10
            }
        })
    }
    fn example_output(&self) -> String {
        "System will reboot in 10 seconds".to_string()
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
        let delay = parameters
            .get("delay")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            let mut args: Vec<String> = vec!["/r".to_string()];
            if delay > 0 {
                args.push("/t".to_string());
                args.push(delay.to_string());
            }
            if force {
                args.push("/f".to_string());
            }
            let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            exec_async("shutdown", &args_ref, None).await?;
        }
        #[cfg(not(target_os = "windows"))]
        {
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
        }
        Ok(format!("System will reboot in {} seconds", delay))
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
