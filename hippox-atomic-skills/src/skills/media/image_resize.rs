//! Image resize skill

use crate::{
    SkillCallback, SkillCategory, SkillContext, file_exists,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImageResizeSkill;

#[async_trait::async_trait]
impl Skill for ImageResizeSkill {
    fn name(&self) -> &str {
        "image_resize"
    }

    fn description(&self) -> &str {
        "Resize an image to specified width and height"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to change the dimensions of an image. \
        Supports maintaining aspect ratio with the 'preserve_aspect' parameter."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/input.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Target width in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Target height in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "preserve_aspect".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to preserve the original aspect ratio".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
            SkillParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description:
                    "Resampling filter (nearest, triangle, catmullrom, gaussian, lanczos3)"
                        .to_string(),
                required: false,
                default: Some(Value::String("lanczos3".to_string())),
                example: Some(Value::String("gaussian".to_string())),
                enum_values: Some(vec![
                    "nearest".to_string(),
                    "triangle".to_string(),
                    "catmullrom".to_string(),
                    "gaussian".to_string(),
                    "lanczos3".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_resize",
            "parameters": {
                "source": "/photos/original.jpg",
                "destination": "/photos/thumbnail.jpg",
                "width": 300,
                "height": 300,
                "preserve_aspect": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully resized image from 1920x1080 to 300x225".to_string()
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
                Some("Starting image resize operation".to_string()),
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
        let preserve_aspect = parameters
            .get("preserve_aspect")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let filter_name = parameters
            .get("filter")
            .and_then(|v| v.as_str())
            .unwrap_or("lanczos3");
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Source: {}, destination: {}, width: {}, height: {}, preserve_aspect: {}, filter: {}",
                    source, destination, width, height, preserve_aspect, filter_name)),
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
        let (orig_w, orig_h) = img.dimensions();
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Original dimensions: {}x{}", orig_w, orig_h)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        let (new_w, new_h) = if preserve_aspect {
            let ratio = orig_w as f32 / orig_h as f32;
            let target_ratio = width as f32 / height as f32;
            if ratio > target_ratio {
                let new_w = width;
                let new_h = (width as f32 / ratio).round() as u32;
                (new_w, new_h.max(1))
            } else {
                let new_h = height;
                let new_w = (height as f32 * ratio).round() as u32;
                (new_w.max(1), new_h)
            }
        } else {
            (width, height)
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Calculated new dimensions: {}x{}", new_w, new_h)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        let filter = match filter_name {
            "nearest" => image::imageops::FilterType::Nearest,
            "triangle" => image::imageops::FilterType::Triangle,
            "catmullrom" => image::imageops::FilterType::CatmullRom,
            "gaussian" => image::imageops::FilterType::Gaussian,
            _ => image::imageops::FilterType::Lanczos3,
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Resizing image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
        }
        let resized = img.resize(new_w, new_h, filter);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Saving resized image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(85), None);
        }
        resized
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save: {}", e))?;
        let result = format!(
            "Successfully resized image from {}x{} to {}x{}",
            orig_w, orig_h, new_w, new_h
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
                Some("image_resize".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
    }
}
