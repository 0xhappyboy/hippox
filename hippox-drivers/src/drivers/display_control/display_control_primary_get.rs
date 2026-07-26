//! Display primary get skill
//!
//! This driver provides functionality to get information about the
//! primary (main) display on the system.
use super::common::get_primary_display;
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
/// Driver for getting primary display information
#[derive(Debug)]
pub struct DisplayControlPrimaryGetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlPrimaryGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_primary_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get information about the primary display"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get details about the main monitor."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_primary_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Primary display: Primary Display (1920x1080, 60Hz)".to_string();
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
        debug!("Executing display_control_primary_get driver");
        let d = get_primary_display().map_err(|e| {
            debug!("Failed to get primary display: {}", e);
            return crate::DriverError::execution(format!("Failed to get primary display: {}", e));
        })?;
        info!("Primary display: {:?} ({:?}x{:?} @ {}Hz)", d.name, d.width, d.height, d.refresh_rate);
        return Ok(format!("Primary display: {} ({}x{} @ {}Hz)", d.name, d.width, d.height, d.refresh_rate));
    }
}
