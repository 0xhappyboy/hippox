//! File copy skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

use super::common::{copy_directory, copy_file, ensure_dir, validate_path};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct CopyFileSkill;

#[async_trait::async_trait]
impl Skill for CopyFileSkill {
    fn name(&self) -> &str {
        "file_copy"
    }

    fn description(&self) -> &str {
        "Copy or move a file or directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to copy, move, rename, or duplicate a file or directory"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source file or directory path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/source.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file or directory path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/dest.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "move".to_string(),
                param_type: "boolean".to_string(),
                description: "Move instead of copy (rename/move)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Copy directory recursively (if source is a directory)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_copy",
            "parameters": {
                "source": "/tmp/source.txt",
                "destination": "/tmp/dest.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "Copied /tmp/source.txt to /tmp/dest.txt".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let move_file = parameters
            .get("move")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let recursive = parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let validated_source = validate_path(source, None)?;
        let validated_dest = validate_path(destination, None)?;

        if !validated_source.exists() {
            anyhow::bail!("Source not found: {}", source);
        }

        if let Some(parent) = validated_dest.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }

        if move_file {
            // If destination exists, remove it first (for move)
            if validated_dest.exists() {
                if validated_dest.is_dir() {
                    fs::remove_dir_all(&validated_dest)?;
                } else {
                    fs::remove_file(&validated_dest)?;
                }
            }
            fs::rename(&validated_source, &validated_dest)?;
            Ok(format!("Moved {} to {}", source, destination))
        } else {
            // Copy
            let size = if validated_source.is_dir() {
                if !recursive {
                    anyhow::bail!("Source is a directory. Use recursive=true to copy directories.");
                }
                copy_directory(
                    &validated_source.to_string_lossy(),
                    &validated_dest.to_string_lossy(),
                )?
            } else {
                copy_file(
                    &validated_source.to_string_lossy(),
                    &validated_dest.to_string_lossy(),
                )?
            };
            Ok(format!(
                "Copied {} to {} ({} bytes)",
                source, destination, size
            ))
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: source"))?;
        parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: destination"))?;
        Ok(())
    }
}
