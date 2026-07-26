//! Image stitch driver module
//!
//! This module provides functionality to stitch multiple images together
//! horizontally or vertically.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
/// Driver for stitching images together
#[derive(Debug)]
pub struct ImageStitchDriver;
#[async_trait::async_trait]
impl Driver for ImageStitchDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_stitch";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Stitch multiple images together horizontally or vertically";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to combine multiple images into a single image. \
        Images will be resized to match dimensions if they differ.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "images".to_string(),
                param_type: "array".to_string(),
                description: "Array of image file paths to stitch".to_string(),
                required: true,
                default: None,
                example: Some(json!(["/path/to/1.jpg", "/path/to/2.jpg"])),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/stitched.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "direction".to_string(),
                param_type: "string".to_string(),
                description: "Stitch direction: 'horizontal' or 'vertical'".to_string(),
                required: false,
                default: Some(Value::String("horizontal".to_string())),
                example: Some(Value::String("vertical".to_string())),
                enum_values: Some(vec!["horizontal".to_string(), "vertical".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_stitch",
            "parameters": {
                "images": ["/photos/1.jpg", "/photos/2.jpg", "/photos/3.jpg"],
                "destination": "/photos/panorama.jpg",
                "direction": "horizontal"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully stitched 3 images horizontally".to_string();
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
        debug!("Executing image_stitch driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image stitch operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let images = parameters.get("images").and_then(|v| v.as_array()).ok_or_else(|| DriverError::invalid_type("images", "array", "other"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let direction = parameters.get("direction").and_then(|v| v.as_str()).unwrap_or("horizontal");
        if images.is_empty() {
            return Err(DriverError::execution("At least one image required".to_string()));
        }
        debug!("Stitching {} images, direction={}", images.len(), direction);
        if let Some(cb) = callback {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Destination: {}, direction: {}, image count: {}", destination, direction, images.len())),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        // Load all images
        use image::{GenericImageView, ImageBuffer, Rgba};
        let mut loaded_images = Vec::new();
        let mut max_width = 0;
        let mut max_height = 0;
        for (idx, img_path) in images.iter().enumerate() {
            let path = img_path.as_str().ok_or_else(|| DriverError::execution("Invalid image path".to_string()))?;
            if !file_exists(path) {
                return Err(DriverError::execution(format!("Image not found: {}", path)));
            }
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Loading image {}: {}", idx + 1, path)));
                cb.on_progress(task_id.clone(), driver_index, Some(20 + (idx as u32 * 5)), None);
            }
            let img = image::open(path).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", path, e)))?;
            let (w, h) = img.dimensions();
            max_width = max_width.max(w);
            max_height = max_height.max(h);
            loaded_images.push(img);
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Max dimensions: {}x{}", max_width, max_height)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Calculate output dimensions
        let (total_w, total_h) = if direction == "horizontal" {
            let total_w = loaded_images.iter().map(|img| img.width()).sum::<u32>();
            (total_w, max_height)
        } else {
            (max_width, loaded_images.iter().map(|img| img.height()).sum::<u32>())
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Output dimensions: {}x{}", total_w, total_h)));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        // Create output image
        let mut output = ImageBuffer::new(total_w, total_h);
        let mut offset_x = 0;
        let mut offset_y = 0;
        for (idx, img) in loaded_images.iter().enumerate() {
            let (w, h) = img.dimensions();
            // Resize to match target dimension if needed
            let scaled = if direction == "horizontal" {
                if h != max_height { img.resize(w * max_height / h, max_height, image::imageops::FilterType::Lanczos3) } else { img.clone() }
            } else {
                if w != max_width { img.resize(max_width, h * max_width / w, image::imageops::FilterType::Lanczos3) } else { img.clone() }
            };
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Stitching image {} at offset ({}, {})", idx + 1, offset_x, offset_y)));
                cb.on_progress(task_id.clone(), driver_index, Some(60 + (idx as u32 * 10)), None);
            }
            let (sw, sh) = scaled.dimensions();
            for y in 0..sh {
                for x in 0..sw {
                    let px = scaled.get_pixel(x, y);
                    output.put_pixel(offset_x + x, offset_y + y, px);
                }
            }
            if direction == "horizontal" {
                offset_x += sw;
            } else {
                offset_y += sh;
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving stitched image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(90), None);
        }
        let result = image::DynamicImage::ImageRgba8(output);
        result.save(destination).map_err(|e| DriverError::execution(format!("Failed to save stitched image: {}", e)))?;
        let result_msg = format!("Successfully stitched {} images {}", loaded_images.len(), direction);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_stitch".to_string()), Some(result_msg.clone()));
        }
        info!("Image stitch completed: {} images", loaded_images.len());
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("images").and_then(|v| v.as_array()).ok_or_else(|| DriverError::missing_parameter("images"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        return Ok(());
    }
}
