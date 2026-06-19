//! Window get process info skill

use super::common::find_window;
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
pub struct WindowControlGetProcessDriver;

#[async_trait::async_trait]
impl Driver for WindowControlGetProcessDriver {
    fn name(&self) -> &str {
        "window_control_get_process"
    }

    fn description(&self) -> &str {
        "Get the process name and PID of a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find which process owns a window"
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
            "action": "window_control_get_process",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Process: WeChat.exe (PID: 12345)".to_string()
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

        use super::common::list_windows;
        let windows = list_windows()?;
        let window = windows
            .iter()
            .find(|w| w.id == window_id)
            .ok_or_else(|| anyhow::anyhow!("Window not found"))?;

        Ok(format!(
            "Process: {} (PID: {})",
            window.process_name, window.pid
        ))
    }
}
