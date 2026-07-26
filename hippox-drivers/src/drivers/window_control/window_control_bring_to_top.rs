//! Window bring to top driver
//!
//! This driver provides functionality to bring a window to the top (foreground).
use super::common::{find_window, set_foreground_window};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for bringing a window to the top
#[derive(Debug)]
pub struct WindowControlBringToTopDriver;
#[async_trait::async_trait]
impl Driver for WindowControlBringToTopDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_bring_to_top"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Bring a window to the top (foreground)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to bring a window to the front of all other windows"
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_bring_to_top",
            "parameters": {
                "title": "微信"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window brought to top".to_string();
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
        debug!("Executing window_control_bring_to_top driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        info!("Bringing window to top: title={:?}, process={:?}", title, process);
        let window_id = find_window(title, process)?;
        set_foreground_window(window_id)?;
        info!("Window brought to top: ID={}", window_id);
        return Ok("Window brought to top".to_string());
    }
}
