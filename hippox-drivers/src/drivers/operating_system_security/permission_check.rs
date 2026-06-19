//! Permission check Driver

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    operating_system_security::common::{
        PermissionScanResult, check_file_permissions, scan_permissions,
    },
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct PermissionCheckDriver;

#[async_trait::async_trait]
impl Driver for PermissionCheckDriver {
    fn name(&self) -> &str {
        "security_permission_check"
    }

    fn description(&self) -> &str {
        "Check file and directory permissions for security issues"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to audit file permissions and identify insecure configurations"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_permission_check",
            "parameters": {
                "path": "/etc",
                "recursive": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Permission Check Results:\n\nPath: /etc/passwd\nExists: Yes\nReadable: Yes\nWritable: No\nOwner: root\nGroup: root\nPermissions: 644\nIssues: None".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemSecurity
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

        let path_obj = Path::new(path);
        if !path_obj.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        let mut output = String::new();
        output.push_str(&format!("Permission Check Results for {}:\n\n", path));

        if path_obj.is_file() || !recursive {
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
                output.push_str("\nIssues:\n");
                for issue in &result.issues {
                    output.push_str(&format!("  - {}\n", issue));
                }
            } else {
                output.push_str("\nNo security issues found.");
            }
        } else {
            let scan_result = scan_permissions(path, recursive);
            output.push_str(&format!(
                "Total files scanned: {}\n",
                scan_result.total_files
            ));
            output.push_str(&format!("Issues found: {}\n\n", scan_result.issues_found));

            let issues: Vec<_> = scan_result
                .results
                .iter()
                .filter(|r| !r.issues.is_empty())
                .collect();

            if issues.is_empty() {
                output.push_str("No permission issues found.");
            } else {
                for result in issues {
                    output.push_str(&format!("  {}: {:?}\n", result.path, result.issues));
                }
            }
        }

        Ok(output)
    }
}
