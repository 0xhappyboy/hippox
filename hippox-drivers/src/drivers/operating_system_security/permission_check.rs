//! Permission check driver
//!
//! This driver provides functionality to check file and directory permissions for security issues.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::{PermissionScanResult, check_file_permissions, scan_permissions},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};
/// Driver for checking file permissions
#[derive(Debug)]
pub struct PermissionCheckDriver;
#[async_trait::async_trait]
impl Driver for PermissionCheckDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_permission_check"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check file and directory permissions for security issues"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to audit file permissions and identify insecure configurations"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to file or directory to check".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/etc/passwd".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Scan directory recursively (default: false)".to_string(),
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
            "action": "security_permission_check",
            "parameters": {
                "path": "/etc",
                "recursive": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Permission Check Results:\n\nPath: /etc/passwd\nExists: Yes\nReadable: Yes\nWritable: No\nOwner: root\nGroup: root\nPermissions: 644\nIssues: None".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemSecurity;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing security_permission_check driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let recursive = parameters.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Permission check: path={}, recursive={}", path, recursive);
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            debug!("Path does not exist: {}", path);
            return Err(DriverError::execution(format!("Path does not exist: {}", path)));
        }
        let mut output = String::new();
        output.push_str(&format!("Permission Check Results for {}:\n\n", path));
        if path_obj.is_file() || !recursive {
            debug!("Checking single file: {}", path);
            let result = check_file_permissions(path);
            output.push_str(&format!("Path: {}\n", result.path));
            output.push_str(&format!("Exists: {}\n", result.exists));
            output.push_str(&format!("Readable: {}\n", result.readable));
            output.push_str(&format!("Writable: {}\n", result.writable));
            output.push_str(&format!("Executable: {}\n", result.executable));
            output.push_str(&format!("Owner: {}\n", result.owner));
            output.push_str(&format!("Group: {}\n", result.group));
            output.push_str(&format!("Permissions: {}\n", result.permissions));
            if !result.issues.is_empty() {
                info!("Found {} issues for {}", result.issues.len(), path);
                output.push_str("\nIssues:\n");
                for issue in &result.issues {
                    output.push_str(&format!("  - {}\n", issue));
                }
            } else {
                output.push_str("\nNo security issues found.");
            }
        } else {
            debug!("Scanning directory recursively: {}", path);
            let scan_result = scan_permissions(path, recursive);
            output.push_str(&format!("Total files scanned: {}\n", scan_result.total_files));
            output.push_str(&format!("Issues found: {}\n\n", scan_result.issues_found));
            let issues: Vec<_> = scan_result.results.iter().filter(|r| !r.issues.is_empty()).collect();
            if issues.is_empty() {
                output.push_str("No permission issues found.");
                info!("No permission issues found in {}", path);
            } else {
                info!("Found {} files with permission issues", issues.len());
                for result in issues {
                    output.push_str(&format!("  {}: {:?}\n", result.path, result.issues));
                }
            }
        }
        return Ok(output);
    }
}
