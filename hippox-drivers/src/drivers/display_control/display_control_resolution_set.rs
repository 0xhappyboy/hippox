//! Display resolution set skill
//!
//! This driver provides functionality to set the display resolution.
//! May cause temporary screen flicker on some systems.
use super::common::set_resolution;
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
/// Driver for setting display resolution
#[derive(Debug)]
pub struct DisplayControlResolutionSetDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlResolutionSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_resolution_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the display resolution"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to change the screen resolution. May cause temporary screen flicker."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Desired width in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1920.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Desired height in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1080.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "display_id".to_string(),
                param_type: "integer".to_string(),
                description: "Display ID (optional, uses primary if not specified)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_resolution_set",
            "parameters": {
                "width": 1920,
                "height": 1080
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Resolution set to 1920x1080".to_string();
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
        debug!("Executing display_control_resolution_set driver");
        let width = parameters.get("width").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'width' parameter");
            return crate::DriverError::missing_parameter("width");
        })? as u32;
        let height = parameters.get("height").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'height' parameter");
            return crate::DriverError::missing_parameter("height");
        })? as u32;
        let display_id = parameters.get("display_id").and_then(|v| v.as_u64()).map(|v| v as u32);
        debug!("Setting resolution to {}x{} for display {:?}", width, height, display_id);
        set_resolution(width, height, display_id).map_err(|e| {
            debug!("Failed to set resolution: {}", e);
            return crate::DriverError::execution(format!("Failed to set resolution: {}", e));
        })?;
        info!("Resolution set to {}x{}", width, height);
        return Ok(format!("Resolution set to {}x{}", width, height));
    }
}
