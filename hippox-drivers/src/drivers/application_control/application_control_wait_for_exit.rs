//! Application wait for exit skill

use super::common::{find_process_by_name, wait_for_exit};
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ApplicationControlWaitForExitDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlWaitForExitDriver {
    fn name(&self) -> &str {
        "application_control_wait_for_exit"
    }

    fn description(&self) -> &str {
        "Wait for an application to exit"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to wait until an application has completely closed."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Application name or process name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum wait time in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(30000.into())),
                example: Some(Value::Number(10000.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_wait_for_exit",
            "parameters": {
                "name": "notepad.exe",
                "timeout_ms": 10000
            }
        })
    }

    fn example_output(&self) -> String {
        "Application exited within timeout".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Application
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;

        let timeout_ms = parameters
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30000);

        let processes = find_process_by_name(name)?;

        if processes.is_empty() {
            return Ok("Application is not running".to_string());
        }

        let pid = processes[0].pid;
        let exited = wait_for_exit(pid, timeout_ms).await?;

        if exited {
            Ok("Application exited within timeout".to_string())
        } else {
            anyhow::bail!("Timeout waiting for application to exit")
        }
    }
}
