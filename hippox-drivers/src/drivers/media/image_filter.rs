//! Image filter driver module
//!
//! This module provides functionality to apply various filters to images
//! including grayscale, blur, sharpen, brightness, contrast, saturation,
//! sepia, and invert.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for applying filters to images
#[derive(Debug)]
pub struct ImageFilterDriver;
#[async_trait::async_trait]
impl Driver for ImageFilterDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_filter";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Apply various filters to an image (grayscale, blur, sharpen, brightness, contrast, saturation)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to enhance or modify images. Available filters: grayscale, blur, sharpen, brightness, contrast, saturation, sepia, invert.";
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
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Filter type: grayscale, blur, sharpen, brightness, contrast, saturation, sepia, invert".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("grayscale".to_string())),
                enum_values: Some(vec![
                    "grayscale".to_string(),
                    "blur".to_string(),
                    "sharpen".to_string(),
                    "brightness".to_string(),
                    "contrast".to_string(),
                    "saturation".to_string(),
                    "sepia".to_string(),
                    "invert".to_string(),
                ]),
            },
            DriverParameter {
                name: "amount".to_string(),
                param_type: "number".to_string(),
                description: "Amount for filters (brightness: -1.0 to 1.0, contrast: 0.0 to 2.0, saturation: 0.0 to 2.0, blur: radius)".to_string(),
                required: false,
                default: Some(json!(0.5)),
                example: Some(json!(0.3)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_filter",
            "parameters": {
                "source": "/photos/colorful.jpg",
                "destination": "/photos/grayscale.jpg",
                "filter": "grayscale"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Applied 'grayscale' filter to image".to_string();
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
        debug!("Executing image_filter driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting image filter operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let filter_type = parameters.get("filter").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("filter"))?;
        let amount = parameters.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.5);
        debug!("Applying filter: source={}, filter={}, amount={}", source, filter_type, amount);
        if let Some(cb) = callback {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Source: {}, destination: {}, filter: {}, amount: {}", source, destination, filter_type, amount)),
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
        // Open image
        let img = image::open(source).map_err(|e| DriverError::execution(format!("Failed to open image '{}': {}", source, e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Applying '{}' filter...", filter_type)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Apply filter based on type
        let result = match filter_type {
            "grayscale" => img.grayscale(),
            "blur" => img.blur(amount as f32),
            "sharpen" => apply_sharpen(&img, amount),
            "brightness" => apply_brightness(&img, amount),
            "contrast" => apply_contrast(&img, amount),
            "saturation" => apply_saturation(&img, amount),
            "sepia" => apply_sepia(&img),
            "invert" => apply_invert(&img),
            _ => return Err(DriverError::execution(format!("Unknown filter: {}", filter_type))),
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving filtered image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        result.save(destination).map_err(|e| DriverError::execution(format!("Failed to save filtered image: {}", e)))?;
        let result_msg = format!("Applied '{}' filter to image", filter_type);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_filter".to_string()), Some(result_msg.clone()));
        }
        info!("Image filter applied: {}", filter_type);
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        parameters.get("filter").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("filter"))?;
        return Ok(());
    }
}
/// Applies sharpen filter to an image
///
/// # Arguments
/// * `img` - Source image
/// * `amount` - Sharpening amount
///
/// # Returns
/// * `image::DynamicImage` - Sharpened image
fn apply_sharpen(img: &image::DynamicImage, amount: f64) -> image::DynamicImage {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    let amount = amount as f32;
    let kernel: [[f32; 3]; 3] = [[0.0, -1.0, 0.0], [-1.0, 4.0 + amount * 2.0, -1.0], [0.0, -1.0, 0.0]];
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = img.get_pixel(x + kx - 1, y + ky - 1);
                    let k = kernel[ky as usize][kx as usize];
                    r += px[0] as f32 * k;
                    g += px[1] as f32 * k;
                    b += px[2] as f32 * k;
                }
            }
            let r = r.clamp(0.0, 255.0) as u8;
            let g = g.clamp(0.0, 255.0) as u8;
            let b = b.clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    return image::DynamicImage::ImageRgba8(output);
}
/// Applies brightness adjustment to an image
///
/// # Arguments
/// * `img` - Source image
/// * `amount` - Brightness amount (-1.0 to 1.0)
///
/// # Returns
/// * `image::DynamicImage` - Adjusted image
fn apply_brightness(img: &image::DynamicImage, amount: f64) -> image::DynamicImage {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    let amount = (amount * 255.0) as i32;
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            let r = ((px[0] as i32 + amount).clamp(0, 255)) as u8;
            let g = ((px[1] as i32 + amount).clamp(0, 255)) as u8;
            let b = ((px[2] as i32 + amount).clamp(0, 255)) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, px[3]]));
        }
    }
    return image::DynamicImage::ImageRgba8(output);
}
/// Applies contrast adjustment to an image
///
/// # Arguments
/// * `img` - Source image
/// * `amount` - Contrast amount (0.0 to 2.0)
///
/// # Returns
/// * `image::DynamicImage` - Adjusted image
fn apply_contrast(img: &image::DynamicImage, amount: f64) -> image::DynamicImage {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    let factor = amount as f32;
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            let r = ((px[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            let g = ((px[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            let b = ((px[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, px[3]]));
        }
    }
    return image::DynamicImage::ImageRgba8(output);
}
/// Applies saturation adjustment to an image
///
/// # Arguments
/// * `img` - Source image
/// * `amount` - Saturation amount (0.0 to 2.0)
///
/// # Returns
/// * `image::DynamicImage` - Adjusted image
fn apply_saturation(img: &image::DynamicImage, amount: f64) -> image::DynamicImage {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    let factor = amount as f32;
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            let r = px[0] as f32;
            let g = px[1] as f32;
            let b = px[2] as f32;
            let gray = r * 0.299 + g * 0.587 + b * 0.114;
            let r = (gray + (r - gray) * factor).clamp(0.0, 255.0) as u8;
            let g = (gray + (g - gray) * factor).clamp(0.0, 255.0) as u8;
            let b = (gray + (b - gray) * factor).clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, px[3]]));
        }
    }
    return image::DynamicImage::ImageRgba8(output);
}
/// Applies sepia filter to an image
///
/// # Arguments
/// * `img` - Source image
///
/// # Returns
/// * `image::DynamicImage` - Sepia-toned image
fn apply_sepia(img: &image::DynamicImage) -> image::DynamicImage {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            let r = px[0] as f32;
            let g = px[1] as f32;
            let b = px[2] as f32;
            let tr = (r * 0.393 + g * 0.769 + b * 0.189).clamp(0.0, 255.0) as u8;
            let tg = (r * 0.349 + g * 0.686 + b * 0.168).clamp(0.0, 255.0) as u8;
            let tb = (r * 0.272 + g * 0.534 + b * 0.131).clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([tr, tg, tb, px[3]]));
        }
    }
    return image::DynamicImage::ImageRgba8(output);
}
/// Applies invert filter to an image
///
/// # Arguments
/// * `img` - Source image
///
/// # Returns
/// * `image::DynamicImage` - Inverted image
fn apply_invert(img: &image::DynamicImage) -> image::DynamicImage {
    use image::{ImageBuffer, Rgba};
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            output.put_pixel(x, y, Rgba([255 - px[0], 255 - px[1], 255 - px[2], px[3]]));
        }
    }
    return image::DynamicImage::ImageRgba8(output);
}
