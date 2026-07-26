//! Image compression driver module
//!
//! This module provides functionality to compress images with configurable
//! quality settings and optional resizing to reduce file size.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info};
/// Driver for compressing images to reduce file size
#[derive(Debug)]
pub struct ImageCompressDriver;
#[async_trait::async_trait]
impl Driver for ImageCompressDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_compress";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Compress an image to reduce file size with configurable quality";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to reduce image file sizes for web optimization. \
        Lower quality = smaller file size but more artifacts. Quality 70-85 is usually a good balance.";
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
                example: Some(Value::String("/path/to/large_image.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for compressed image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/compressed.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Compression quality (1-100). Higher = better quality".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(75.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "max_width".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum width (optional). Image will be scaled down proportionally".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1920.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "max_height".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum height (optional). Image will be scaled down proportionally".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1080.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_compress",
            "parameters": {
                "source": "/uploads/photo.jpg",
                "destination": "/uploads/photo_compressed.jpg",
                "quality": 80,
                "max_width": 1920
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Compressed image: 2.5MB -> 850KB (66.0% reduction)".to_string();
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
        debug!("Executing image_compress driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image compression".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let quality = parameters.get("quality").and_then(|v| v.as_u64()).unwrap_or(80) as u8;
        let max_width = parameters.get("max_width").and_then(|v| v.as_u64()).map(|w| w as u32);
        let max_height = parameters.get("max_height").and_then(|v| v.as_u64()).map(|h| h as u32);
        debug!("Compressing image: source={}, dest={}, quality={}", source, destination, quality);
        if let Some(cb) = callback {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Source: {}, destination: {}, quality: {}, max_width: {:?}, max_height: {:?}",
                    source, destination, quality, max_width, max_height
                )),
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
        let original_size = fs::metadata(source).map_err(|e| DriverError::execution(format!("Failed to read source file metadata: {}", e)))?.len();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Original file size: {} bytes", original_size)));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        // Open and process image
        use image::GenericImageView;
        let img = image::open(source).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", source, e)))?;
        let mut processed = img;
        // Resize if max dimensions are specified
        if let (Some(max_w), Some(max_h)) = (max_width, max_height) {
            let (w, h) = processed.dimensions();
            if w > max_w || h > max_h {
                let ratio = (w as f32 / h as f32).min(max_w as f32 / max_h as f32);
                let new_w = if w > max_w { max_w } else { w };
                let new_h = (new_w as f32 / ratio) as u32;
                processed = processed.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
                if let Some(cb) = callback {
                    cb.on_log(task_id.clone(), driver_index, Some(format!("Resized to {}x{}", new_w, new_h)));
                }
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Compressing image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        // Save with compression
        let dest_ext = Path::new(destination).extension().and_then(|ext| ext.to_str()).unwrap_or("").to_lowercase();
        match dest_ext.as_str() {
            "jpg" | "jpeg" => {
                let mut bytes = Vec::new();
                let cursor = std::io::Cursor::new(&mut bytes);
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, quality);
                processed.write_with_encoder(encoder).map_err(|e| DriverError::execution(format!("Failed to encode JPEG: {}", e)))?;
                fs::write(destination, bytes).map_err(|e| DriverError::execution(format!("Failed to write JPEG: {}", e)))?;
            }
            "webp" => {
                processed
                    .save_with_format(destination, image::ImageFormat::WebP)
                    .map_err(|e| DriverError::execution(format!("Failed to save WebP: {}", e)))?;
            }
            _ => {
                processed.save(destination).map_err(|e| DriverError::execution(format!("Failed to save image: {}", e)))?;
            }
        }
        let compressed_size =
            fs::metadata(destination).map_err(|e| DriverError::execution(format!("Failed to read compressed file metadata: {}", e)))?.len();
        let reduction = if original_size > 0 { ((original_size - compressed_size) as f64 / original_size as f64) * 100.0 } else { 0.0 };
        let result = format!(
            "Compressed image: {:.2}MB -> {:.2}MB ({:.1}% reduction) at quality {}",
            original_size as f64 / (1024.0 * 1024.0),
            compressed_size as f64 / (1024.0 * 1024.0),
            reduction,
            quality
        );
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_compress".to_string()), Some(result.clone()));
        }
        info!("Image compression completed: {}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        return Ok(());
    }
}
