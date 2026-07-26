//! Image rotate driver module
//!
//! This module provides functionality to rotate images by 90, 180, or 270 degrees.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
/// Driver for rotating images
#[derive(Debug)]
pub struct ImageRotateDriver;
#[async_trait::async_trait]
impl Driver for ImageRotateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_rotate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Rotate an image by 90, 180, or 270 degrees";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to rotate images that are oriented incorrectly.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/input.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for rotated image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "angle".to_string(),
                param_type: "integer".to_string(),
                description: "Rotation angle in degrees: 90, 180, 270".to_string(),
                required: false,
                default: Some(Value::Number(90.into())),
                example: Some(Value::Number(180.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_rotate",
            "parameters": {
                "source": "/photos/portrait.jpg",
                "destination": "/photos/rotated.jpg",
                "angle": 90
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully rotated image by 90 degrees".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Media;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing image_rotate driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image rotation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let angle = parameters.get("angle").and_then(|v| v.as_u64()).unwrap_or(90) as u32;
        debug!("Rotating image: source={}, angle={}", source, angle);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Source: {}, destination: {}, angle: {}", source, destination, angle)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        // Validate source file exists
        if !file_exists(source) {
            return Err(DriverError::execution(format!("Source image not found: {}", source)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Source file verified: {}", source)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        // Open image
        let img = image::open(source).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", source, e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Rotating image by {} degrees...", angle)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Apply rotation
        let rotated = match angle % 360 {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            0 => img,
            _ => return Err(DriverError::execution(format!("Unsupported angle {}. Supported: 90, 180, 270", angle))),
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving rotated image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        rotated.save(destination).map_err(|e| DriverError::execution(format!("Failed to save rotated image: {}", e)))?;
        let result = format!("Successfully rotated image by {} degrees", angle);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_rotate".to_string()), Some(result.clone()));
        }
        info!("Image rotation completed: {}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        return Ok(());
    }
}
