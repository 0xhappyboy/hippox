use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    ensure_dir, file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path, write_file_content,
};

#[derive(Debug)]
pub struct WriteFileSkill;

#[async_trait::async_trait]
impl Skill for WriteFileSkill {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to save, write, create, or append content to a file"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/output.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Content to write to the file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, World!".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "append".to_string(),
                param_type: "boolean".to_string(),
                description: "Append to file instead of overwriting".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_write",
            "parameters": {
                "path": "/tmp/hello.txt",
                "content": "Hello, World!"
            }
        })
    }

    fn example_output(&self) -> String {
        "Content written to file: /tmp/hello.txt".to_string()
    }

    fn category(&self) -> &str {
        "file"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let append = parameters
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        if append {
            let existing = if file_exists(&validated_path.to_string_lossy()) {
                read_file_content(&validated_path.to_string_lossy())?
            } else {
                String::new()
            };
            let new_content = format!("{}{}", existing, content);
            write_file_content(&validated_path.to_string_lossy(), &new_content, false)?;
            Ok(format!("Content appended to file: {}", path))
        } else {
            write_file_content(&validated_path.to_string_lossy(), content, false)?;
            Ok(format!("Content written to file: {}", path))
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        Ok(())
    }
}
