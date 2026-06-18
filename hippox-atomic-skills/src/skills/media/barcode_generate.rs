//! Barcode generate skill

use crate::{
    SkillCallback, SkillCategory, SkillContext,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use image::{ImageBuffer, Rgb};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct BarcodeGenerateSkill;

#[async_trait::async_trait]
impl Skill for BarcodeGenerateSkill {
    fn name(&self) -> &str {
        "barcode_generate"
    }

    fn description(&self) -> &str {
        "Generate a barcode (Code128, EAN-13, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to create barcodes for product codes, ISBN, etc."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Content to encode in the barcode".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("123456789012".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Output file path (PNG format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/barcode.png".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "barcode_type".to_string(),
                param_type: "string".to_string(),
                description: "Barcode type: code128, ean13, upca, code39".to_string(),
                required: false,
                default: Some(Value::String("code128".to_string())),
                example: Some(Value::String("ean13".to_string())),
                enum_values: Some(vec![
                    "code128".to_string(),
                    "ean13".to_string(),
                    "upca".to_string(),
                    "code39".to_string(),
                ]),
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of barcode in pixels".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(120.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "barcode_generate",
            "parameters": {
                "content": "9781234567897",
                "destination": "/output/barcode.png",
                "barcode_type": "ean13"
            }
        })
    }

    fn example_output(&self) -> String {
        "Barcode generated successfully at /output/barcode.png".to_string()
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
                Some("Starting barcode generation".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let barcode_type = parameters
            .get("barcode_type")
            .and_then(|v| v.as_str())
            .unwrap_or("code128");
        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as u32;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Content: {}, type: {}, height: {}",
                    content, barcode_type, height
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Encoding barcode...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        let encoded = match barcode_type {
            "code128" => {
                use barcoders::sym::code128::Code128;
                let encoder = Code128::new(content)
                    .map_err(|e| anyhow::anyhow!("Failed to create Code128: {}", e))?;
                encoder.encode()
            }
            "ean13" => {
                use barcoders::sym::ean13::EAN13;
                let encoder = EAN13::new(content)
                    .map_err(|e| anyhow::anyhow!("Failed to create EAN13: {}", e))?;
                encoder.encode()
            }
            "code39" => {
                use barcoders::sym::code39::Code39;
                let encoder = Code39::new(content)
                    .map_err(|e| anyhow::anyhow!("Failed to create Code39: {}", e))?;
                encoder.encode()
            }
            _ => anyhow::bail!("Unsupported barcode type: {}", barcode_type),
        };
        let width = encoded.len() as u32;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Encoded length: {} modules", width)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        let module_width = 2;
        let image_width = width * module_width;
        let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(image_width, height);
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Rendering barcode image...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(75), None);
        }
        for (i, &bit) in encoded.iter().enumerate() {
            if bit == 1 {
                let i_usize = i as u32;
                for y in 0..height {
                    for x in 0..module_width {
                        image.put_pixel((i_usize * module_width) + x, y, Rgb([0, 0, 0]));
                    }
                }
            }
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Saving barcode...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(90), None);
        }
        image
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save barcode: {}", e))?;
        let result_msg = format!("Barcode generated successfully at {}", destination);
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
                Some("barcode_generate".to_string()),
                Some(result_msg.clone()),
            );
        }

        Ok(result_msg)
    }
}
