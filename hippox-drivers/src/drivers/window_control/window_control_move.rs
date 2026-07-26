//! Window move driver
//!
//! This driver provides functionality to move a specified window to a new position.
use super::common::{find_window, get_window_rect, set_window_pos};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for moving a window
#[derive(Debug)]
pub struct WindowControlMoveDriver;
#[async_trait::async_trait]
impl Driver for WindowControlMoveDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_move"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Move a specified window to a new position"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to move a window to specific coordinates on the screen"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_move",
            "parameters": {
                "title": "微信",
                "x": 100,
                "y": 100
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window moved to (100, 100)".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Window;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing window_control_move driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let x = parameters.get("x").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("x"))? as i32;
        let y = parameters.get("y").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("y"))? as i32;
        info!("Moving window: title={:?}, process={:?}, x={}, y={}", title, process, x, y);
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        set_window_pos(window_id, x, y, rect.width, rect.height)?;
        info!("Window moved to ({}, {}): ID={}", x, y, window_id);
        return Ok(format!("Window moved to ({}, {})", x, y));
    }
}
