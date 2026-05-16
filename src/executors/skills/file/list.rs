use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use super::common;
use crate::executors::types::Skill;

#[derive(Debug)]
pub struct ListDirectorySkill;

#[async_trait::async_trait]
impl Skill for ListDirectorySkill {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "List contents of a directory. Parameters: path (required) - directory path, show_hidden (optional, default false) - show hidden files, detail (optional, default false) - show detailed info"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let show_hidden = parameters
            .get("show_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let detail = parameters
            .get("detail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_path = common::validate_path(path, None)?;
        if !validated_path.is_dir() {
            anyhow::bail!("Not a directory: {}", path);
        }
        let entries = fs::read_dir(&validated_path)?;
        let mut result = Vec::new();
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            if detail {
                let metadata = entry.metadata()?;
                let file_type = if metadata.is_dir() { "DIR" } else { "FILE" };
                let size = metadata.len();
                result.push(format!("{}  {}  {} bytes", file_type, name, size));
            } else {
                result.push(name);
            }
        }
        if result.is_empty() {
            Ok(format!("Directory is empty: {}", path))
        } else {
            Ok(format!("Contents of {}:\n{}", path, result.join("\n")))
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
