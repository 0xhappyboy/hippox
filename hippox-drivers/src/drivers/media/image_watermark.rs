//! Image watermark Driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, file_exists,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImageWatermarkDriver;

#[async_trait::async_trait]
impl Driver for ImageWatermarkDriver {
    fn name(&self) -> &str {
        "image_watermark"
    }

    fn description(&self) -> &str {
        "Add text or image watermark to an image"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to add watermarks to images. \
        You can add text watermarks or overlay another image as a watermark."
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
                description: "Destination file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to use as watermark (use with watermark_type=text)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("© 2024".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "watermark_image".to_string(),
                param_type: "string".to_string(),
                description: "Path to watermark image (use with watermark_type=image)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/logo.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "watermark_type".to_string(),
                param_type: "string".to_string(),
                description: "Type of watermark: 'text' or 'image'".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("text".to_string())),
                enum_values: Some(vec!["text".to_string(), "image".to_string()]),
            },
            DriverParameter {
                name: "position".to_string(),
                param_type: "string".to_string(),
                description: "Position: top-left, top-right, bottom-left, bottom-right, center"
                    .to_string(),
                required: false,
                default: Some(Value::String("bottom-right".to_string())),
                example: Some(Value::String("center".to_string())),
                enum_values: Some(vec![
                    "top-left".to_string(),
                    "top-right".to_string(),
                    "bottom-left".to_string(),
                    "bottom-right".to_string(),
                    "center".to_string(),
                ]),
            },
            DriverParameter {
                name: "opacity".to_string(),
                param_type: "number".to_string(),
                description: "Opacity of watermark (0.0 to 1.0)".to_string(),
                required: false,
                default: Some(json!(0.5)),
                example: Some(json!(0.7)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_watermark",
            "parameters": {
                "source": "/photos/photo.jpg",
                "destination": "/photos/watermarked.jpg",
                "text": "© 2024",
                "watermark_type": "text",
                "position": "bottom-right",
                "opacity": 0.7
            }
        })
    }

    fn example_output(&self) -> String {
        "Added text watermark to image".to_string()
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
                Some("Starting image watermark operation".to_string()),
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
        let watermark_type = parameters
            .get("watermark_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'watermark_type' parameter"))?;
        let position = parameters
            .get("position")
            .and_then(|v| v.as_str())
            .unwrap_or("bottom-right");
        let opacity = parameters
            .get("opacity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Source: {}, destination: {}, type: {}, position: {}, opacity: {}",
                    source, destination, watermark_type, position, opacity
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

        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Adding {} watermark...", watermark_type)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }

        let result = match watermark_type {
            "text" => {
                let text = parameters
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'text' parameter for text watermark")
                    })?;
                add_text_watermark(&img, text, position, opacity)?
            }
            "image" => {
                let watermark_path = parameters
                    .get("watermark_image")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'watermark_image' parameter"))?;
                if !file_exists(watermark_path) {
                    anyhow::bail!("Watermark image not found: {}", watermark_path);
                }
                let watermark = image::open(watermark_path)
                    .map_err(|e| anyhow::anyhow!("Failed to open watermark: {}", e))?;
                add_image_watermark(&img, &watermark, position, opacity)?
            }
            _ => anyhow::bail!("Unknown watermark type: {}", watermark_type),
        };

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Saving watermarked image...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }

        result
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save watermarked image: {}", e))?;

        let result_msg = format!("Added {} watermark to image", watermark_type);

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
                Some("image_watermark".to_string()),
                Some(result_msg.clone()),
            );
        }

        Ok(result_msg)
    }
}

fn add_text_watermark(
    img: &DynamicImage,
    text: &str,
    position: &str,
    opacity: f64,
) -> Result<DynamicImage> {
    let (w, h) = img.dimensions();
    let mut output: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            output.put_pixel(x, y, px);
        }
    }

    let (text_x, text_y) = calculate_position(w, h, 200, 50, position);
    let opacity_byte = (opacity * 255.0) as u8;

    for dy in 0..60 {
        for dx in 0..(text.len() as u32 * 12 + 20) {
            let x = text_x + dx;
            let y = text_y + dy;
            if x < w && y < h {
                let px = output.get_pixel_mut(x, y);
                px[0] = px[0].saturating_add((255 - px[0]) * opacity_byte / 255);
                px[1] = px[1].saturating_add((255 - px[1]) * opacity_byte / 255);
                px[2] = px[2].saturating_add((255 - px[2]) * opacity_byte / 255);
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

fn add_image_watermark(
    img: &DynamicImage,
    watermark: &DynamicImage,
    position: &str,
    opacity: f64,
) -> Result<DynamicImage> {
    let (w, h) = img.dimensions();
    let (ww, wh) = watermark.dimensions();
    let mut output: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            output.put_pixel(x, y, px);
        }
    }

    let (pos_x, pos_y) = calculate_position(w, h, ww, wh, position);

    for dy in 0..wh.min(h - pos_y) {
        for dx in 0..ww.min(w - pos_x) {
            let wx = pos_x + dx;
            let wy = pos_y + dy;
            let wp = watermark.get_pixel(dx, dy);
            let op = output.get_pixel_mut(wx, wy);
            if wp[3] > 0 {
                let alpha = wp[3] as f32 / 255.0 * opacity as f32;
                op[0] = (op[0] as f32 * (1.0 - alpha) + wp[0] as f32 * alpha) as u8;
                op[1] = (op[1] as f32 * (1.0 - alpha) + wp[1] as f32 * alpha) as u8;
                op[2] = (op[2] as f32 * (1.0 - alpha) + wp[2] as f32 * alpha) as u8;
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

fn calculate_position(
    total_w: u32,
    total_h: u32,
    item_w: u32,
    item_h: u32,
    position: &str,
) -> (u32, u32) {
    let margin = 20;
    match position {
        "top-left" => (margin, margin),
        "top-right" => (total_w - item_w - margin, margin),
        "bottom-left" => (margin, total_h - item_h - margin),
        "center" => ((total_w - item_w) / 2, (total_h - item_h) / 2),
        _ => (total_w - item_w - margin, total_h - item_h - margin),
    }
}
