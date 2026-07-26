//! Mouse move to driver module
//!
//! This module provides functionality to move the mouse cursor to specified
//! absolute coordinates.
use super::common::set_mouse_position;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for moving the mouse to absolute coordinates
#[derive(Debug)]
pub struct MouseControlMoveToDriver;
#[async_trait::async_trait]
impl Driver for MouseControlMoveToDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_move_to";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Move mouse cursor to specified coordinates";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to move the mouse to an absolute position on the screen.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to move to".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to move to".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_move_to",
            "parameters": {
                "x": 500,
                "y": 300
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Mouse moved to (500, 300)".to_string();
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
        debug!("Executing mouse_control_move_to driver");
        let x = parameters.get("x").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("x"))? as i32;
        let y = parameters.get("y").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("y"))? as i32;
        debug!("Moving mouse to ({}, {})", x, y);
        set_mouse_position(x, y)?;
        let result = format!("Mouse moved to ({}, {})", x, y);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("x").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("x"))?;
        parameters.get("y").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("y"))?;
        return Ok(());
    }
}
