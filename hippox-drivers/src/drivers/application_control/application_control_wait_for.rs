//! Application wait for skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::find_process_by_name;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct ApplicationControlWaitForDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlWaitForDriver {
    fn name(&self) -> &str {
        "application_control_wait_for"
    }

    fn description(&self) -> &str {
        "Wait for an application to start running"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to wait until an application has launched."
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
            "action": "application_control_wait_for",
            "parameters": {
                "name": "notepad.exe",
                "timeout_ms": 10000
            }
        })
    }

    fn example_output(&self) -> String {
        "Application started within timeout".to_string()
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
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > std::time::Duration::from_millis(timeout_ms) {
                anyhow::bail!("Timeout waiting for application to start");
            }
            let processes = find_process_by_name(name)?;
            if !processes.is_empty() {
                return Ok("Application started within timeout".to_string());
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }
}
