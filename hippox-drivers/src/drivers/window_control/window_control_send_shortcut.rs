//! Window send shortcut driver
//!
//! This driver provides functionality to send a keyboard shortcut to a specified window.
use super::common::find_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for sending keyboard shortcuts
#[derive(Debug)]
pub struct WindowControlSendShortcutDriver;
#[async_trait::async_trait]
impl Driver for WindowControlSendShortcutDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_send_shortcut"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a keyboard shortcut to a specified window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to send shortcuts like Ctrl+C, Ctrl+V, Alt+Tab"
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
                name: "shortcut".to_string(),
                param_type: "string".to_string(),
                description: "Shortcut name (e.g., Ctrl+C, Ctrl+V, Ctrl+S, Alt+F4)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Ctrl+S".to_string())),
                enum_values: Some(vec![
                    "Ctrl+C".to_string(),
                    "Ctrl+V".to_string(),
                    "Ctrl+X".to_string(),
                    "Ctrl+Z".to_string(),
                    "Ctrl+Y".to_string(),
                    "Ctrl+S".to_string(),
                    "Ctrl+A".to_string(),
                    "Alt+F4".to_string(),
                    "Alt+Tab".to_string(),
                    "Enter".to_string(),
                    "Tab".to_string(),
                    "Escape".to_string(),
                    "Delete".to_string(),
                ]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_send_shortcut",
            "parameters": {
                "title": "记事本",
                "shortcut": "Ctrl+S"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Shortcut sent to window".to_string();
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
        debug!("Executing window_control_send_shortcut driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let shortcut = parameters.get("shortcut").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("shortcut"))?;
        info!("Sending shortcut: title={:?}, process={:?}, shortcut={}", title, process, shortcut);
        // Find window to activate
        if let Some(window_id) = find_window(title, process).ok() {
            info!("Found window ID: {}, will send shortcut", window_id);
            // TODO: Activate window and send shortcut using enigo or similar
        }
        // Platform-specific shortcut implementation
        // Use enigo or similar to send shortcut
        let _ = shortcut;
        return Ok("Shortcut sent to window (implementation pending)".to_string());
    }
}
