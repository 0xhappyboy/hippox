//! File read skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{file_exists, read_file_content, validate_path};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug)]
pub struct ReadFileSkill;

#[async_trait::async_trait]
impl Skill for ReadFileSkill {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read content from a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read, view, or display the contents of a file"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Absolute or relative path to the file to read".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/document.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_size".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of bytes to read (prevents huge files)".to_string(),
                required: false,
                default: Some(Value::Number(1048576.into())),
                example: Some(Value::Number(1024.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_read",
            "parameters": {
                "path": "/tmp/config.json"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"name\": \"example\",\n  \"version\": \"1.0\"\n}".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let start_time = Instant::now();
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name.clone());
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting file read operation".to_string()),
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
                Some(format!("Target path: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Validating file path".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let validated_path = validate_path(path, None)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Checking if file exists".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        if !file_exists(&validated_path.to_string_lossy()) {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("File not found: {}", path)),
                );
                cb.on_progress(task_id.clone(), skill_index, Some(100), None);
                cb.on_complete(
                    task_id.clone(),
                    skill_index,
                    step_name,
                    Some(format!("File not found: {}", path)),
                );
            }
            anyhow::bail!("File not found: {}", path);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Reading file content".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Read {} bytes", content.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
        }
        let max_size = parameters
            .get("max_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024 * 1024);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Max size limit: {} bytes", max_size)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(80), None);
        }
        let result = if content.len() > max_size as usize {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!(
                        "File exceeds max size ({} > {})",
                        content.len(),
                        max_size
                    )),
                );
            }
            format!(
                "File too large ({} bytes). Showing first {} bytes:\n{}",
                content.len(),
                max_size,
                &content[..max_size as usize]
            )
        } else {
            content
        };
        if let Some(cb) = cb {
            let duration = start_time.elapsed().as_millis() as u64;
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Completed in {}ms", duration)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                step_name,
                Some(result.clone()),
            );
        }
        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}
