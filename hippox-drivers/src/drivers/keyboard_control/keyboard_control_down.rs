// keyboard_control/keyboard_control_down.rs
//! Keyboard down skill - press and hold a key
//!
//! This driver provides functionality to press and hold a keyboard key
//! without releasing it.
use super::common::{get_key_code, send_key_down};
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
/// Driver for pressing and holding a key
#[derive(Debug)]
pub struct KeyboardControlDownDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlDownDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_down"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Press and hold a keyboard key (without releasing)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to hold down a key. The key will remain pressed until 'keyboard_control_up' is called."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "key".to_string(),
            param_type: "string".to_string(),
            description: "Key to press and hold (e.g., 'shift', 'ctrl', 'alt')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("shift".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_down",
            "parameters": {
                "key": "shift"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Key down: shift".to_string();
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
        debug!("Executing keyboard_control_down driver");
        let key = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            return crate::DriverError::missing_parameter("key");
        })?;
        debug!("Pressing and holding key: {}", key);
        let key_code = get_key_code(key).ok_or_else(|| {
            warn!("Unknown key: {}", key);
            return crate::DriverError::validation("key", format!("Unknown key: {}", key));
        })?;
        send_key_down(key_code).map_err(|e| {
            debug!("Failed to send key down: {}", e);
            return crate::DriverError::execution(format!("Failed to send key down: {}", e));
        })?;
        info!("Key down: {}", key);
        return Ok(format!("Key down: {}", key));
    }
}
