//! Application close driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{close_process_window, find_process_by_name};
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ApplicationControlCloseDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlCloseDriver {
    fn name(&self) -> &str {
        "application_control_close"
    }

    fn description(&self) -> &str {
        "Close an application gracefully"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to close an application by sending a close message to its window."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Application name or process name".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad.exe".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_close",
            "parameters": {
                "name": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Application closed".to_string()
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

        let processes = find_process_by_name(name)?;

        if processes.is_empty() {
            anyhow::bail!("No process found with name: {}", name);
        }

        for process in processes {
            let _ = close_process_window(process.pid);
        }

        Ok("Application closed".to_string())
    }
}
