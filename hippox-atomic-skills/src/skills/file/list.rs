//! File list skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{list_directory, validate_path};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct ListDirectorySkill;

#[async_trait::async_trait]
impl Skill for ListDirectorySkill {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "List contents of a directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to list, show, or see what's inside a directory"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Directory path to list".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "show_hidden".to_string(),
                param_type: "boolean".to_string(),
                description: "Show hidden files (starting with dot)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "List directory contents recursively".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "detail".to_string(),
                param_type: "boolean".to_string(),
                description: "Show detailed information (type, size)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_list",
            "parameters": {
                "path": "/home/user"
            }
        })
    }

    fn example_output(&self) -> String {
        "Contents of /home/user:\ndocuments\nDownloads\nPictures".to_string()
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
                Some("Starting directory listing".to_string()),
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
                Some(format!("Path: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        let show_hidden = parameters
            .get("show_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let recursive = parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let detail = parameters
            .get("detail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Show hidden: {}", show_hidden)),
            );
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Recursive: {}", recursive)),
            );
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Detail: {}", detail)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let validated_path = validate_path(path, None)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Path validated".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        if !validated_path.is_dir() {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Not a directory: {}", path)),
                );
                cb.on_error(
                    task_id.clone(),
                    skill_index,
                    step_name.clone(),
                    Some("Not a directory".to_string()),
                );
            }
            anyhow::bail!("Not a directory: {}", path);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Reading directory contents".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }
        let entries = list_directory(&validated_path.to_string_lossy(), recursive, show_hidden)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Found {} entries", entries.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        if entries.is_empty() {
            let result = format!("Directory is empty: {}", path);
            if let Some(cb) = cb {
                cb.on_log(task_id.clone(), skill_index, Some(result.clone()));
                cb.on_progress(task_id.clone(), skill_index, Some(100), None);
                cb.on_complete(
                    task_id.clone(),
                    skill_index,
                    Some("file_list".to_string()),
                    Some(result.clone()),
                );
            }
            return Ok(result);
        }
        let mut result_vec = Vec::new();
        if detail {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some("Gathering detailed file information".to_string()),
                );
                cb.on_progress(task_id.clone(), skill_index, Some(70), None);
            }
            for (idx, entry_path) in entries.iter().enumerate() {
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if let Ok(metadata) = fs::metadata(entry_path) {
                    let file_type = if metadata.is_dir() { "DIR" } else { "FILE" };
                    let size = metadata.len();
                    result_vec.push(format!("{}  {}  {} bytes", file_type, name, size));
                } else {
                    result_vec.push(name);
                }
                if let Some(cb) = cb {
                    let progress = 70 + ((idx + 1) * 20 / entries.len()) as u32;
                    cb.on_progress(task_id.clone(), skill_index, Some(progress), None);
                }
            }
        } else {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some("Processing file names".to_string()),
                );
                cb.on_progress(task_id.clone(), skill_index, Some(70), None);
            }
            for (idx, entry_path) in entries.iter().enumerate() {
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                result_vec.push(name);
                if let Some(cb) = cb {
                    let progress = 70 + ((idx + 1) * 20 / entries.len()) as u32;
                    cb.on_progress(task_id.clone(), skill_index, Some(progress), None);
                }
            }
        }
        let header = if recursive {
            format!("Contents of {} (recursive):", path)
        } else {
            format!("Contents of {}:", path)
        };
        let result = format!("{}\n{}", header, result_vec.join("\n"));
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Formatted output with {} entries",
                    result_vec.len()
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(90), None);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Result length: {} characters", result.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("file_list".to_string()),
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
