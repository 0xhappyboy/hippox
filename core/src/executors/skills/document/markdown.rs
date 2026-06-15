use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{ensure_dir, file_exists, read_file_content, types::{Skill, SkillParameter}, validate_path, write_file_content};

#[derive(Debug)]
pub struct MarkdownReadSkill;

#[async_trait::async_trait]
impl Skill for MarkdownReadSkill {
    fn name(&self) -> &str {
        "markdown_read"
    }

    fn description(&self) -> &str {
        "Read and parse Markdown file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a Markdown file, view documentation, or extract content from a .md file"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the Markdown file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("README.md".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "extract_frontmatter".to_string(),
                param_type: "boolean".to_string(),
                description: "Extract YAML frontmatter metadata if present".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "markdown_read",
            "parameters": {
                "path": "README.md"
            }
        })
    }

    fn example_output(&self) -> String {
        "# Title\n\nThis is the content of the markdown file.".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let extract_frontmatter = parameters
            .get("extract_frontmatter")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("Markdown file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        if extract_frontmatter && content.starts_with("---") {
            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() >= 3 {
                let frontmatter = parts[1].trim();
                let markdown_content = parts[2].trim();
                return Ok(format!(
                    "Frontmatter:\n{}\n\n---\n\nContent:\n{}",
                    frontmatter, markdown_content
                ));
            }
        }
        Ok(content)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MarkdownWriteSkill;

#[async_trait::async_trait]
impl Skill for MarkdownWriteSkill {
    fn name(&self) -> &str {
        "markdown_write"
    }

    fn description(&self) -> &str {
        "Write or generate Markdown content to a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create, save, or generate a Markdown document"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the Markdown file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.md".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Markdown content to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "# Hello\n\nThis is **markdown**.".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "append".to_string(),
                param_type: "boolean".to_string(),
                description: "Append to existing file instead of overwriting".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "markdown_write",
            "parameters": {
                "path": "output.md",
                "content": "# Title\n\nContent here"
            }
        })
    }

    fn example_output(&self) -> String {
        "Markdown written to: output.md".to_string()
    }

    fn category(&self) -> &str {
        "document"
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
            let new_content = format!("{}\n\n{}", existing, content);
            write_file_content(&validated_path.to_string_lossy(), &new_content, false)?;
            Ok(format!("Content appended to Markdown file: {}", path))
        } else {
            write_file_content(&validated_path.to_string_lossy(), content, false)?;
            Ok(format!("Markdown written to: {}", path))
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
