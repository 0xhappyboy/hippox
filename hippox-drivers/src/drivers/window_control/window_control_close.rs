//! Window close skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{close_window, find_window};
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WindowControlCloseDriver;

#[async_trait::async_trait]
impl Driver for WindowControlCloseDriver {
    fn name(&self) -> &str {
        "window_control_close"
    }

    fn description(&self) -> &str {
        "Close a specified window (graceful close)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to close a window by title or process name"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("计算器".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("calc.exe".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_close",
            "parameters": {
                "title": "计算器"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window closed".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Window
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());

        let window_id = find_window(title, process)?;
        close_window(window_id)?;

        Ok("Window closed".to_string())
    }
}
