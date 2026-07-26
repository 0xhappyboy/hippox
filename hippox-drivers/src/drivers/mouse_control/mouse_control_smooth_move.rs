//! Mouse smooth move driver module
//!
//! This module provides functionality to move the mouse cursor smoothly
//! with acceleration and deceleration.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use super::common::smooth_move_to;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for smooth mouse movement
#[derive(Debug)]
pub struct MouseControlSmoothMoveDriver;
#[async_trait::async_trait]
impl Driver for MouseControlSmoothMoveDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_smooth_move";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Move mouse cursor smoothly to target with acceleration";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill for more natural-looking mouse movements. The cursor will accelerate and decelerate smoothly.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "Target X coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Target Y coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "duration_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Movement duration in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(200.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_smooth_move",
            "parameters": {
                "x": 500,
                "y": 300,
                "duration_ms": 300
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Mouse smoothly moved to (500, 300) in 300ms".to_string();
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
        debug!("Executing mouse_control_smooth_move driver");
        let x = parameters.get("x").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("x"))? as i32;
        let y = parameters.get("y").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("y"))? as i32;
        let duration_ms = parameters.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(200);
        debug!("Smooth moving to ({}, {}) in {}ms", x, y, duration_ms);
        smooth_move_to(x, y, duration_ms).await?;
        let result = format!("Mouse smoothly moved to ({}, {}) in {}ms", x, y, duration_ms);
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
