//! Window move skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{find_window, get_window_rect, set_window_pos};
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WindowControlMoveDriver;

#[async_trait::async_trait]
impl Driver for WindowControlMoveDriver {
    fn name(&self) -> &str {
        "window_control_move"
    }

    fn description(&self) -> &str {
        "Move a specified window to a new position"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to move a window to specific coordinates on the screen"
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
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "New X coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "New Y coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_move",
            "parameters": {
                "title": "微信",
                "x": 100,
                "y": 100
            }
        })
    }

    fn example_output(&self) -> String {
        "Window moved to (100, 100)".to_string()
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
        let x = parameters
            .get("x")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing x"))? as i32;
        let y = parameters
            .get("y")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing y"))? as i32;

        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;

        set_window_pos(window_id, x, y, rect.width, rect.height)?;

        Ok(format!("Window moved to ({}, {})", x, y))
    }
}
