//! Mouse scroll driver module
//!
//! This module provides functionality to scroll the mouse wheel.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use super::common::mouse_scroll;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for scrolling the mouse wheel
#[derive(Debug)]
pub struct MouseControlScrollDriver;
#[async_trait::async_trait]
impl Driver for MouseControlScrollDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_scroll";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Scroll the mouse wheel";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to scroll up or down. Positive delta scrolls up, negative scrolls down.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "delta".to_string(),
            param_type: "integer".to_string(),
            description: "Scroll amount (positive=up, negative=down). 120 is typical for one click.".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(120.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_scroll",
            "parameters": {
                "delta": 120
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Scrolled by 120".to_string();
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
        debug!("Executing mouse_control_scroll driver");
        let delta = parameters.get("delta").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("delta"))? as i32;
        debug!("Scrolling by {}", delta);
        mouse_scroll(delta)?;
        let result = format!("Scrolled by {}", delta);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("delta").and_then(|v| v.as_i64()).ok_or_else(|| DriverError::missing_parameter("delta"))?;
        return Ok(());
    }
}
