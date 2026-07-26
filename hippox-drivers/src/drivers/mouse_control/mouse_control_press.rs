//! Mouse press driver module
//!
//! This module provides functionality to press and hold a mouse button
//! without releasing it.
use super::common::{MouseButton, get_mouse_position, mouse_press};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for pressing a mouse button
#[derive(Debug)]
pub struct MouseControlPressDriver;
#[async_trait::async_trait]
impl Driver for MouseControlPressDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_press";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Press and hold a mouse button (without releasing)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to hold down a mouse button. Use 'mouse_control_release' to release it.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "button".to_string(),
                param_type: "string".to_string(),
                description: "Mouse button: 'left', 'right', or 'middle'".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("left".to_string())),
                enum_values: Some(vec!["left".to_string(), "right".to_string(), "middle".to_string()]),
            },
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to press at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to press at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_press",
            "parameters": {
                "button": "left",
                "x": 500,
                "y": 300
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Mouse button left pressed".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Mouse;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing mouse_control_press driver");
        let button_str = parameters.get("button").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("button"))?;
        let button = match button_str {
            "left" => MouseButton::Left,
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => return Err(DriverError::validation("button", format!("Unknown button: {}", button_str))),
        };
        let x = parameters.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
        let y = parameters.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
        let (press_x, press_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = get_mouse_position()?;
            (pos.x, pos.y)
        };
        debug!("Pressing {} at ({}, {})", button_str, press_x, press_y);
        mouse_press(button, press_x, press_y)?;
        let result = format!("Mouse button {} pressed", button_str);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("button").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("button"))?;
        return Ok(());
    }
}
