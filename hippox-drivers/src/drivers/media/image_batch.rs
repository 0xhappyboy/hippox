//! Image batch conversion driver module
//!
//! This module provides functionality to batch convert all images in a directory
//! to a target format.
use super::common::get_format_from_extension;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info};
/// Driver for batch converting images in a directory
#[derive(Debug)]
pub struct ImageBatchConvertDriver;
#[async_trait::async_trait]
impl Driver for ImageBatchConvertDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_batch_convert";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Batch convert all images in a directory to a target format";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to convert multiple images at once. \
        Specify input directory, output directory, and target format.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "input_dir".to_string(),
                param_type: "string".to_string(),
                description: "Input directory containing images".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/photos/input".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "output_dir".to_string(),
                param_type: "string".to_string(),
                description: "Output directory for converted images".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/photos/output".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Target format: jpg, png, webp, bmp, gif".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("webp".to_string())),
                enum_values: Some(vec![
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "png".to_string(),
                    "webp".to_string(),
                    "bmp".to_string(),
                    "gif".to_string(),
                ]),
            },
            DriverParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Quality for lossy formats (1-100)".to_string(),
                required: false,
                default: Some(Value::Number(85.into())),
                example: Some(Value::Number(80.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_batch_convert",
            "parameters": {
                "input_dir": "/photos/raw",
                "output_dir": "/photos/webp",
                "format": "webp",
                "quality": 85
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully converted 15 images to webp".to_string();
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
        debug!("Executing image_batch_convert driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting batch image conversion".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let input_dir = parameters.get("input_dir").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("input_dir"))?;
        let output_dir = parameters.get("output_dir").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("output_dir"))?;
        let target_format = parameters.get("format").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("format"))?;
        debug!("Batch converting: input={}, output={}, format={}", input_dir, output_dir, target_format);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Input: {}, output: {}, format: {}", input_dir, output_dir, target_format)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let input_path = Path::new(input_dir);
        let output_path = Path::new(output_dir);
        if !input_path.exists() {
            return Err(DriverError::execution(format!("Input directory not found: {}", input_dir)));
        }
        if !output_path.exists() {
            fs::create_dir_all(output_path).map_err(|e| DriverError::execution(format!("Failed to create output directory: {}", e)))?;
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Created output directory: {}", output_dir)));
            }
        }
        let format = get_format_from_extension(&format!("dummy.{}", target_format))
            .ok_or_else(|| DriverError::execution(format!("Unsupported format: {}", target_format)))?;
        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "avif"];
        let mut converted = 0;
        let mut total = 0;
        if let Some(cb) = callback {
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        // Count total images to convert
        for entry in fs::read_dir(input_path).map_err(|e| DriverError::execution(format!("Failed to read input directory: {}", e)))? {
            let entry = entry.map_err(|e| DriverError::execution(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase());
            if let Some(ext) = ext {
                if image_extensions.contains(&ext.as_str()) {
                    total += 1;
                }
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Found {} images to convert", total)));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let mut processed = 0;
        for entry in fs::read_dir(input_path).map_err(|e| DriverError::execution(format!("Failed to read input directory: {}", e)))? {
            let entry = entry.map_err(|e| DriverError::execution(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase());
            if let Some(ext) = ext {
                if !image_extensions.contains(&ext.as_str()) {
                    continue;
                }
            }
            let file_name = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| DriverError::execution("Invalid file name".to_string()))?;
            let output_file = output_path.join(format!("{}.{}", file_name, target_format));
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Converting {}/{}: {}", processed + 1, total, path.display())));
                let progress = 40 + ((processed as f32 / total as f32) * 50.0) as u32;
                cb.on_progress(task_id.clone(), driver_index, Some(progress.min(95)), None);
            }
            match convert_image(&path, &output_file, format) {
                Ok(_) => {
                    converted += 1;
                    processed += 1;
                }
                Err(e) => {
                    debug!("Failed to convert {}: {}", path.display(), e);
                    if let Some(cb) = callback {
                        cb.on_log(task_id.clone(), driver_index, Some(format!("Failed to convert {}: {}", path.display(), e)));
                    }
                    processed += 1;
                }
            }
        }
        let result_msg = format!("Successfully converted {} images to {}", converted, target_format);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_batch_convert".to_string()), Some(result_msg.clone()));
        }
        info!("Batch conversion completed: {} images converted to {}", converted, target_format);
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("input_dir").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("input_dir"))?;
        parameters.get("output_dir").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("output_dir"))?;
        parameters.get("format").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("format"))?;
        return Ok(());
    }
}
/// Converts a single image to the target format
///
/// # Arguments
/// * `input` - Input image path
/// * `output` - Output image path
/// * `format` - Target image format
///
/// # Returns
/// * `DriverResult<()>` - Success or error
fn convert_image(input: &Path, output: &Path, format: image::ImageFormat) -> DriverResult<()> {
    let img = image::open(input).map_err(|e| DriverError::execution(format!("Failed to open {:?}: {}", input, e)))?;
    img.save_with_format(output, format).map_err(|e| DriverError::execution(format!("Failed to save {:?}: {}", output, e)))?;
    return Ok(());
}
