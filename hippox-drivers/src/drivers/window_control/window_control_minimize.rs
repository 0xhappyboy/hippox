//! Window minimize driver
//!
//! This driver provides functionality to minimize a specified window.
use super::common::{find_window, show_window};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for minimizing a window
#[derive(Debug)]
pub struct WindowControlMinimizeDriver;
#[async_trait::async_trait]
impl Driver for WindowControlMinimizeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_minimize"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Minimize a specified window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to minimize a window by title or process name"
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
            "action": "window_control_minimize",
            "parameters": {
                "title": "微信"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window minimized".to_string();
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
        debug!("Executing window_control_minimize driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        info!("Minimizing window: title={:?}, process={:?}", title, process);
        let window_id = find_window(title, process)?;
        #[cfg(target_os = "windows")]
        {
            show_window(window_id, 6)?; // SW_MINIMIZE = 6
            info!("Window minimized: ID={}", window_id);
        }
        #[cfg(target_os = "linux")]
        {
            show_window(window_id, 6)?;
            info!("Window minimized: ID={}", window_id);
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            info!("Minimize not implemented on this platform");
            return Err(DriverError::execution("Minimize not implemented on this platform"));
        }
        return Ok("Window minimized".to_string());
    }
}
