//! Application restart skill

use super::common::{close_process_window, find_process_by_name, launch_app, wait_for_exit};
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
pub struct ApplicationControlRestartDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlRestartDriver {
    fn name(&self) -> &str {
        "application_control_restart"
    }

    fn description(&self) -> &str {
        "Restart an application (close and relaunch)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to restart a hung or misbehaving application."
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
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the application executable (if different from name)"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String(
                    "C:\\Windows\\System32\\notepad.exe".to_string(),
                )),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_restart",
            "parameters": {
                "name": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Application restarted".to_string()
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
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(name);
        // Find and close existing instances
        let processes = find_process_by_name(name)?;
        for process in processes {
            let _ = close_process_window(process.pid);
            let _ = wait_for_exit(process.pid, 5000).await;
        }
        // Launch new instance
        let pid = launch_app(path)?;
        Ok(format!("Application restarted with PID: {}", pid))
    }
}
