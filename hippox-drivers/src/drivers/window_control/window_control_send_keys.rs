//! Window send keys Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::find_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct WindowControlSendKeysDriver;

#[async_trait::async_trait]
impl Driver for WindowControlSendKeysDriver {
    fn name(&self) -> &str {
        "window_control_send_keys"
    }

    fn description(&self) -> &str {
        "Send keyboard input to a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to type text into a window"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("记事本".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to type".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_send_keys",
            "parameters": {
                "title": "记事本",
                "text": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "Text sent to window".to_string()
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
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing text"))?;

        if let Some(_window_id) = find_window(title, process).ok() {
            // Activate window first
            // set_foreground_window(window_id)?;
        }

        // Use enigo or similar to type
        // For now, placeholder
        let _ = text;

        Ok("Text sent to window (implementation pending)".to_string())
    }
}
