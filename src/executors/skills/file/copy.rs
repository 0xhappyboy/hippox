use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use super::common;
use crate::executors::types::Skill;

#[derive(Debug)]
pub struct CopyFileSkill;

#[async_trait::async_trait]
impl Skill for CopyFileSkill {
    fn name(&self) -> &str {
        "file_copy"
    }

    fn description(&self) -> &str {
        "Copy or move a file. Parameters: source (required) - source path, destination (required) - destination path, move (optional, default false) - move instead of copy"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let move_file = parameters
            .get("move")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_source = common::validate_path(source, None)?;
        let validated_dest = common::validate_path(destination, None)?;
        if !validated_source.exists() {
            anyhow::bail!("Source not found: {}", source);
        }
        if let Some(parent) = validated_dest.parent() {
            common::ensure_dir(&parent.to_string_lossy())?;
        }
        if move_file {
            fs::rename(&validated_source, &validated_dest)?;
            Ok(format!("Moved {} to {}", source, destination))
        } else {
            fs::copy(&validated_source, &validated_dest)?;
            Ok(format!("Copied {} to {}", source, destination))
        }
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
