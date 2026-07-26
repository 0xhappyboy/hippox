//! Display brightness get skill
//!
//! This driver provides functionality to get the current display
//! brightness level on supported systems (primarily laptops).
use super::common::get_brightness;
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
/// Driver for getting display brightness
#[derive(Debug)]
pub struct DisplayControlBrightnessGetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlBrightnessGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_brightness_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current display brightness level (laptops only)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the screen brightness (0-100). Works on laptops, may not work on desktops."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_brightness_get"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display brightness: 75%".to_string();
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
        debug!("Executing display_control_brightness_get driver");
        let brightness = get_brightness().map_err(|e| {
            debug!("Failed to get brightness: {}", e);
            return crate::DriverError::execution(format!("Failed to get brightness: {}", e));
        })?;
        info!("Current brightness: {}%", brightness);
        return Ok(format!("Display brightness: {}%", brightness));
    }
}
