//! Mouse position get driver module
//!
//! This module provides functionality to get the current mouse cursor position.
use super::common::get_mouse_position;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting the current mouse position
#[derive(Debug)]
pub struct MouseControlPositionGetDriver;
#[async_trait::async_trait]
impl Driver for MouseControlPositionGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_position_get";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get the current mouse cursor position";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to read the current coordinates of the mouse cursor on screen";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_position_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Mouse position: x=500, y=300".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Mouse;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing mouse_control_position_get driver");
        let pos = get_mouse_position()?;
        let result = format!("Mouse position: x={}, y={}", pos.x, pos.y);
        info!("Mouse position retrieved: ({}, {})", pos.x, pos.y);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
