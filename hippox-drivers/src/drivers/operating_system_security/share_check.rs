//! Network share check Driver

use crate::DriverCallback;
use crate::{
    DriverCategory, DriverContext,
    operating_system_security::common::check_network_shares,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ShareCheckDriver;

#[async_trait::async_trait]
impl Driver for ShareCheckDriver {
    fn name(&self) -> &str {
        "security_share_check"
    }

    fn description(&self) -> &str {
        "Check network shares for security issues"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to audit network shares and identify insecure configurations"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_share_check",
            "parameters": {
                "show_all": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Network Share Security Check:\n\nShares found: 3\n\n/share (NFS Export) [World-readable NFS export]\n/var/www (Samba share)\n\nSecurity Issues: 1".to_string()
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
        let show_all = parameters
            .get("show_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let shares = check_network_shares();

        let mut output = String::new();
        output.push_str(&format!(
            "Network Share Security Check:\n\nShares found: {}\n",
            shares.len()
        ));

        if shares.is_empty() {
            output.push_str("\nNo network shares found.");
            return Ok(output);
        }

        let has_issues: Vec<_> = shares
            .iter()
            .filter(|s| !s.security_issues.is_empty())
            .collect();
        let no_issues: Vec<_> = shares
            .iter()
            .filter(|s| s.security_issues.is_empty())
            .collect();

        if !has_issues.is_empty() {
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
        }

        Ok(output)
    }
}
