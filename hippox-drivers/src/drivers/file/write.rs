//! File write skill

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, ensure_dir, file_exists, read_file_content,
    types::{Driver, DriverParameter},
    validate_path, write_file_content,
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug)]
pub struct WriteFileDriver;

#[async_trait::async_trait]
impl Driver for WriteFileDriver {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to save, write, create, or append content to a file"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/output.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Content to write to the file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, World!".to_string())),
                enum_values: None,
            },
            DriverParameter {
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

    fn category(&self) -> DriverCategory {
        DriverCategory::File
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let start_time = Instant::now();
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting file write operation".to_string()),
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
                Some(format!("Target path: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        if let Some(cb) = cb {
            let content_preview = if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.to_string()
            };
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Content length: {} bytes", content.len())),
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
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Validating file path".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let validated_path = validate_path(path, None)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Ensuring parent directory exists".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Writing content to file".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        let result = if append {
            let existing = if file_exists(&validated_path.to_string_lossy()) {
                read_file_content(&validated_path.to_string_lossy())?
            } else {
                String::new()
            };
            let new_content = format!("{}{}", existing, content);
            write_file_content(&validated_path.to_string_lossy(), &new_content, false)?;
            format!("Content appended to file: {}", path)
        } else {
            write_file_content(&validated_path.to_string_lossy(), content, false)?;
            format!("Content written to file: {}", path)
        };
        if let Some(cb) = cb {
            let duration = start_time.elapsed().as_millis() as u64;
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Completed in {}ms", duration)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
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
        parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        Ok(())
    }
}
