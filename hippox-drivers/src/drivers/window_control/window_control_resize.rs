//! Window resize driver
//!
//! This driver provides functionality to resize a specified window.
use super::common::{find_window, get_window_rect, set_window_pos};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for resizing a window
#[derive(Debug)]
pub struct WindowControlResizeDriver;
#[async_trait::async_trait]
impl Driver for WindowControlResizeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_resize"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Resize a specified window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to change the size of a window"
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
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "New width in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "New height in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_resize",
            "parameters": {
                "title": "记事本",
                "width": 800,
                "height": 600
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window resized to 800x600".to_string();
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
        debug!("Executing window_control_resize driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let width = parameters.get("width").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("width"))? as u32;
        let height = parameters.get("height").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("height"))? as u32;
        info!("Resizing window: title={:?}, process={:?}, w={}, h={}", title, process, width, height);
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        set_window_pos(window_id, rect.x, rect.y, width, height)?;
        info!("Window resized to {}x{}: ID={}", width, height, window_id);
        return Ok(format!("Window resized to {}x{}", width, height));
    }
}
