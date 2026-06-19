//! Image crop Driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, file_exists,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImageCropDriver;

#[async_trait::async_trait]
impl Driver for ImageCropDriver {
    fn name(&self) -> &str {
        "image_crop"
    }

    fn description(&self) -> &str {
        "Crop an image to a specified rectangular region"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to remove unwanted areas from an image. \
        Specify the crop region by coordinates (x, y, width, height)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_crop",
            "parameters": {
                "source": "/photos/family.jpg",
                "destination": "/photos/cropped.jpg",
                "x": 200,
                "y": 150,
                "width": 1000,
                "height": 800
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully cropped image from 1920x1080 to 1000x800 at position (200, 150)".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Media
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting image crop operation".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let x = parameters
            .get("x")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'x' parameter"))?
            as u32;
        let y = parameters
            .get("y")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'y' parameter"))?
            as u32;
        let width = parameters
            .get("width")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'width' parameter"))?
            as u32;
        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'height' parameter"))?
            as u32;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Source: {}, destination: {}, crop region: ({}, {}) {}x{}",
                    source, destination, x, y, width, height
                )),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Source file verified: {}", source)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let mut img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;
        let (orig_w, orig_h) = img.dimensions();
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Original dimensions: {}x{}", orig_w, orig_h)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        if x + width > orig_w {
            anyhow::bail!("Crop width exceeds image bounds");
        }
        if y + height > orig_h {
            anyhow::bail!("Crop height exceeds image bounds");
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Cropping image...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        let cropped = img.crop(x, y, width, height);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Saving cropped image...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        cropped
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save cropped image: {}", e))?;
        let result = format!(
            "Successfully cropped image from {}x{} to {}x{} at position ({}, {})",
            orig_w, orig_h, width, height, x, y
        );
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Result: {}", result)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("image_crop".to_string()),
                Some(result.clone()),
            );
        }
        Ok(result)
    }
}
