use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
    validate_path,
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct DeleteFileDriver;

#[async_trait::async_trait]
impl Driver for DeleteFileDriver {
    fn name(&self) -> &str {
        "file_delete"
    }

    fn description(&self) -> &str {
        "Delete a file or empty directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to delete, remove, or delete a file or empty directory"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_delete",
            "parameters": {
                "path": "/tmp/temp.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "File deleted: /tmp/temp.txt".to_string()
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
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let recursive = parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_path = validate_path(path, None)?;
        if !validated_path.exists() {
            anyhow::bail!("Path not found: {}", path);
        }
        if validated_path.is_dir() {
            if recursive {
                fs::remove_dir_all(&validated_path)?;
                Ok(format!("Directory deleted recursively: {}", path))
            } else {
                fs::remove_dir(&validated_path)?;
                Ok(format!("Empty directory deleted: {}", path))
            }
        } else {
            fs::remove_file(&validated_path)?;
            Ok(format!("File deleted: {}", path))
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
