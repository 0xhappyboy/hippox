//! Log packing skill

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{ensure_dir, pack_logs, validate_path};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct LogPackDriver;

#[async_trait::async_trait]
impl Driver for LogPackDriver {
    fn name(&self) -> &str {
        "log_pack"
    }

    fn description(&self) -> &str {
        "Pack log files from a directory into an archive"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to collect and compress log files from a directory into a single archive (tar, tar.gz, or zip)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "source_dir".to_string(),
                param_type: "string".to_string(),
                description: "Directory containing log files to pack".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/var/log/myapp".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Output archive path (without extension)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/logs_backup".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Archive format: 'tar', 'tar.gz', 'tgz', or 'zip'".to_string(),
                required: false,
                default: Some(Value::String("tar.gz".to_string())),
                example: Some(Value::String("zip".to_string())),
                enum_values: Some(vec![
                    "tar".to_string(),
                    "tar.gz".to_string(),
                    "tgz".to_string(),
                    "zip".to_string(),
                ]),
            },
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "File pattern to match (e.g., '*.log', 'app-*.log')".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("*.log".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "log_pack",
            "parameters": {
                "source_dir": "/var/log/myapp",
                "destination": "/tmp/logs_backup",
                "format": "tar.gz",
                "pattern": "*.log"
            }
        })
    }

    fn example_output(&self) -> String {
        "Logs packed successfully: /tmp/logs_backup.tar.gz\nFiles packed: 15".to_string()
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
        let source_dir = parameters
            .get("source_dir")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source_dir' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("tar.gz");
        let pattern = parameters.get("pattern").and_then(|v| v.as_str());

        let source_path = validate_path(source_dir, None)?;
        if !source_path.exists() || !source_path.is_dir() {
            anyhow::bail!("Source directory does not exist: {}", source_dir);
        }

        // Create a temporary directory for filtered files if pattern is specified
        let temp_dir = if let Some(pattern_str) = pattern {
            let temp = std::env::temp_dir().join(format!("log_pack_{}", std::process::id()));
            ensure_dir(temp.to_str().unwrap())?;

            // Copy matching files to temp directory
            let mut count = 0;
            for entry in walkdir::WalkDir::new(&source_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                if glob_match(pattern_str, &file_name) {
                    let dest_file = temp.join(&*file_name);
                    fs::copy(file_path, &dest_file)?;
                    count += 1;
                }
            }

            if count == 0 {
                fs::remove_dir_all(&temp)?;
                anyhow::bail!(
                    "No files matching pattern '{}' found in {}",
                    pattern_str,
                    source_dir
                );
            }

            Some(temp)
        } else {
            None
        };

        let source_to_pack = if let Some(temp) = &temp_dir {
            temp.to_str().unwrap()
        } else {
            source_dir
        };

        // Pack the logs
        let archive_path = pack_logs(source_to_pack, destination, format)?;

        // Clean up temp directory
        if let Some(temp) = temp_dir {
            fs::remove_dir_all(&temp)?;
        }

        // Count files in archive
        let file_count = if let Ok(entries) = fs::read_dir(source_path) {
            entries.filter_map(|e| e.ok()).count()
        } else {
            0
        };

        Ok(format!(
            "Logs packed successfully: {}\nFiles packed: {}",
            archive_path, file_count
        ))
    }
}

/// Simple glob pattern matching (supports * only)
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }

    let pattern_parts: Vec<&str> = pattern.split('*').collect();

    if pattern_parts.len() == 1 {
        return text == pattern;
    }

    // Match first part
    if !pattern_parts[0].is_empty() && !text.starts_with(pattern_parts[0]) {
        return false;
    }

    // Match middle parts
    let mut pos = pattern_parts[0].len();
    for part in &pattern_parts[1..pattern_parts.len() - 1] {
        if let Some(idx) = text[pos..].find(part) {
            pos += idx + part.len();
        } else {
            return false;
        }
    }

    // Match last part
    let last = pattern_parts.last().unwrap();
    if !last.is_empty() {
        if text.len() < last.len() {
            return false;
        }
        if !text[text.len() - last.len()..].starts_with(last) {
            return false;
        }
    }

    true
}
