//! Application launch skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::launch_app;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct ApplicationControlLaunchDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlLaunchDriver {
    fn name(&self) -> &str {
        "application_control_launch"
    }

    fn description(&self) -> &str {
        "Launch an application"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to start an application by its path or name."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the application executable".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad.exe".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_launch",
            "parameters": {
                "path": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Application launched with PID: 12345".to_string()
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
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let pid = launch_app(path)?;
        Ok(format!("Application launched with PID: {}", pid))
    }
}
