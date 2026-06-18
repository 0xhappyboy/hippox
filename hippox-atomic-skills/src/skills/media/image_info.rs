//! Image info skill

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
pub struct ImageInfoSkill;

#[async_trait::async_trait]
impl Skill for ImageInfoSkill {
    fn name(&self) -> &str {
        "image_info"
    }

    fn description(&self) -> &str {
        "Get metadata information about an image (dimensions, format, file size)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to inspect an image's properties before processing it."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/image.jpg".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_info",
            "parameters": {
                "path": "/photos/landscape.jpg"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"dimensions":{"width":1920,"height":1080},"format":"JPEG","file_size_bytes":245760}"#
            .to_string()
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
                Some("Starting image info extraction".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Image path: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        if !file_exists(path) {
            anyhow::bail!("Image not found: {}", path);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("File verified: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let metadata = fs::metadata(path)?;
        let file_size_bytes = metadata.len();
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("File size: {} bytes", file_size_bytes)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        let img = image::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", path, e))?;
        let (width, height) = img.dimensions();
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Image dimensions: {}x{}", width, height)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        let format = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_uppercase();
        let color_type = match img.color() {
            image::ColorType::L8 => "Grayscale 8-bit",
            image::ColorType::La8 => "Grayscale with Alpha 8-bit",
            image::ColorType::Rgb8 => "RGB 8-bit",
            image::ColorType::Rgba8 => "RGBA 8-bit",
            _ => "Other",
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Format: {}, color type: {}", format, color_type)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(80), None);
        }
        let info = json!({
            "path": path,
            "dimensions": {
                "width": width,
                "height": height,
                "aspect_ratio": format!("{:.2}", width as f64 / height as f64)
            },
            "format": format,
            "file_size": {
                "bytes": file_size_bytes,
                "kb": file_size_bytes as f64 / 1024.0,
                "mb": file_size_bytes as f64 / (1024.0 * 1024.0)
            },
            "color_type": color_type,
            "total_pixels": width as u64 * height as u64
        });
        let result = serde_json::to_string_pretty(&info)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Image info extraction complete".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("image_info".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
    }
}
