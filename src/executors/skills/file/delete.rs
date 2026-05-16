use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use super::common;
use crate::executors::types::Skill;

#[derive(Debug)]
pub struct DeleteFileSkill;

#[async_trait::async_trait]
impl Skill for DeleteFileSkill {
    fn name(&self) -> &str {
        "file_delete"
    }

    fn description(&self) -> &str {
        "Delete a file or empty directory. Parameters: path (required) - file/directory path, recursive (optional, default false) - delete directory recursively"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let recursive = parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_path = common::validate_path(path, None)?;
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
