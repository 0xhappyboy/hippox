//! Image format conversion skill

use super::common::get_format_from_extension;
use crate::{
    SkillCallback, SkillCategory, SkillContext, file_exists,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct ImageConvertSkill;

#[async_trait::async_trait]
impl Skill for ImageConvertSkill {
    fn name(&self) -> &str {
        "image_convert"
    }

    fn description(&self) -> &str {
        "Convert an image from one format to another"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to change an image's file format. \
        Quality parameter applies to JPEG and WebP outputs."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/image.png".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path (extension determines output format)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/image.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Quality for lossy formats (JPEG/WebP), 1-100".to_string(),
                required: false,
                default: Some(Value::Number(85.into())),
                example: Some(Value::Number(90.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_convert",
            "parameters": {
                "source": "/photos/screenshot.png",
                "destination": "/photos/screenshot.jpg",
                "quality": 85
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully converted image from PNG to JPEG (quality: 85)".to_string()
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
                Some("Starting image format conversion".to_string()),
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

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Source: {}, destination: {}", source, destination)),
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

        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;

        let source_format = Path::new(source)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let dest_ext = Path::new(destination)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Converting from {} to {}", source_format, dest_ext)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }

        let format = get_format_from_extension(destination)
            .ok_or_else(|| anyhow::anyhow!("Unsupported output format: {}", dest_ext))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Saving converted image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
        }

        img.save_with_format(destination, format)
            .map_err(|e| anyhow::anyhow!("Failed to save image: {}", e))?;

        let result = format!(
            "Successfully converted image from {} to {}",
            source_format.to_uppercase(),
            dest_ext.to_uppercase()
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
                Some("image_convert".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
    }
}
