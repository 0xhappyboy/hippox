//! Image format conversion driver module
//!
//! This module provides functionality to convert images from one format to another,
//! supporting PNG, JPEG, WebP, BMP, and GIF formats.
use super::common::get_format_from_extension;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
/// Driver for converting images between formats
#[derive(Debug)]
pub struct ImageConvertDriver;
#[async_trait::async_trait]
impl Driver for ImageConvertDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_convert";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Convert an image from one format to another";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to change an image's file format. \
        Quality parameter applies to JPEG and WebP outputs.";
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
                example: Some(Value::String("/path/to/image.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path (extension determines output format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/image.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Quality for lossy formats (JPEG/WebP), 1-100".to_string(),
                required: false,
                default: Some(Value::Number(85.into())),
                example: Some(Value::Number(90.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_convert",
            "parameters": {
                "source": "/photos/screenshot.png",
                "destination": "/photos/screenshot.jpg",
                "quality": 85
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully converted image from PNG to JPEG (quality: 85)".to_string();
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
        debug!("Executing image_convert driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image format conversion".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        debug!("Converting image: source={}, dest={}", source, destination);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Source: {}, destination: {}", source, destination)));
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
        // Open source image
        let img = image::open(source).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", source, e)))?;
        let source_format = Path::new(source).extension().and_then(|ext| ext.to_str()).unwrap_or("unknown").to_lowercase();
        let dest_ext = Path::new(destination).extension().and_then(|ext| ext.to_str()).unwrap_or("").to_lowercase();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Converting from {} to {}", source_format, dest_ext)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Get target format
        let format =
            get_format_from_extension(destination).ok_or_else(|| DriverError::execution(format!("Unsupported output format: {}", dest_ext)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving converted image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        // Save with format
        img.save_with_format(destination, format).map_err(|e| DriverError::execution(format!("Failed to save image: {}", e)))?;
        let result = format!("Successfully converted image from {} to {}", source_format.to_uppercase(), dest_ext.to_uppercase());
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_convert".to_string()), Some(result.clone()));
        }
        info!("Image conversion completed: {} -> {}", source_format, dest_ext);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        return Ok(());
    }
}
