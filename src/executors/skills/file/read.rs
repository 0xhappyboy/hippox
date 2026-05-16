use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use super::common;
use crate::executors::types::Skill;

#[derive(Debug)]
pub struct ReadFileSkill;

#[async_trait::async_trait]
impl Skill for ReadFileSkill {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read content from a file. Parameters: path (required) - file path to read"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = common::validate_path(path, None)?;
        if !common::file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let content = common::read_file_content(&validated_path.to_string_lossy())?;
        let max_size = parameters
            .get("max_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024 * 1024); // 1MB
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
