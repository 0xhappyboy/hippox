//! Window bring to top skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{find_window, set_foreground_window};
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WindowControlBringToTopDriver;

#[async_trait::async_trait]
impl Driver for WindowControlBringToTopDriver {
    fn name(&self) -> &str {
        "window_control_bring_to_top"
    }

    fn description(&self) -> &str {
        "Bring a window to the top (foreground)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to bring a window to the front of all other windows"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("微信".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_bring_to_top",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window brought to top".to_string()
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
        set_foreground_window(window_id)?;

        Ok("Window brought to top".to_string())
    }
}
