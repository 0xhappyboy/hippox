//! Application is running check driver

use super::common::find_process_by_name;
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
pub struct ApplicationControlIsRunningDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlIsRunningDriver {
    fn name(&self) -> &str {
        "application_control_is_running"
    }

    fn description(&self) -> &str {
        "Check if an application is currently running"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if a specific application is active."
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
            "action": "application_control_is_running",
            "parameters": {
                "name": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Application is running: true".to_string()
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
        let is_running = !processes.is_empty();
        Ok(format!("Application is running: {}", is_running))
    }
}
