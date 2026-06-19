use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{ensure_dir, file_exists, read_file_content, validate_path, write_file_content};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MarkdownReadDriver;

#[async_trait::async_trait]
impl Driver for MarkdownReadDriver {
    fn name(&self) -> &str {
        "markdown_read"
    }

    fn description(&self) -> &str {
        "Read and parse Markdown file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a Markdown file, view documentation, or extract content from a .md file"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the Markdown file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("README.md".to_string())),
                enum_values: None,
            },
            DriverParameter {
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

    fn category(&self) -> DriverCategory {
        DriverCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting markdown read operation".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Reading file: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let extract_frontmatter = parameters
            .get("extract_frontmatter")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Extract frontmatter: {}", extract_frontmatter)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }

        let validated_path = validate_path(path, None)?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Validated path: {}",
                    validated_path.to_string_lossy()
                )),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(55), None);
        }

        if !file_exists(&validated_path.to_string_lossy()) {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("File not found: {}", path)),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(60), None);
            }
            anyhow::bail!("Markdown file not found: {}", path);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("File exists, reading content".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }

        let content = read_file_content(&validated_path.to_string_lossy())?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Read {} bytes", content.len())),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }

        if extract_frontmatter && content.starts_with("---") {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Detected frontmatter, parsing".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(90), None);
            }

            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() >= 3 {
                let frontmatter = parts[1].trim();
                let markdown_content = parts[2].trim();
                let result = format!(
                    "Frontmatter:\n{}\n\n---\n\nContent:\n{}",
                    frontmatter, markdown_content
                );

                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        driver_index,
                        Some("Frontmatter extraction complete".to_string()),
                    );
                    cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                    cb.on_complete(
                        task_id.clone(),
                        driver_index,
                        Some("markdown_read".to_string()),
                        Some(result.clone()),
                    );
                }

                return Ok(result);
            }
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Read operation complete".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("markdown_read".to_string()),
                Some(content.clone()),
            );
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
pub struct MarkdownWriteDriver;

#[async_trait::async_trait]
impl Driver for MarkdownWriteDriver {
    fn name(&self) -> &str {
        "markdown_write"
    }

    fn description(&self) -> &str {
        "Write or generate Markdown content to a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create, save, or generate a Markdown document"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the Markdown file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.md".to_string())),
                enum_values: None,
            },
            DriverParameter {
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
            DriverParameter {
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

    fn category(&self) -> DriverCategory {
        DriverCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting markdown write operation".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Writing to file: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Content length: {} characters", content.len())),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let append = parameters
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Append mode: {}", append)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let validated_path = validate_path(path, None)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Validated path: {}",
                    validated_path.to_string_lossy()
                )),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        if let Some(parent) = validated_path.parent() {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!(
                        "Ensuring parent directory: {}",
                        parent.to_string_lossy()
                    )),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(60), None);
            }
            ensure_dir(&parent.to_string_lossy())?;
        }
        if append {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Appending to existing file".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(70), None);
            }
            let existing = if file_exists(&validated_path.to_string_lossy()) {
                read_file_content(&validated_path.to_string_lossy())?
            } else {
                String::new()
            };
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!(
                        "Existing content length: {} characters",
                        existing.len()
                    )),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(80), None);
            }
            let new_content = format!("{}\n\n{}", existing, content);
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Writing appended content".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(90), None);
            }
            write_file_content(&validated_path.to_string_lossy(), &new_content, false)?;
            let result = format!("Content appended to Markdown file: {}", path);
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Append operation complete".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                cb.on_complete(
                    task_id.clone(),
                    driver_index,
                    Some("markdown_write".to_string()),
                    Some(result.clone()),
                );
            }
            Ok(result)
        } else {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Writing new file (overwriting if exists)".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(90), None);
            }
            write_file_content(&validated_path.to_string_lossy(), content, false)?;
            let result = format!("Markdown written to: {}", path);
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Write operation complete".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                cb.on_complete(
                    task_id.clone(),
                    driver_index,
                    Some("markdown_write".to_string()),
                    Some(result.clone()),
                );
            }
            Ok(result)
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
