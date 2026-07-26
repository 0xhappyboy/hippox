//! Window restore driver
//!
//! This driver provides functionality to restore a minimized or maximized window.
use super::common::{find_window, show_window};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for restoring a window
#[derive(Debug)]
pub struct WindowControlRestoreDriver;
#[async_trait::async_trait]
impl Driver for WindowControlRestoreDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_restore"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Restore a minimized or maximized window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to restore a window from minimized or maximized state"
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
            "action": "window_control_restore",
            "parameters": {
                "title": "微信"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window restored".to_string();
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
        debug!("Executing window_control_restore driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        info!("Restoring window: title={:?}, process={:?}", title, process);
        let window_id = find_window(title, process)?;
        #[cfg(target_os = "windows")]
        {
            show_window(window_id, 9)?; // SW_RESTORE = 9
            info!("Window restored: ID={}", window_id);
        }
        #[cfg(target_os = "linux")]
        {
            show_window(window_id, 9)?;
            info!("Window restored: ID={}", window_id);
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            info!("Restore not implemented on this platform");
            return Err(DriverError::execution("Restore not implemented on this platform"));
        }
        return Ok("Window restored".to_string());
    }
}
