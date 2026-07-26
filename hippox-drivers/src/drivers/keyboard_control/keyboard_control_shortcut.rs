// keyboard_control/keyboard_control_shortcut.rs
//! Keyboard shortcut skill - send combination keys
//!
//! This driver provides functionality to send keyboard shortcuts
//! (combinations of keys like Ctrl+C, Ctrl+Shift+S).
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
/// Driver for sending keyboard shortcuts
#[derive(Debug)]
pub struct KeyboardControlShortcutDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlShortcutDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_shortcut"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send keyboard shortcut (combination of keys like Ctrl+C, Ctrl+Shift+S)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to send common keyboard shortcuts. Examples: Ctrl+C, Ctrl+V, Ctrl+Shift+S, Alt+F4, Ctrl+Alt+Delete"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "shortcut".to_string(),
            param_type: "string".to_string(),
            description: "Shortcut combination (e.g., 'Ctrl+C', 'Ctrl+Shift+S', 'Alt+F4')".to_string(),
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
                "Ctrl+Alt+Delete".to_string(),
                "Ctrl+Shift+Esc".to_string(),
                "Win+R".to_string(),
            ]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_shortcut",
            "parameters": {
                "shortcut": "Ctrl+S"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Shortcut sent: Ctrl+S".to_string();
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
        debug!("Executing keyboard_control_shortcut driver");
        let shortcut = parameters.get("shortcut").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'shortcut' parameter");
            return crate::DriverError::missing_parameter("shortcut");
        })?;
        debug!("Sending shortcut: {}", shortcut);
        send_shortcut(shortcut).map_err(|e| {
            debug!("Failed to send shortcut: {}", e);
            return crate::DriverError::execution(format!("Failed to send shortcut: {}", e));
        })?;
        info!("Shortcut sent: {}", shortcut);
        return Ok(format!("Shortcut sent: {}", shortcut));
    }
}
