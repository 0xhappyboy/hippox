//! Screenshot Driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use image::{GenericImageView, ImageBuffer, Rgba};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ScreenshotDriver;

#[async_trait::async_trait]
impl Driver for ScreenshotDriver {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> &str {
        "Capture a screenshot of the entire screen or a region"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to capture screenshots. Specify region with x, y, width, height."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Output file path (PNG format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/screenshot.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate of capture region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate of capture region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Width of capture region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of capture region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "screenshot",
            "parameters": {
                "destination": "/screenshots/desktop.png"
            }
        })
    }

    fn example_output(&self) -> String {
        "Screenshot saved to /screenshots/desktop.png".to_string()
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
                Some("Starting screenshot capture".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let x = parameters
            .get("x")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32);
        let y = parameters
            .get("y")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32);
        let width = parameters
            .get("width")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Destination: {:?}, region: x={:?}, y={:?}, w={:?}, h={:?}",
                    destination, x, y, width, height
                )),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        use xcap::Monitor;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Getting monitors...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let monitors =
            Monitor::all().map_err(|e| anyhow::anyhow!("Failed to get monitors: {}", e))?;
        let monitor = monitors
            .first()
            .ok_or_else(|| anyhow::anyhow!("No monitor found"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Capturing screen...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let image = monitor
            .capture_image()
            .map_err(|e| anyhow::anyhow!("Failed to capture screen: {}", e))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Processing captured image...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        let result = if let (Some(x), Some(y), Some(w), Some(h)) = (x, y, width, height) {
            let (img_w, img_h) = image.dimensions();
            if x + w <= img_w && y + h <= img_h {
                image::imageops::crop(&mut image.clone(), x, y, w, h).to_image()
            } else {
                anyhow::bail!(
                    "Crop region out of bounds: image size {}x{}, region {}x{} at ({}, {})",
                    img_w,
                    img_h,
                    w,
                    h,
                    x,
                    y
                );
            }
        } else {
            image
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Saving screenshot...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(85), None);
        }
        result
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save screenshot: {}", e))?;
        let result_msg = format!("Screenshot saved to {}", destination);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Result: {}", result_msg)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("screenshot".to_string()),
                Some(result_msg.clone()),
            );
        }
        Ok(result_msg)
    }
}
