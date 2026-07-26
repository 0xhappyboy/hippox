// keyboard_control/keyboard_control_press.rs
//! Keyboard press skill - press and release a key
//!
//! This driver provides functionality to press and release a single
//! keyboard key.
use super::common::{get_key_code, send_key_press};
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
/// Driver for pressing a single key
#[derive(Debug)]
pub struct KeyboardControlPressDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlPressDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_press"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Press and release a keyboard key"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to press a single key (e.g., Enter, Space, A, 1). The key will be pressed and immediately released."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "key".to_string(),
            param_type: "string".to_string(),
            description: "Key to press (e.g., 'a', 'enter', 'space', 'f1', 'ctrl')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("enter".to_string())),
            enum_values: Some(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "enter".to_string(),
                "space".to_string(),
                "tab".to_string(),
                "esc".to_string(),
                "f1".to_string(),
                "f2".to_string(),
            ]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_press",
            "parameters": {
                "key": "enter"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Key pressed: enter".to_string();
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
        debug!("Executing keyboard_control_press driver");
        let key = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            return crate::DriverError::missing_parameter("key");
        })?;
        debug!("Pressing key: {}", key);
        let key_code = get_key_code(key).ok_or_else(|| {
            warn!("Unknown key: {}", key);
            return crate::DriverError::validation("key", format!("Unknown key: {}", key));
        })?;
        send_key_press(key_code).map_err(|e| {
            debug!("Failed to send key press: {}", e);
            return crate::DriverError::execution(format!("Failed to send key press: {}", e));
        })?;
        info!("Key pressed: {}", key);
        return Ok(format!("Key pressed: {}", key));
    }
}
