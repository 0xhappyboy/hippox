//! File list skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{list_directory, validate_path};

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

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
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

        let validated_path = validate_path(path, None)?;

        if !validated_path.is_dir() {
            anyhow::bail!("Not a directory: {}", path);
        }

        let entries = list_directory(&validated_path.to_string_lossy(), recursive, show_hidden)?;

        if entries.is_empty() {
            return Ok(format!("Directory is empty: {}", path));
        }

        let mut result = Vec::new();

        if detail {
            for entry_path in &entries {
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if let Ok(metadata) = fs::metadata(entry_path) {
                    let file_type = if metadata.is_dir() { "DIR" } else { "FILE" };
                    let size = metadata.len();
                    result.push(format!("{}  {}  {} bytes", file_type, name, size));
                } else {
                    result.push(name);
                }
            }
        } else {
            for entry_path in entries {
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                result.push(name);
            }
        }

        let header = if recursive {
            format!("Contents of {} (recursive):", path)
        } else {
            format!("Contents of {}:", path)
        };

        Ok(format!("{}\n{}", header, result.join("\n")))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}
