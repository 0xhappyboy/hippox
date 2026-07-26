//! Window find driver
//!
//! This driver provides functionality to find a window by title or process name.
use super::common::find_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for finding a window
#[derive(Debug)]
pub struct WindowControlFindDriver;
#[async_trait::async_trait]
impl Driver for WindowControlFindDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_find"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Find a window by title or process name"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get window information by searching by title or process"
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
                description: "Process name (e.g., WeChat.exe)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_find",
            "parameters": {
                "title": "微信"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Found window: 微信 (ID: 12345678)".to_string();
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
        debug!("Executing window_control_find driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        info!("Finding window: title={:?}, process={:?}", title, process);
        let window_id = find_window(title, process)?;
        use super::common::list_windows;
        let windows = list_windows()?;
        let window = windows.iter().find(|w| w.id == window_id).ok_or_else(|| DriverError::execution("Window not found"))?;
        info!("Found window: {} (ID: {}, Process: {})", window.title, window.id, window.process_name);
        return Ok(format!("Found window: {} (ID: {}, Process: {})", window.title, window.id, window.process_name));
    }
}
