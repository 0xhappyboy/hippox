//! Image watermark driver module
//!
//! This module provides functionality to add text or image watermarks to images.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for adding watermarks to images
#[derive(Debug)]
pub struct ImageWatermarkDriver;
#[async_trait::async_trait]
impl Driver for ImageWatermarkDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_watermark";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Add text or image watermark to an image";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to add watermarks to images. \
        You can add text watermarks or overlay another image as a watermark.";
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
                description: "Position: top-left, top-right, bottom-left, bottom-right, center".to_string(),
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_watermark",
            "parameters": {
                "source": "/photos/photo.jpg",
                "destination": "/photos/watermarked.jpg",
                "text": "© 2024",
                "watermark_type": "text",
                "position": "bottom-right",
                "opacity": 0.7
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Added text watermark to image".to_string();
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
        debug!("Executing image_watermark driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image watermark operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let watermark_type =
            parameters.get("watermark_type").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("watermark_type"))?;
        let position = parameters.get("position").and_then(|v| v.as_str()).unwrap_or("bottom-right");
        let opacity = parameters.get("opacity").and_then(|v| v.as_f64()).unwrap_or(0.5).clamp(0.0, 1.0);
        debug!("Adding watermark: source={}, type={}, position={}, opacity={}", source, watermark_type, position, opacity);
        if let Some(cb) = callback {
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
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Adding {} watermark...", watermark_type)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Apply watermark based on type
        let result = match watermark_type {
            "text" => {
                let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
                add_text_watermark(&img, text, position, opacity)?
            }
            "image" => {
                let watermark_path =
                    parameters.get("watermark_image").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("watermark_image"))?;
                if !file_exists(watermark_path) {
                    return Err(DriverError::execution(format!("Watermark image not found: {}", watermark_path)));
                }
                let watermark = image::open(watermark_path).map_err(|e| DriverError::execution(format!("Failed to open watermark: {}", e)))?;
                add_image_watermark(&img, &watermark, position, opacity)?
            }
            _ => return Err(DriverError::execution(format!("Unknown watermark type: {}", watermark_type))),
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving watermarked image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        result.save(destination).map_err(|e| DriverError::execution(format!("Failed to save watermarked image: {}", e)))?;
        let result_msg = format!("Added {} watermark to image", watermark_type);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_watermark".to_string()), Some(result_msg.clone()));
        }
        info!("Image watermark applied: {}", watermark_type);
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        parameters.get("watermark_type").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("watermark_type"))?;
        return Ok(());
    }
}
/// Adds a text watermark to an image
///
/// # Arguments
/// * `img` - Source image
/// * `text` - Text to add as watermark
/// * `position` - Position of the watermark
/// * `opacity` - Opacity of the watermark
///
/// # Returns
/// * `DriverResult<image::DynamicImage>` - Watermarked image
fn add_text_watermark(img: &image::DynamicImage, text: &str, position: &str, opacity: f64) -> DriverResult<image::DynamicImage> {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    // Copy original image
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            output.put_pixel(x, y, px);
        }
    }
    let (text_x, text_y) = calculate_position(w, h, 200, 50, position);
    let opacity_byte = (opacity * 255.0) as u8;
    // Simple text rendering (placeholder - uses a colored block)
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
    return Ok(image::DynamicImage::ImageRgba8(output));
}
/// Adds an image watermark to an image
///
/// # Arguments
/// * `img` - Source image
/// * `watermark` - Watermark image
/// * `position` - Position of the watermark
/// * `opacity` - Opacity of the watermark
///
/// # Returns
/// * `DriverResult<image::DynamicImage>` - Watermarked image
fn add_image_watermark(
    img: &image::DynamicImage,
    watermark: &image::DynamicImage,
    position: &str,
    opacity: f64,
) -> DriverResult<image::DynamicImage> {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let (ww, wh) = watermark.dimensions();
    let mut output: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    // Copy original image
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            output.put_pixel(x, y, px);
        }
    }
    let (pos_x, pos_y) = calculate_position(w, h, ww, wh, position);
    // Composite watermark
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
    return Ok(image::DynamicImage::ImageRgba8(output));
}
/// Calculates the position for a watermark
///
/// # Arguments
/// * `total_w` - Total image width
/// * `total_h` - Total image height
/// * `item_w` - Watermark width
/// * `item_h` - Watermark height
/// * `position` - Position string
///
/// # Returns
/// * `(u32, u32)` - X and Y coordinates
fn calculate_position(total_w: u32, total_h: u32, item_w: u32, item_h: u32, position: &str) -> (u32, u32) {
    let margin = 20;
    match position {
        "top-left" => (margin, margin),
        "top-right" => (total_w - item_w - margin, margin),
        "bottom-left" => (margin, total_h - item_h - margin),
        "center" => ((total_w - item_w) / 2, (total_h - item_h) / 2),
        _ => (total_w - item_w - margin, total_h - item_h - margin),
    }
}
