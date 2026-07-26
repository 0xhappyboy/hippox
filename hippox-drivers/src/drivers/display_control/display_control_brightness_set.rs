//! Display brightness set skill
//!
//! This driver provides functionality to set the display brightness
//! level on supported systems (primarily laptops).
use super::common::set_brightness;
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
/// Driver for setting display brightness
#[derive(Debug)]
pub struct DisplayControlBrightnessSetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlBrightnessSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_brightness_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the display brightness level (laptops only)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to adjust screen brightness (0-100). Works on laptops, may not work on desktops."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "brightness".to_string(),
            param_type: "integer".to_string(),
            description: "Brightness level from 0 to 100".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(50.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_brightness_set",
            "parameters": {
                "brightness": 50
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display brightness set to 50%".to_string();
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
        debug!("Executing display_control_brightness_set driver");
        let brightness = parameters.get("brightness").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'brightness' parameter");
            return crate::DriverError::missing_parameter("brightness");
        })? as u32;
        let brightness = brightness.clamp(0, 100);
        debug!("Setting brightness to {}%", brightness);
        set_brightness(brightness).map_err(|e| {
            debug!("Failed to set brightness: {}", e);
            return crate::DriverError::execution(format!("Failed to set brightness: {}", e));
        })?;
        info!("Brightness set to {}%", brightness);
        return Ok(format!("Display brightness set to {}%", brightness));
    }
}
