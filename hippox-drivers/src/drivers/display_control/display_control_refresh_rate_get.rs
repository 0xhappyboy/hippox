//! Display refresh rate get skill
//!
//! This driver provides functionality to get the current display
//! refresh rate in Hertz.
use super::common::get_refresh_rate;
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
/// Driver for getting display refresh rate
#[derive(Debug)]
pub struct DisplayControlRefreshRateGetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlRefreshRateGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_refresh_rate_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current display refresh rate in Hz"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the monitor's refresh rate (e.g., 60Hz, 144Hz)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_refresh_rate_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display refresh rate: 60 Hz".to_string();
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
        debug!("Executing display_control_refresh_rate_get driver");
        let rate = get_refresh_rate(None).map_err(|e| {
            debug!("Failed to get refresh rate: {}", e);
            return crate::DriverError::execution(format!("Failed to get refresh rate: {}", e));
        })?;
        info!("Display refresh rate: {} Hz", rate);
        return Ok(format!("Display refresh rate: {} Hz", rate));
    }
}
