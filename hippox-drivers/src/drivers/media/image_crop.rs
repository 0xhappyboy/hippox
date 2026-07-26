//! Image crop driver module
//!
//! This module provides functionality to crop images to a specified rectangular region.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
/// Driver for cropping images
#[derive(Debug)]
pub struct ImageCropDriver;
#[async_trait::async_trait]
impl Driver for ImageCropDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_crop";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Crop an image to a specified rectangular region";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to remove unwanted areas from an image. \
        Specify the crop region by coordinates (x, y, width, height).";
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
                description: "Destination file path for cropped image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate of the top-left corner".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate of the top-left corner".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Width of the crop region in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of the crop region in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_crop",
            "parameters": {
                "source": "/photos/family.jpg",
                "destination": "/photos/cropped.jpg",
                "x": 200,
                "y": 150,
                "width": 1000,
                "height": 800
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully cropped image from 1920x1080 to 1000x800 at position (200, 150)".to_string();
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
        debug!("Executing image_crop driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image crop operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let x = parameters.get("x").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("x"))? as u32;
        let y = parameters.get("y").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("y"))? as u32;
        let width = parameters.get("width").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("width"))? as u32;
        let height = parameters.get("height").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("height"))? as u32;
        debug!("Cropping image: source={}, region=({},{}){}x{}", source, x, y, width, height);
        if let Some(cb) = callback {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Source: {}, destination: {}, crop region: ({}, {}) {}x{}", source, destination, x, y, width, height)),
            );
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
        // Open and crop image
        use image::GenericImageView;
        let mut img = image::open(source).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", source, e)))?;
        let (orig_w, orig_h) = img.dimensions();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Original dimensions: {}x{}", orig_w, orig_h)));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        // Validate crop bounds
        if x + width > orig_w {
            return Err(DriverError::execution(format!("Crop width exceeds image bounds: x={}, width={}, image_width={}", x, width, orig_w)));
        }
        if y + height > orig_h {
            return Err(DriverError::execution(format!("Crop height exceeds image bounds: y={}, height={}, image_height={}", y, height, orig_h)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Cropping image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        let cropped = img.crop(x, y, width, height);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving cropped image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        cropped.save(destination).map_err(|e| DriverError::execution(format!("Failed to save cropped image: {}", e)))?;
        let result = format!("Successfully cropped image from {}x{} to {}x{} at position ({}, {})", orig_w, orig_h, width, height, x, y);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_crop".to_string()), Some(result.clone()));
        }
        info!("Image crop completed: {}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        parameters.get("x").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("x"))?;
        parameters.get("y").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("y"))?;
        parameters.get("width").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("width"))?;
        parameters.get("height").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("height"))?;
        return Ok(());
    }
}
