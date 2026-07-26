//! Network share check driver
//!
//! This driver provides functionality to check network shares for security issues.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::check_network_shares,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking network shares
#[derive(Debug)]
pub struct ShareCheckDriver;
#[async_trait::async_trait]
impl Driver for ShareCheckDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_share_check"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check network shares for security issues"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to audit network shares and identify insecure configurations"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "show_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Show all shares (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "check_writable".to_string(),
                param_type: "boolean".to_string(),
                description: "Check for writable shares (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_share_check",
            "parameters": {
                "show_all": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Network Share Security Check:\n\nShares found: 3\n\n/share (NFS Export) [World-readable NFS export]\n/var/www (Samba share)\n\nSecurity Issues: 1".to_string();
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
        debug!("Executing security_share_check driver");
        let show_all = parameters.get("show_all").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Share check: show_all={}", show_all);
        let shares = check_network_shares();
        let mut output = String::new();
        output.push_str(&format!("Network Share Security Check:\n\nShares found: {}\n", shares.len()));
        if shares.is_empty() {
            output.push_str("\nNo network shares found.");
            info!("No network shares found");
            return Ok(output);
        }
        let has_issues: Vec<_> = shares.iter().filter(|s| !s.security_issues.is_empty()).collect();
        let no_issues: Vec<_> = shares.iter().filter(|s| s.security_issues.is_empty()).collect();
        if !has_issues.is_empty() {
            info!("Found {} shares with security issues", has_issues.len());
            output.push_str("\nShares with security issues:\n");
            for share in &has_issues {
                output.push_str(&format!("  {} ({})\n", share.name, share.path));
                for issue in &share.security_issues {
                    output.push_str(&format!("    - {}\n", issue));
                }
            }
        }
        if show_all && !no_issues.is_empty() {
            output.push_str("\nOther shares (no issues):\n");
            for share in &no_issues {
                output.push_str(&format!("  {} ({})\n", share.name, share.path));
            }
        }
        if has_issues.is_empty() && !show_all {
            output.push_str("\nNo security issues found in network shares.");
            info!("No security issues found in network shares");
        }
        return Ok(output);
    }
}
