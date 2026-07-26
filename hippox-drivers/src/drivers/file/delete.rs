//! File delete skill
//!
//! This driver provides functionality to delete files and directories
//! with support for recursive directory deletion.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
    validate_path,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info, warn};
/// Driver for deleting files and directories
#[derive(Debug)]
pub struct DeleteFileDriver;
#[async_trait::async_trait]
impl Driver for DeleteFileDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "file_delete"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Delete a file or empty directory"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to delete, remove, or delete a file or empty directory"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file or directory to delete".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/temp.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Delete directory recursively (including all contents)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "file_delete",
            "parameters": {
                "path": "/tmp/temp.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "File deleted: /tmp/temp.txt".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing file_delete driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        let recursive = parameters.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Deleting {} (recursive: {})", path, recursive);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !validated_path.exists() {
            warn!("Path not found: {}", path);
            return Err(crate::DriverError::execution(format!("Path not found: {}", path)));
        }
        let result = if validated_path.is_dir() {
            if recursive {
                debug!("Removing directory recursively: {}", path);
                fs::remove_dir_all(&validated_path).map_err(|e| {
                    debug!("Failed to remove directory recursively: {}", e);
                    return crate::DriverError::io(format!("Failed to remove directory recursively: {}", e));
                })?;
                info!("Directory deleted recursively: {}", path);
                format!("Directory deleted recursively: {}", path)
            } else {
                debug!("Removing empty directory: {}", path);
                fs::remove_dir(&validated_path).map_err(|e| {
                    debug!("Failed to remove empty directory: {}", e);
                    return crate::DriverError::io(format!("Failed to remove empty directory: {}", e));
                })?;
                info!("Empty directory deleted: {}", path);
                format!("Empty directory deleted: {}", path)
            }
        } else {
            debug!("Removing file: {}", path);
            fs::remove_file(&validated_path).map_err(|e| {
                debug!("Failed to remove file: {}", e);
                return crate::DriverError::io(format!("Failed to remove file: {}", e));
            })?;
            info!("File deleted: {}", path);
            format!("File deleted: {}", path)
        };
        return Ok(result);
    }
    /// Validates parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if parameters.get("path").and_then(|v| v.as_str()).is_none() {
            return Err(crate::DriverError::missing_parameter("path"));
        }
        return Ok(());
    }
}
