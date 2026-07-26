//! Image info driver module
//!
//! This module provides functionality to extract metadata information from images
//! including dimensions, format, file size, and color type.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info};
/// Driver for getting image metadata information
#[derive(Debug)]
pub struct ImageInfoDriver;
#[async_trait::async_trait]
impl Driver for ImageInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_info";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get metadata information about an image (dimensions, format, file size)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to inspect an image's properties before processing it.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/image.jpg".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_info",
            "parameters": {
                "path": "/photos/landscape.jpg"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"dimensions":{"width":1920,"height":1080},"format":"JPEG","file_size_bytes":245760}"#.to_string();
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
        debug!("Executing image_info driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image info extraction".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Getting image info for: {}", path);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Image path: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        // Validate file exists
        if !file_exists(path) {
            return Err(DriverError::execution(format!("Image not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File verified: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        // Get file metadata
        let metadata = fs::metadata(path).map_err(|e| DriverError::execution(format!("Failed to read file metadata: {}", e)))?;
        let file_size_bytes = metadata.len();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File size: {} bytes", file_size_bytes)));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        // Open image to get dimensions
        use image::GenericImageView;
        let img = image::open(path).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", path, e)))?;
        let (width, height) = img.dimensions();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Image dimensions: {}x{}", width, height)));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        // Get format and color type
        let format = Path::new(path).extension().and_then(|ext| ext.to_str()).unwrap_or("unknown").to_uppercase();
        let color_type = match img.color() {
            image::ColorType::L8 => "Grayscale 8-bit",
            image::ColorType::La8 => "Grayscale with Alpha 8-bit",
            image::ColorType::Rgb8 => "RGB 8-bit",
            image::ColorType::Rgba8 => "RGBA 8-bit",
            _ => "Other",
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Format: {}, color type: {}", format, color_type)));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        // Build result
        let info = json!({
            "path": path,
            "dimensions": {
                "width": width,
                "height": height,
                "aspect_ratio": format!("{:.2}", width as f64 / height as f64)
            },
            "format": format,
            "file_size": {
                "bytes": file_size_bytes,
                "kb": file_size_bytes as f64 / 1024.0,
                "mb": file_size_bytes as f64 / (1024.0 * 1024.0)
            },
            "color_type": color_type,
            "total_pixels": width as u64 * height as u64
        });
        let result = serde_json::to_string_pretty(&info).map_err(|e| DriverError::execution(format!("Failed to serialize image info: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Image info extraction complete".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_info".to_string()), Some(result.clone()));
        }
        info!("Image info extraction completed: {}", path);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
