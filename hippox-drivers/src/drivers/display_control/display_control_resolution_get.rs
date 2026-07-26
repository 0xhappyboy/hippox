//! Display resolution get skill
//!
//! This driver provides functionality to get the current display
//! resolution (width x height) in pixels.
use super::common::get_resolution;
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
/// Driver for getting display resolution
#[derive(Debug)]
pub struct DisplayControlResolutionGetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlResolutionGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_resolution_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current display resolution"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the width and height of the primary display."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_resolution_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current resolution: 1920x1080".to_string();
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
        debug!("Executing display_control_resolution_get driver");
        let (width, height) = get_resolution(None).map_err(|e| {
            debug!("Failed to get resolution: {}", e);
            return crate::DriverError::execution(format!("Failed to get resolution: {}", e));
        })?;
        info!("Current resolution: {}x{}", width, height);
        return Ok(format!("Current resolution: {}x{}", width, height));
    }
}
