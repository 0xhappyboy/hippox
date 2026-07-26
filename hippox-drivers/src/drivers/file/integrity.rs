//! File integrity monitoring skill
//!
//! This driver provides functionality to monitor file integrity by tracking
//! hash changes over time. It supports initializing baselines, checking
//! for changes, and listing monitored files.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::calculate_file_integrity_hash;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{IntegrityResult, file_exists, validate_path};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tracing::{debug, info, warn};
/// File integrity database (in-memory for demonstration)
/// In production, this would be stored in a file or database
static INTEGRITY_DB: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
/// Get the integrity database instance
fn get_integrity_db() -> &'static Mutex<HashMap<String, String>> {
    INTEGRITY_DB.get_or_init(|| Mutex::new(HashMap::new()))
}
/// Driver for monitoring file integrity
#[derive(Debug)]
pub struct FileIntegrityMonitorDriver;
#[async_trait::async_trait]
impl Driver for FileIntegrityMonitorDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "file_integrity_monitor"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Monitor file integrity by checking hash changes"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to track file changes by comparing hash values over time."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file or directory to monitor".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/monitored_file.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action: 'init' to record baseline, 'check' to verify, 'list' to show all monitored files".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("check".to_string())),
                enum_values: Some(vec!["init".to_string(), "check".to_string(), "list".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "file_integrity_monitor",
            "parameters": {
                "path": "/tmp/monitored_file.txt",
                "action": "init"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Integrity baseline recorded for /tmp/monitored_file.txt".to_string();
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
        debug!("Executing file_integrity_monitor driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        let action = parameters.get("action").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'action' parameter");
            return crate::DriverError::missing_parameter("action");
        })?;
        debug!("Performing '{}' action on {}", action, path);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !validated_path.exists() {
            warn!("Path not found: {}", path);
            return Err(crate::DriverError::execution(format!("Path not found: {}", path)));
        }
        let db = get_integrity_db();
        let mut guard = db.lock().map_err(|e| {
            debug!("Failed to lock database: {}", e);
            return crate::DriverError::internal(format!("Failed to lock database: {}", e));
        })?;
        let path_str = validated_path.to_string_lossy().to_string();
        match action {
            "init" => {
                debug!("Initializing integrity baseline for {}", path);
                if validated_path.is_file() {
                    if !file_exists(&path_str) {
                        warn!("File not found: {}", path);
                        return Err(crate::DriverError::execution(format!("File not found: {}", path)));
                    }
                    let hash = calculate_file_integrity_hash(&path_str).map_err(|e| {
                        debug!("Failed to calculate hash: {}", e);
                        return crate::DriverError::execution(format!("Failed to calculate hash: {}", e));
                    })?;
                    guard.insert(path_str.clone(), hash);
                    info!("Integrity baseline recorded for {}", path);
                    return Ok(format!("Integrity baseline recorded for {}", path));
                } else if validated_path.is_dir() {
                    let mut count = 0;
                    for entry in walkdir::WalkDir::new(&validated_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
                        let file_path = entry.path().to_string_lossy().to_string();
                        if let Ok(hash) = calculate_file_integrity_hash(&file_path) {
                            guard.insert(file_path, hash);
                            count += 1;
                        }
                    }
                    info!("Integrity baseline recorded for {} files in {}", count, path);
                    return Ok(format!("Integrity baseline recorded for {} files in {}", count, path));
                } else {
                    debug!("Path is neither file nor directory: {}", path);
                    return Err(crate::DriverError::validation("path", "Path is neither file nor directory"));
                }
            }
            "check" => {
                debug!("Checking integrity for {}", path);
                let mut results = Vec::new();
                if validated_path.is_file() {
                    if !file_exists(&path_str) {
                        warn!("File not found: {}", path);
                        return Err(crate::DriverError::execution(format!("File not found: {}", path)));
                    }
                    let current_hash = calculate_file_integrity_hash(&path_str).map_err(|e| {
                        debug!("Failed to calculate hash: {}", e);
                        return crate::DriverError::execution(format!("Failed to calculate hash: {}", e));
                    })?;
                    let previous_hash = guard.get(&path_str).cloned();
                    let previous_hash_ref = previous_hash.clone();
                    let result = IntegrityResult {
                        path: path_str.clone(),
                        changed: previous_hash.is_some() && previous_hash.as_ref() != Some(&current_hash),
                        previous_hash: previous_hash.unwrap_or_else(|| "No baseline".to_string()),
                        current_hash,
                        action: if previous_hash_ref.is_none() { "added".to_string() } else { "unchanged".to_string() },
                    };
                    results.push(result);
                } else if validated_path.is_dir() {
                    // Check all files in directory
                    let current_files: HashMap<String, String> = walkdir::WalkDir::new(&validated_path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .filter_map(|e| {
                            let path = e.path().to_string_lossy().to_string();
                            calculate_file_integrity_hash(&path).ok().map(|h| (path, h))
                        })
                        .collect();
                    // Check for added/modified/deleted
                    for (file_path, current_hash) in &current_files {
                        let previous_hash = guard.get(file_path).cloned();
                        let previous_hash_ref = previous_hash.clone();
                        results.push(IntegrityResult {
                            path: file_path.clone(),
                            changed: previous_hash.is_some() && previous_hash.as_ref() != Some(current_hash),
                            previous_hash: previous_hash.unwrap_or_else(|| "No baseline".to_string()),
                            current_hash: current_hash.clone(),
                            action: if previous_hash_ref.is_none() { "added".to_string() } else { "unchanged".to_string() },
                        });
                    }
                    // Check for deleted files
                    for (file_path, _) in guard.iter() {
                        if file_path.starts_with(&path_str) && !current_files.contains_key(file_path) {
                            results.push(IntegrityResult {
                                path: file_path.clone(),
                                changed: true,
                                previous_hash: "".to_string(),
                                current_hash: "".to_string(),
                                action: "deleted".to_string(),
                            });
                        }
                    }
                }
                if results.is_empty() {
                    info!("No files to check");
                    return Ok("No files to check".to_string());
                }
                let mut output = format!("Integrity check results for {}:\n", path);
                let mut changed_count = 0;
                for result in results {
                    let status = if result.action == "deleted" {
                        "DELETED"
                    } else if result.changed {
                        "MODIFIED"
                    } else if result.action == "added" {
                        "ADDED"
                    } else {
                        "UNCHANGED"
                    };
                    if result.action == "deleted" || result.changed {
                        changed_count += 1;
                    }
                    output.push_str(&format!("  {}: {}\n", status, result.path));
                    if result.changed || result.action == "deleted" {
                        output.push_str(&format!("      Previous: {}\n", result.previous_hash));
                        output.push_str(&format!("      Current:  {}\n", result.current_hash));
                    }
                }
                output.push_str(&format!("\nSummary: {} files changed", changed_count));
                info!("Integrity check complete: {} files changed", changed_count);
                return Ok(output);
            }
            "list" => {
                debug!("Listing monitored files for {}", path);
                let mut output = format!("Monitored files:\n");
                let mut count = 0;
                for (file_path, hash) in guard.iter() {
                    if file_path.starts_with(&path_str) {
                        output.push_str(&format!("  {}: {}\n", file_path, hash));
                        count += 1;
                    }
                }
                if count == 0 {
                    output.push_str("  No files being monitored in this path\n");
                } else {
                    output.push_str(&format!("\nTotal: {} files", count));
                }
                info!("Listed {} monitored files", count);
                return Ok(output);
            }
            _ => {
                debug!("Unknown action: {}", action);
                return Err(crate::DriverError::invalid_enum_value(
                    "action",
                    action.to_string(),
                    vec!["init".to_string(), "check".to_string(), "list".to_string()],
                ));
            }
        }
    }
}
