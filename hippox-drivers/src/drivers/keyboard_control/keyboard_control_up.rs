// keyboard_control/keyboard_control_up.rs
//! Keyboard up skill - release a held key
//!
//! This driver provides functionality to release a keyboard key
//! that was previously held down.
use super::common::{get_key_code, send_key_up};
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
/// Driver for releasing a held key
#[derive(Debug)]
pub struct KeyboardControlUpDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlUpDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_up"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Release a keyboard key that was being held down"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to release a key that was previously pressed with 'keyboard_control_down'."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "key".to_string(),
            param_type: "string".to_string(),
            description: "Key to release (e.g., 'shift', 'ctrl', 'alt')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("shift".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_up",
            "parameters": {
                "key": "shift"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Key up: shift".to_string();
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
        debug!("Executing keyboard_control_up driver");
        let key = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            return crate::DriverError::missing_parameter("key");
        })?;
        debug!("Releasing key: {}", key);
        let key_code = get_key_code(key).ok_or_else(|| {
            warn!("Unknown key: {}", key);
            return crate::DriverError::validation("key", format!("Unknown key: {}", key));
        })?;
        send_key_up(key_code).map_err(|e| {
            debug!("Failed to send key up: {}", e);
            return crate::DriverError::execution(format!("Failed to send key up: {}", e));
        })?;
        info!("Key up: {}", key);
        return Ok(format!("Key up: {}", key));
    }
}
