//! Disk forensic analysis skill
//!
//! This driver provides functionality to perform forensic analysis on
//! files and directories to detect suspicious patterns and metadata.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{perform_forensic_analysis, validate_path};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for performing forensic analysis on files and directories
#[derive(Debug)]
pub struct DiskForensicDriver;
#[async_trait::async_trait]
impl Driver for DiskForensicDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "disk_forensic_analyze"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Perform forensic analysis on a file or directory"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to analyze files for forensic evidence, suspicious patterns, and metadata."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file or directory to analyze".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/suspicious_file".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Analyze directory recursively (default: false)".to_string(),
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
            "action": "disk_forensic_analyze",
            "parameters": {
                "path": "/tmp/suspicious_file",
                "recursive": false
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Forensic Analysis Results:\nFile: /tmp/suspicious_file\nType: File\nMagic Bytes: ELF executable\nMetadata:\n  - Size: 1024 bytes\n  - Modified: 1704067200\nSuspicious: Yes\nReasons:\n  - Contains suspicious code pattern: eval(\n  - Executable file with unusual format".to_string();
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
        debug!("Executing disk_forensic_analyze driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        let recursive = parameters.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Analyzing {} (recursive: {})", path, recursive);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !validated_path.exists() {
            warn!("Path not found: {}", path);
            return Err(crate::DriverError::execution(format!("Path not found: {}", path)));
        }
        let mut results = Vec::new();
        if validated_path.is_file() {
            debug!("Analyzing single file: {}", path);
            let result = perform_forensic_analysis(&validated_path.to_string_lossy()).map_err(|e| {
                debug!("Failed to perform forensic analysis: {}", e);
                return crate::DriverError::execution(format!("Failed to perform forensic analysis: {}", e));
            })?;
            results.push(result);
        } else if validated_path.is_dir() && recursive {
            debug!("Analyzing directory recursively: {}", path);
            for entry in walkdir::WalkDir::new(&validated_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
                let file_path = entry.path().to_string_lossy().to_string();
                if let Ok(result) = perform_forensic_analysis(&file_path) {
                    results.push(result);
                }
            }
        } else {
            debug!("Path is a directory but recursive not set");
            return Err(crate::DriverError::validation("recursive", "Path is a directory. Use recursive=true to analyze directory contents."));
        }
        if results.is_empty() {
            info!("No files analyzed");
            return Ok("No files analyzed".to_string());
        }
        let mut output = "Forensic Analysis Results:\n".to_string();
        let mut suspicious_count = 0;
        for result in results {
            let status = if result.suspicious {
                suspicious_count += 1;
                "SUSPICIOUS"
            } else {
                "CLEAR"
            };
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
        output.push_str(&format!("\nSummary: {} suspicious files found", suspicious_count));
        info!("Forensic analysis complete: {} suspicious files found", suspicious_count);
        return Ok(output);
    }
}
