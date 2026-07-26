//! Mouse move relative driver module
//!
//! This module provides functionality to move the mouse cursor relative
//! to its current position.
use super::common::{get_mouse_position, set_mouse_position};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for moving the mouse relative to current position
#[derive(Debug)]
pub struct MouseControlMoveRelativeDriver;
#[async_trait::async_trait]
impl Driver for MouseControlMoveRelativeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_move_relative";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Move mouse cursor relative to current position";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to move the mouse by a delta from its current position.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "dx".to_string(),
                param_type: "integer".to_string(),
                description: "Delta X to move (positive=right, negative=left)".to_string(),
                required: true,
                default: None,
                example: Some(json!(100)),
                enum_values: None,
            },
            DriverParameter {
                name: "dy".to_string(),
                param_type: "integer".to_string(),
                description: "Delta Y to move (positive=down, negative=up)".to_string(),
                required: true,
                default: None,
                example: Some(json!(-50)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_move_relative",
            "parameters": {
                "dx": 100,
                "dy": -50
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Mouse moved relative by (100, -50)".to_string();
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
        debug!("Executing mouse_control_move_relative driver");
        let dx = parameters.get("dx").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("dx"))? as i32;
        let dy = parameters.get("dy").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("dy"))? as i32;
        let current = get_mouse_position()?;
        let new_x = current.x + dx;
        let new_y = current.y + dy;
        debug!("Moving mouse from ({}, {}) by ({}, {}) to ({}, {})", current.x, current.y, dx, dy, new_x, new_y);
        set_mouse_position(new_x, new_y)?;
        let result = format!("Mouse moved relative by ({}, {})", dx, dy);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("dx").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("dx"))?;
        parameters.get("dy").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("dy"))?;
        return Ok(());
    }
}
