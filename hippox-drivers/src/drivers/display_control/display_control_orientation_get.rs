//! Display orientation get skill
//!
//! This driver provides functionality to get the current display
//! orientation (landscape, portrait, etc.).
use super::common::get_orientation;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting display orientation
#[derive(Debug)]
pub struct DisplayControlOrientationGetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlOrientationGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_orientation_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the display orientation (landscape, portrait, etc.)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check if the display is in landscape or portrait mode."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_orientation_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display orientation: landscape".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Display;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing display_control_orientation_get driver");
        let orientation = get_orientation(None).map_err(|e| {
            debug!("Failed to get orientation: {}", e);
            return crate::DriverError::execution(format!("Failed to get orientation: {}", e));
        })?;
        info!("Display orientation: {}", orientation);
        return Ok(format!("Display orientation: {}", orientation));
    }
}
