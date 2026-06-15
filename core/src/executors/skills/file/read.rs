use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path,
};

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

    fn category(&self) -> &str {
        "file"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        let max_size = parameters
            .get("max_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024 * 1024);
        if content.len() > max_size as usize {
            Ok(format!(
                "File too large ({} bytes). Showing first {} bytes:\n{}",
                content.len(),
                max_size,
                &content[..max_size as usize]
            ))
        } else {
            Ok(content)
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}
