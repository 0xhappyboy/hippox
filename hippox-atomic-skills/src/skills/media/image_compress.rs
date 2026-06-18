//! Image compress skill

use crate::{
    SkillCallback, SkillCategory, SkillContext, file_exists,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ImageCompressSkill;

#[async_trait::async_trait]
impl Skill for ImageCompressSkill {
    fn name(&self) -> &str {
        "image_compress"
    }

    fn description(&self) -> &str {
        "Compress an image to reduce file size with configurable quality"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to reduce image file sizes for web optimization. \
        Lower quality = smaller file size but more artifacts. Quality 70-85 is usually a good balance."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/large_image.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for compressed image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/compressed.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Compression quality (1-100). Higher = better quality".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(75.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_width".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum width (optional). Image will be scaled down proportionally"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1920.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_height".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum height (optional). Image will be scaled down proportionally"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1080.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_compress",
            "parameters": {
                "source": "/uploads/photo.jpg",
                "destination": "/uploads/photo_compressed.jpg",
                "quality": 80,
                "max_width": 1920
            }
        })
    }

    fn example_output(&self) -> String {
        "Compressed image: 2.5MB -> 850KB (66.0% reduction)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting image compression".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let quality = parameters
            .get("quality")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as u8;
        let max_width = parameters
            .get("max_width")
            .and_then(|v| v.as_u64())
            .map(|w| w as u32);
        let max_height = parameters
            .get("max_height")
            .and_then(|v| v.as_u64())
            .map(|h| h as u32);

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Source: {}, destination: {}, quality: {}, max_width: {:?}, max_height: {:?}",
                    source, destination, quality, max_width, max_height
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }

        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Source file verified: {}", source)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }

        let original_size = fs::metadata(source)?.len();

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Original file size: {} bytes", original_size)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }

        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;

        let mut processed = img;

        if let (Some(max_w), Some(max_h)) = (max_width, max_height) {
            let (w, h) = processed.dimensions();
            if w > max_w || h > max_h {
                let ratio = (w as f32 / h as f32).min(max_w as f32 / max_h as f32);
                let new_w = if w > max_w { max_w } else { w };
                let new_h = (new_w as f32 / ratio) as u32;
                processed = processed.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);

                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        skill_index,
                        Some(format!("Resized to {}x{}", new_w, new_h)),
                    );
                }
            }
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Compressing image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }

        let dest_ext = Path::new(destination)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match dest_ext.as_str() {
            "jpg" | "jpeg" => {
                let mut bytes = Vec::new();
                let cursor = std::io::Cursor::new(&mut bytes);
                let mut encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, quality);
                processed
                    .write_with_encoder(encoder)
                    .map_err(|e| anyhow::anyhow!("Failed to encode JPEG: {}", e))?;
                fs::write(destination, bytes)?;
            }
            "webp" => {
                processed
                    .save_with_format(destination, image::ImageFormat::WebP)
                    .map_err(|e| anyhow::anyhow!("Failed to save WebP: {}", e))?;
            }
            _ => {
                processed.save(destination)?;
            }
        }

        let compressed_size = fs::metadata(destination)?.len();
        let reduction = if original_size > 0 {
            ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        let result = format!(
            "Compressed image: {:.2}MB -> {:.2}MB ({:.1}% reduction) at quality {}",
            original_size as f64 / (1024.0 * 1024.0),
            compressed_size as f64 / (1024.0 * 1024.0),
            reduction,
            quality
        );

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Result: {}", result)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("image_compress".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
    }
}
