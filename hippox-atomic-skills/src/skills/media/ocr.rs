//! OCR (Optical Character Recognition) skill

use crate::{
    SkillCallback, SkillCategory, SkillContext, file_exists,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct OcrSkill;

#[async_trait::async_trait]
impl Skill for OcrSkill {
    fn name(&self) -> &str {
        "ocr"
    }

    fn description(&self) -> &str {
        "Extract text from images using OCR (requires Tesseract)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to extract text from images, scanned documents, or PDFs. \
        Requires Tesseract OCR to be installed on the system."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the image or PDF file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/document.png".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "language".to_string(),
                param_type: "string".to_string(),
                description: "Language code (eng, chi_sim, etc.)".to_string(),
                required: false,
                default: Some(Value::String("eng".to_string())),
                example: Some(Value::String("chi_sim".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "ocr",
            "parameters": {
                "path": "/documents/scan.jpg",
                "language": "eng"
            }
        })
    }

    fn example_output(&self) -> String {
        "Extracted text: Hello World! This is OCR text.".to_string()
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
                Some("Starting OCR text extraction".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let language = parameters
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("eng");

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Image: {}, language: {}", path, language)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }

        if !file_exists(path) {
            anyhow::bail!("File not found: {}", path);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("File verified: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Running Tesseract OCR...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let output = std::process::Command::new("tesseract")
                .args([path, "stdout", "-l", language])
                .output()
                .map_err(|e| {
                    anyhow::anyhow!("Tesseract not found: {}. Please install Tesseract OCR.", e)
                })?;

            if !output.status.success() {
                anyhow::bail!("OCR failed: {}", String::from_utf8_lossy(&output.stderr));
            }

            let text = String::from_utf8(output.stdout)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 output: {}", e))?;

            if text.trim().is_empty() {
                anyhow::bail!("No text found in image");
            }

            let result_msg = format!("Extracted text: {}", text.trim());

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
                    Some("ocr".to_string()),
                    Some(result_msg.clone()),
                );
            }

            Ok(result_msg)
        }

        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("tesseract")
                .args([path, "stdout", "-l", language])
                .output()
                .map_err(|e| anyhow::anyhow!("Tesseract not found: {}", e))?;

            if !output.status.success() {
                anyhow::bail!("OCR failed: {}", String::from_utf8_lossy(&output.stderr));
            }

            let text = String::from_utf8_lossy(&output.stdout);
            if text.trim().is_empty() {
                anyhow::bail!("No text found in image");
            }

            let result_msg = format!("Extracted text: {}", text.trim());

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
                    Some("ocr".to_string()),
                    Some(result_msg.clone()),
                );
            }

            Ok(result_msg)
        }
    }
}
