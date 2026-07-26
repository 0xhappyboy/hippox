// keyboard_control/keyboard_control_hotkey.rs
//! Keyboard hotkey skill - system-level hotkeys
//!
//! This driver provides functionality to send system-level hotkeys
//! that control the operating system.
use super::common::send_shortcut;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for sending system-level hotkeys
#[derive(Debug)]
pub struct KeyboardControlHotkeyDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlHotkeyDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_hotkey"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send system-level hotkeys (e.g., Win+R, Win+E, Alt+Tab)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to send system-wide hotkeys that control the operating system."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "hotkey".to_string(),
            param_type: "string".to_string(),
            description: "Hotkey combination (e.g., 'Win+R', 'Win+E', 'Alt+Tab')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Win+R".to_string())),
            enum_values: Some(vec![
                "Win+R".to_string(),
                "Win+E".to_string(),
                "Win+D".to_string(),
                "Win+L".to_string(),
                "Win+S".to_string(),
                "Alt+Tab".to_string(),
                "Ctrl+Alt+Delete".to_string(),
                "Alt+F4".to_string(),
            ]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_hotkey",
            "parameters": {
                "hotkey": "Win+R"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Hotkey sent: Win+R".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Keyboard;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing keyboard_control_hotkey driver");
        let hotkey = parameters.get("hotkey").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'hotkey' parameter");
            return crate::DriverError::missing_parameter("hotkey");
        })?;
        debug!("Sending hotkey: {}", hotkey);
        send_shortcut(hotkey).map_err(|e| {
            debug!("Failed to send hotkey: {}", e);
            return crate::DriverError::execution(format!("Failed to send hotkey: {}", e));
        })?;
        info!("Hotkey sent: {}", hotkey);
        return Ok(format!("Hotkey sent: {}", hotkey));
    }
}
