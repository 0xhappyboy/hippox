//! Display orientation set skill
//!
//! This driver provides functionality to set the display orientation
//! (landscape, portrait, landscape_flipped, portrait_flipped).
use super::common::set_orientation;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter,  },
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting display orientation
#[derive(Debug)]
pub struct DisplayControlOrientationSetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlOrientationSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_orientation_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the display orientation"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to rotate the screen orientation (landscape, portrait, etc.)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "orientation".to_string(),
            param_type: "string".to_string(),
            description: "Orientation: 'landscape', 'portrait', 'landscape_flipped', or 'portrait_flipped'".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("portrait".to_string())),
            enum_values: Some(vec!["landscape".to_string(), "portrait".to_string(), "landscape_flipped".to_string(), "portrait_flipped".to_string()]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_orientation_set",
            "parameters": {
                "orientation": "portrait"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display orientation set to portrait".to_string();
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
        debug!("Executing display_control_orientation_set driver");
        let orientation = parameters.get("orientation").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'orientation' parameter");
            return crate::DriverError::missing_parameter("orientation");
        })?;
        debug!("Setting orientation to '{}'", orientation);
        set_orientation(orientation, None).map_err(|e| {
            debug!("Failed to set orientation: {}", e);
            return crate::DriverError::execution(format!("Failed to set orientation: {}", e));
        })?;
        info!("Display orientation set to {}", orientation);
        return Ok(format!("Display orientation set to {}", orientation));
    }
}
