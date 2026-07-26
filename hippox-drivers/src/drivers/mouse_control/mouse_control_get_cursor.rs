//! Mouse get cursor type driver module
//!
//! This module provides functionality to get the current cursor type.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use super::common::get_cursor_type;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for getting the current cursor type
#[derive(Debug)]
pub struct MouseControlGetCursorDriver;
#[async_trait::async_trait]
impl Driver for MouseControlGetCursorDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mouse_control_get_cursor";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get the current cursor type (arrow, hand, wait, etc.)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to determine what kind of cursor is currently displayed, which can indicate UI state.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mouse_control_get_cursor"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Cursor type: arrow".to_string();
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
        debug!("Executing mouse_control_get_cursor driver");
        let cursor_type = get_cursor_type()?;
        let result = format!("Cursor type: {}", cursor_type);
        info!("Cursor type retrieved: {}", cursor_type);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
