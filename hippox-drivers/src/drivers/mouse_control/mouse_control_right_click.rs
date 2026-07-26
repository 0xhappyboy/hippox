//! Mouse right click driver module
//!
//! This module provides functionality to perform a right-click at the
//! current mouse position or specified coordinates.
use super::common::{MouseButton, get_mouse_position, mouse_click};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for performing a right mouse click
#[derive(Debug)]
pub struct MouseControlRightClickDriver;
#[async_trait::async_trait]
impl Driver for MouseControlRightClickDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_right_click";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Right-click at the current mouse position or specified coordinates";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to open context menus. Optionally specify x and y coordinates.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to right-click at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to right-click at".to_string(),
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
            "action": "mouse_control_right_click",
            "parameters": {
                "x": 500,
                "y": 300
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Right-clicked at (500, 300)".to_string();
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
        debug!("Executing mouse_control_right_click driver");
        let x = parameters.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
        let y = parameters.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
        let (click_x, click_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = get_mouse_position()?;
            (pos.x, pos.y)
        };
        debug!("Right-clicking at ({}, {})", click_x, click_y);
        mouse_click(MouseButton::Right, click_x, click_y)?;
        let result = format!("Right-clicked at ({}, {})", click_x, click_y);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
