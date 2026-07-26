//! Window close driver
//!
//! This driver provides functionality to close a specified window (graceful close).
use super::common::{close_window, find_window};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for closing a window
#[derive(Debug)]
pub struct WindowControlCloseDriver;
#[async_trait::async_trait]
impl Driver for WindowControlCloseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_close"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Close a specified window (graceful close)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to close a window by title or process name"
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_close",
            "parameters": {
                "title": "计算器"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window closed".to_string();
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
        debug!("Executing window_control_close driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        info!("Closing window: title={:?}, process={:?}", title, process);
        let window_id = find_window(title, process)?;
        close_window(window_id)?;
        info!("Window closed: ID={}", window_id);
        return Ok("Window closed".to_string());
    }
}
