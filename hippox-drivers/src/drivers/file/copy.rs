//! File copy skill

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, copy_directory, copy_file, ensure_dir,
    types::{Driver, DriverParameter},
    validate_path,
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

#[derive(Debug)]
pub struct CopyFileDriver;

#[async_trait::async_trait]
impl Driver for CopyFileDriver {
    fn name(&self) -> &str {
        "file_copy"
    }

    fn description(&self) -> &str {
        "Copy or move a file or directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to copy, move, rename, or duplicate a file or directory"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source file or directory path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/source.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file or directory path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/dest.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "move".to_string(),
                param_type: "boolean".to_string(),
                description: "Move instead of copy (rename/move)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
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
                Some("Starting file copy operation".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Source: {}", source)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(15), None);
        }
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Destination: {}", destination)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let move_file = parameters
            .get("move")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Move mode: {}", move_file)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let recursive = parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Recursive mode: {}", recursive)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Validating source path".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(35), None);
        }
        let validated_source = validate_path(source, None)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Validating destination path".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let validated_dest = validate_path(destination, None)?;
        if !validated_source.exists() {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("Source not found: {}", source)),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                cb.on_complete(
                    task_id.clone(),
                    driver_index,
                    step_name,
                    Some(format!("Source not found: {}", source)),
                );
            }
            anyhow::bail!("Source not found: {}", source);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Ensuring destination directory exists".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        if let Some(parent) = validated_dest.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Checking source type".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(55), None);
        }
        let result = if move_file {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Moving file/directory".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(65), None);
            }
            if validated_dest.exists() {
                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        driver_index,
                        Some("Removing existing destination".to_string()),
                    );
                }
                if validated_dest.is_dir() {
                    fs::remove_dir_all(&validated_dest)?;
                } else {
                    fs::remove_file(&validated_dest)?;
                }
            }
            if let Some(cb) = cb {
                cb.on_progress(task_id.clone(), driver_index, Some(80), None);
            }
            fs::rename(&validated_source, &validated_dest)?;

            if let Some(cb) = cb {
                cb.on_progress(task_id.clone(), driver_index, Some(90), None);
            }
            format!("Moved {} to {}", source, destination)
        } else {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some("Copying file/directory".to_string()),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(65), None);
            }
            let size = if validated_source.is_dir() {
                if !recursive {
                    anyhow::bail!("Source is a directory. Use recursive=true to copy directories.");
                }
                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        driver_index,
                        Some("Copying directory recursively".to_string()),
                    );
                    cb.on_progress(task_id.clone(), driver_index, Some(70), None);
                }
                copy_directory(
                    &validated_source.to_string_lossy(),
                    &validated_dest.to_string_lossy(),
                )?
            } else {
                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        driver_index,
                        Some("Copying file".to_string()),
                    );
                    cb.on_progress(task_id.clone(), driver_index, Some(70), None);
                }
                copy_file(
                    &validated_source.to_string_lossy(),
                    &validated_dest.to_string_lossy(),
                )?
            };
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("Copied {} bytes", size)),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(90), None);
            }
            format!("Copied {} to {} ({} bytes)", source, destination, size)
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
