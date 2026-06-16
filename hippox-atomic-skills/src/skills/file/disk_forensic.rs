//! Disk forensic analysis skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{perform_forensic_analysis, validate_path};

#[derive(Debug)]
pub struct DiskForensicSkill;

#[async_trait::async_trait]
impl Skill for DiskForensicSkill {
    fn name(&self) -> &str {
        "disk_forensic_analyze"
    }

    fn description(&self) -> &str {
        "Perform forensic analysis on a file or directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to analyze files for forensic evidence, suspicious patterns, and metadata."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file or directory to analyze".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/suspicious_file".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Analyze directory recursively (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "disk_forensic_analyze",
            "parameters": {
                "path": "/tmp/suspicious_file",
                "recursive": false
            }
        })
    }

    fn example_output(&self) -> String {
        "Forensic Analysis Results:\nFile: /tmp/suspicious_file\nType: File\nMagic Bytes: ELF executable\nMetadata:\n  - Size: 1024 bytes\n  - Modified: 1704067200\nSuspicious: Yes\nReasons:\n  - Contains suspicious code pattern: eval(\n  - Executable file with unusual format".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
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
        let validated_path = validate_path(path, None)?;
        if !validated_path.exists() {
            anyhow::bail!("Path not found: {}", path);
        }
        let mut results = Vec::new();
        if validated_path.is_file() {
            let result = perform_forensic_analysis(&validated_path.to_string_lossy())?;
            results.push(result);
        } else if validated_path.is_dir() && recursive {
            for entry in walkdir::WalkDir::new(&validated_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path().to_string_lossy().to_string();
                if let Ok(result) = perform_forensic_analysis(&file_path) {
                    results.push(result);
                }
            }
        } else {
            anyhow::bail!("Path is a directory. Use recursive=true to analyze directory contents.");
        }
        if results.is_empty() {
            return Ok("No files analyzed".to_string());
        }
        let mut output = "Forensic Analysis Results:\n".to_string();
        let mut suspicious_count = 0;
        for result in results {
            let status = if result.suspicious {
                "SUSPICIOUS"
            } else {
                "CLEAR"
            };
            if result.suspicious {
                suspicious_count += 1;
            }
            output.push_str(&format!("\n{}: {}\n", status, result.path));
            output.push_str(&format!("  Type: {}\n", result.file_type));
            if let Some(magic) = result.magic_bytes {
                output.push_str(&format!("  Magic Bytes: {}\n", magic));
            }
            if !result.embedded_metadata.is_empty() {
                output.push_str("  Metadata:\n");
                for (key, value) in result.embedded_metadata {
                    output.push_str(&format!("    - {}: {}\n", key, value));
                }
            }
            if result.suspicious {
                output.push_str("  Suspicious Reasons:\n");
                for reason in result.suspicious_reasons {
                    output.push_str(&format!("    - {}\n", reason));
                }
            }
        }
        output.push_str(&format!(
            "\nSummary: {} suspicious files found",
            suspicious_count
        ));
        Ok(output)
    }
}
