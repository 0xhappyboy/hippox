//! Mouse double click driver module
//!
//! This module provides functionality to perform a double-click at the
//! current mouse position or specified coordinates.
use super::common::{MouseButton, get_mouse_position, mouse_double_click};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for performing a mouse double-click
#[derive(Debug)]
pub struct MouseControlDoubleClickDriver;
#[async_trait::async_trait]
impl Driver for MouseControlDoubleClickDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_double_click";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Double-click at the current mouse position or specified coordinates";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to perform a double-click. Optionally specify x and y coordinates.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to double-click at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to double-click at".to_string(),
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
            "action": "mouse_control_double_click",
            "parameters": {
                "x": 500,
                "y": 300
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Mouse double-clicked at (500, 300)".to_string();
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
        debug!("Executing mouse_control_double_click driver");
        let x = parameters.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
        let y = parameters.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
        let (click_x, click_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = get_mouse_position()?;
            (pos.x, pos.y)
        };
        debug!("Double-clicking at ({}, {})", click_x, click_y);
        mouse_double_click(MouseButton::Left, click_x, click_y)?;
        let result = format!("Mouse double-clicked at ({}, {})", click_x, click_y);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
