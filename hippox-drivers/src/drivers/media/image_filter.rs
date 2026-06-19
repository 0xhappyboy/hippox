//! Image filter Driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, file_exists,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImageFilterDriver;

#[async_trait::async_trait]
impl Driver for ImageFilterDriver {
    fn name(&self) -> &str {
        "image_filter"
    }

    fn description(&self) -> &str {
        "Apply various filters to an image (grayscale, blur, sharpen, brightness, contrast, saturation)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to enhance or modify images. Available filters: grayscale, blur, sharpen, brightness, contrast, saturation, sepia, invert."
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_filter",
            "parameters": {
                "source": "/photos/colorful.jpg",
                "destination": "/photos/grayscale.jpg",
                "filter": "grayscale"
            }
        })
    }

    fn example_output(&self) -> String {
        "Applied grayscale filter to image".to_string()
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
                Some("Starting image filter operation".to_string()),
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
        let filter_type = parameters
            .get("filter")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'filter' parameter"))?;
        let amount = parameters
            .get("amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Source: {}, destination: {}, filter: {}, amount: {}",
                    source, destination, filter_type, amount
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
                Some(format!("Applying '{}' filter...", filter_type)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }

        let result = match filter_type {
            "grayscale" => img.grayscale(),
            "blur" => img.blur(amount as f32),
            "sharpen" => apply_sharpen(&img, amount),
            "brightness" => apply_brightness(&img, amount),
            "contrast" => apply_contrast(&img, amount),
            "saturation" => apply_saturation(&img, amount),
            "sepia" => apply_sepia(&img),
            "invert" => apply_invert(&img),
            _ => anyhow::bail!("Unknown filter: {}", filter_type),
        };

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Saving filtered image...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }

        result
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save filtered image: {}", e))?;

        let result_msg = format!("Applied '{}' filter to image", filter_type);

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
                Some("image_filter".to_string()),
                Some(result_msg.clone()),
            );
        }

        Ok(result_msg)
    }
}

fn apply_sharpen(img: &DynamicImage, amount: f64) -> DynamicImage {
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    let amount = amount as f32;
    let kernel: [[f32; 3]; 3] = [
        [0.0, -1.0, 0.0],
        [-1.0, 4.0 + amount * 2.0, -1.0],
        [0.0, -1.0, 0.0],
    ];
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
    DynamicImage::ImageRgba8(output)
}

fn apply_brightness(img: &DynamicImage, amount: f64) -> DynamicImage {
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
    DynamicImage::ImageRgba8(output)
}

fn apply_contrast(img: &DynamicImage, amount: f64) -> DynamicImage {
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
    DynamicImage::ImageRgba8(output)
}

fn apply_saturation(img: &DynamicImage, amount: f64) -> DynamicImage {
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
    DynamicImage::ImageRgba8(output)
}

fn apply_sepia(img: &DynamicImage) -> DynamicImage {
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
    DynamicImage::ImageRgba8(output)
}

fn apply_invert(img: &DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let mut output = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            output.put_pixel(x, y, Rgba([255 - px[0], 255 - px[1], 255 - px[2], px[3]]));
        }
    }
    DynamicImage::ImageRgba8(output)
}
