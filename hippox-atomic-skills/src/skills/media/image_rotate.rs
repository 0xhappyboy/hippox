//! Image rotate skill

use crate::{
    SkillCallback, SkillCategory, SkillContext, file_exists,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImageRotateSkill;

#[async_trait::async_trait]
impl Skill for ImageRotateSkill {
    fn name(&self) -> &str {
        "image_rotate"
    }

    fn description(&self) -> &str {
        "Rotate an image by 90, 180, or 270 degrees"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to rotate images that are oriented incorrectly."
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
                description: "Destination file path for rotated image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "angle".to_string(),
                param_type: "integer".to_string(),
                description: "Rotation angle in degrees: 90, 180, 270".to_string(),
                required: false,
                default: Some(Value::Number(90.into())),
                example: Some(Value::Number(180.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_rotate",
            "parameters": {
                "source": "/photos/portrait.jpg",
                "destination": "/photos/rotated.jpg",
                "angle": 90
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully rotated image by 90 degrees".to_string()
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
                Some("Starting image rotation".to_string()),
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
        let angle = parameters
            .get("angle")
            .and_then(|v| v.as_u64())
            .unwrap_or(90) as u32;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Source: {}, destination: {}, angle: {}",
                    source, destination, angle
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

        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Rotating image by {} degrees...", angle)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }

        let rotated = match angle % 360 {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            0 => img,
            _ => anyhow::bail!("Unsupported angle {}. Supported: 90, 180, 270", angle),
        };

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Saving rotated image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(75), None);
        }

        rotated
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save rotated image: {}", e))?;

        let result = format!("Successfully rotated image by {} degrees", angle);

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
                Some("image_rotate".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
    }
}
