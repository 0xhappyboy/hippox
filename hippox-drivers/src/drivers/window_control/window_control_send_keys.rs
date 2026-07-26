//! Window send keys driver
//!
//! This driver provides functionality to send keyboard input to a specified window.
use super::common::find_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for sending keys to a window
#[derive(Debug)]
pub struct WindowControlSendKeysDriver;
#[async_trait::async_trait]
impl Driver for WindowControlSendKeysDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_send_keys"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send keyboard input to a specified window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to type text into a window"
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
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to type".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_send_keys",
            "parameters": {
                "title": "记事本",
                "text": "Hello World"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Text sent to window".to_string();
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
        debug!("Executing window_control_send_keys driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        info!("Sending keys: title={:?}, process={:?}, text_len={}", title, process, text.len());
        // Find window to activate
        if let Some(window_id) = find_window(title, process).ok() {
            info!("Found window ID: {}, will send keys", window_id);
            // TODO: Activate window and send keys using enigo or similar
        }
        // Use enigo or similar to type
        // For now, placeholder
        info!("Text: {}", text);
        return Ok("Text sent to window (implementation pending)".to_string());
    }
}
