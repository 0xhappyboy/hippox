// keyboard_control/keyboard_control_modifier.rs
//! Keyboard modifier skill - control modifier keys separately
//!
//! This driver provides functionality to press or release individual
//! modifier keys for advanced keyboard operations.
use super::common::{get_key_code, send_key_down, send_key_up};
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
/// Driver for controlling modifier keys
#[derive(Debug)]
pub struct KeyboardControlModifierDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlModifierDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_modifier"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Control modifier keys (Shift, Ctrl, Alt, Win) separately for advanced combinations"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to press or release individual modifier keys. This is useful for complex keyboard operations."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "modifier".to_string(),
                param_type: "string".to_string(),
                description: "Modifier key (shift, ctrl, alt, win)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ctrl".to_string())),
                enum_values: Some(vec!["shift".to_string(), "ctrl".to_string(), "alt".to_string(), "win".to_string()]),
            },
            DriverParameter {
                name: "state".to_string(),
                param_type: "string".to_string(),
                description: "State: 'down' to press, 'up' to release".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("down".to_string())),
                enum_values: Some(vec!["down".to_string(), "up".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_modifier",
            "parameters": {
                "modifier": "ctrl",
                "state": "down"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Modifier ctrl set to down".to_string();
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
        debug!("Executing keyboard_control_modifier driver");
        let modifier = parameters.get("modifier").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'modifier' parameter");
            return crate::DriverError::missing_parameter("modifier");
        })?;
        let state = parameters.get("state").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'state' parameter");
            return crate::DriverError::missing_parameter("state");
        })?;
        debug!("Setting modifier {} to state: {}", modifier, state);
        let key_code = get_key_code(modifier).ok_or_else(|| {
            warn!("Unknown modifier: {}", modifier);
            return crate::DriverError::validation("modifier", format!("Unknown modifier: {}", modifier));
        })?;
        match state {
            "down" => {
                send_key_down(key_code).map_err(|e| {
                    debug!("Failed to send key down: {}", e);
                    return crate::DriverError::execution(format!("Failed to send key down: {}", e));
                })?;
            }
            "up" => {
                send_key_up(key_code).map_err(|e| {
                    debug!("Failed to send key up: {}", e);
                    return crate::DriverError::execution(format!("Failed to send key up: {}", e));
                })?;
            }
            _ => {
                warn!("Invalid state: {}. Must be 'down' or 'up'", state);
                return Err(crate::DriverError::invalid_enum_value("state", state.to_string(), vec!["down".to_string(), "up".to_string()]));
            }
        }
        info!("Modifier {} set to {}", modifier, state);
        return Ok(format!("Modifier {} set to {}", modifier, state));
    }
}
