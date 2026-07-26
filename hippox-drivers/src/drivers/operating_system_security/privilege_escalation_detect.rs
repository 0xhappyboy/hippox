//! Privilege escalation detection driver
//!
//! This driver provides functionality to detect potential privilege escalation vectors.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::check_privilege_escalation,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for detecting privilege escalation vectors
#[derive(Debug)]
pub struct PrivilegeEscalationDetectDriver;
#[async_trait::async_trait]
impl Driver for PrivilegeEscalationDetectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_privilege_escalation_detect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Detect potential privilege escalation vectors"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to identify potential privilege escalation vulnerabilities like SUID binaries, sudo misconfigurations, and writable system files"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "show_all".to_string(),
            param_type: "boolean".to_string(),
            description: "Show all checks including non-vulnerable ones (default: false)".to_string(),
            required: false,
            default: Some(Value::Bool(false)),
            example: Some(Value::Bool(true)),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_privilege_escalation_detect",
            "parameters": {
                "show_all": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Privilege Escalation Detection Results:\n\nVulnerabilities found: 2\n\nSUID Binaries [HIGH]\n  Description: Checking for SUID binaries\n  Details: /usr/bin/sudo\n  /usr/bin/passwd\n\nSudo rights [HIGH]\n  Description: Checking for sudo rights\n  Details: ALL=(ALL) NOPASSWD: ALL\n\nSummary: 2 potential privilege escalation vectors detected".to_string();
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
        debug!("Executing security_privilege_escalation_detect driver");
        let show_all = parameters.get("show_all").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Privilege escalation detection: show_all={}", show_all);
        let results = check_privilege_escalation();
        let mut output = String::new();
        output.push_str("Privilege Escalation Detection Results:\n\n");
        let vulnerable: Vec<_> = results.iter().filter(|r| r.vulnerable).collect();
        let safe: Vec<_> = results.iter().filter(|r| !r.vulnerable).collect();
        if vulnerable.is_empty() && !show_all {
            output.push_str("No privilege escalation vulnerabilities detected.");
            info!("No privilege escalation vulnerabilities detected");
            return Ok(output);
        }
        if !vulnerable.is_empty() {
            info!("Found {} privilege escalation vulnerabilities", vulnerable.len());
            output.push_str(&format!("Vulnerabilities found: {}\n", vulnerable.len()));
            for (i, result) in vulnerable.iter().enumerate() {
                output.push_str(&format!("\n{}. {} [{}]\n", i + 1, result.check_name, result.severity.to_uppercase()));
                output.push_str(&format!("  Description: {}\n", result.description));
                if !result.details.is_empty() && result.details != "No findings" {
                    output.push_str(&format!("  Details: {}\n", result.details));
                }
            }
            output.push_str(&format!("\nSummary: {} potential privilege escalation vectors detected", vulnerable.len()));
        }
        if show_all && !safe.is_empty() {
            output.push_str("\n\nSafe checks:\n");
            for result in safe {
                output.push_str(&format!("  - {}: Secure\n", result.check_name));
            }
        }
        return Ok(output);
    }
}
