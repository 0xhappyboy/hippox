//! Image stitch skill

use crate::{
    SkillCallback, SkillCategory, SkillContext, file_exists,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImageStitchSkill;

#[async_trait::async_trait]
impl Skill for ImageStitchSkill {
    fn name(&self) -> &str {
        "image_stitch"
    }

    fn description(&self) -> &str {
        "Stitch multiple images together horizontally or vertically"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to combine multiple images into a single image. \
        Images will be resized to match dimensions if they differ."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "images".to_string(),
                param_type: "array".to_string(),
                description: "Array of image file paths to stitch".to_string(),
                required: true,
                default: None,
                example: Some(json!(["/path/to/1.jpg", "/path/to/2.jpg"])),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/stitched.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "direction".to_string(),
                param_type: "string".to_string(),
                description: "Stitch direction: 'horizontal' or 'vertical'".to_string(),
                required: false,
                default: Some(Value::String("horizontal".to_string())),
                example: Some(Value::String("vertical".to_string())),
                enum_values: Some(vec!["horizontal".to_string(), "vertical".to_string()]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_stitch",
            "parameters": {
                "images": ["/photos/1.jpg", "/photos/2.jpg", "/photos/3.jpg"],
                "destination": "/photos/panorama.jpg",
                "direction": "horizontal"
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully stitched 3 images horizontally".to_string()
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
                Some("Starting image stitch operation".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        let images = parameters
            .get("images")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'images' parameter"))?;

        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;

        let direction = parameters
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("horizontal");

        if images.is_empty() {
            anyhow::bail!("At least one image required");
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Destination: {}, direction: {}, image count: {}",
                    destination,
                    direction,
                    images.len()
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }

        let mut loaded_images = Vec::new();
        let mut max_width = 0;
        let mut max_height = 0;

        for (idx, img_path) in images.iter().enumerate() {
            let path = img_path
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid image path"))?;

            if !file_exists(path) {
                anyhow::bail!("Image not found: {}", path);
            }

            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Loading image {}: {}", idx + 1, path)),
                );
                cb.on_progress(
                    task_id.clone(),
                    skill_index,
                    Some(20 + (idx as u32 * 5)),
                    None,
                );
            }

            let img = image::open(path)
                .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", path, e))?;

            let (w, h) = img.dimensions();
            max_width = max_width.max(w);
            max_height = max_height.max(h);
            loaded_images.push(img);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Max dimensions: {}x{}", max_width, max_height)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }

        let (total_w, total_h) = if direction == "horizontal" {
            let total_w = loaded_images.iter().map(|img| img.width()).sum::<u32>();
            (total_w, max_height)
        } else {
            (
                max_width,
                loaded_images.iter().map(|img| img.height()).sum::<u32>(),
            )
        };

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Output dimensions: {}x{}", total_w, total_h)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }

        let mut output = ImageBuffer::new(total_w, total_h);
        let mut offset_x = 0;
        let mut offset_y = 0;

        for (idx, img) in loaded_images.iter().enumerate() {
            let (w, h) = img.dimensions();
            let scaled = if direction == "horizontal" {
                if h != max_height {
                    img.resize(
                        w * max_height / h,
                        max_height,
                        image::imageops::FilterType::Lanczos3,
                    )
                } else {
                    img.clone()
                }
            } else {
                if w != max_width {
                    img.resize(
                        max_width,
                        h * max_width / w,
                        image::imageops::FilterType::Lanczos3,
                    )
                } else {
                    img.clone()
                }
            };

            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!(
                        "Stitching image {} at offset ({}, {})",
                        idx + 1,
                        offset_x,
                        offset_y
                    )),
                );
                cb.on_progress(
                    task_id.clone(),
                    skill_index,
                    Some(60 + (idx as u32 * 10)),
                    None,
                );
            }

            let (sw, sh) = scaled.dimensions();
            for y in 0..sh {
                for x in 0..sw {
                    let px = scaled.get_pixel(x, y);
                    output.put_pixel(offset_x + x, offset_y + y, px);
                }
            }

            if direction == "horizontal" {
                offset_x += sw;
            } else {
                offset_y += sh;
            }
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Saving stitched image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(90), None);
        }
        let result = DynamicImage::ImageRgba8(output);
        result
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save stitched image: {}", e))?;
        let result_msg = format!(
            "Successfully stitched {} images {}",
            loaded_images.len(),
            direction
        );
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Result: {}", result_msg)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("image_stitch".to_string()),
                Some(result_msg.clone()),
            );
        }

        Ok(result_msg)
    }
}
