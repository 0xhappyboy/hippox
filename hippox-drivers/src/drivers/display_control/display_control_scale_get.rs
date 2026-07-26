//! Display scale get skill
//!
//! This driver provides functionality to get the DPI scaling factor
//! for the display (e.g., 1.0 for 100%, 1.5 for 150%).
use super::common::get_scale;
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
/// Driver for getting display scaling factor
#[derive(Debug)]
pub struct DisplayControlScaleGetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlScaleGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_scale_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the display scaling factor"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the DPI scaling factor (e.g., 1.0 for 100%, 1.5 for 150%)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_scale_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display scale: 1.5x".to_string();
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
        debug!("Executing display_control_scale_get driver");
        let scale = get_scale(None).map_err(|e| {
            debug!("Failed to get scale: {}", e);
            return crate::DriverError::execution(format!("Failed to get scale: {}", e));
        })?;
        info!("Display scale: {:.1}x", scale);
        return Ok(format!("Display scale: {:.1}x", scale));
    }
}
