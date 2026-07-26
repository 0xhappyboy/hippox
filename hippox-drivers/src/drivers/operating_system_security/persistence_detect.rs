//! Persistence detection driver
//!
//! This driver provides functionality to detect persistence mechanisms that could indicate backdoor or malware.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::check_persistence_mechanisms,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for detecting persistence mechanisms
#[derive(Debug)]
pub struct PersistenceDetectDriver;
#[async_trait::async_trait]
impl Driver for PersistenceDetectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_persistence_detect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Detect persistence mechanisms that could indicate backdoor or malware"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to detect suspicious persistence mechanisms like cron jobs, startup scripts, and registry entries"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "show_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Show all persistence entries (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "min_score".to_string(),
                param_type: "integer".to_string(),
                description: "Minimum suspicious score threshold (1-10, default: 5)".to_string(),
                required: false,
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(3.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_persistence_detect",
            "parameters": {
                "show_all": true,
                "min_score": 3
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Persistence Detection Results:\n\nSuspicious entries found: 3\n\n1. Cron job [SUSPICIOUS]\n  Command: /tmp/malware.sh\n  Source: User crontab\n  Reason: Potential suspicious cron job\n\n2. SSH authorized keys [SUSPICIOUS]\n  Path: ~/.ssh/authorized_keys\n  Reason: Potential persistence mechanism\n\nSummary: Review suspicious persistence entries".to_string();
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
        debug!("Executing security_persistence_detect driver");
        let show_all = parameters.get("show_all").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Persistence detection: show_all={}", show_all);
        let entries = check_persistence_mechanisms();
        let mut output = String::new();
        output.push_str(&format!("Persistence Detection Results:\n\nTotal entries found: {}\n", entries.len()));
        if entries.is_empty() {
            output.push_str("\nNo persistence mechanisms found.");
            info!("No persistence mechanisms found");
            return Ok(output);
        }
        let suspicious: Vec<_> = entries.iter().filter(|e| e.suspicious).collect();
        let legitimate: Vec<_> = entries.iter().filter(|e| !e.suspicious).collect();
        if !suspicious.is_empty() {
            info!("Found {} suspicious persistence entries", suspicious.len());
            output.push_str(&format!("\nSuspicious entries found: {}\n", suspicious.len()));
            for (i, entry) in suspicious.iter().enumerate() {
                output.push_str(&format!("\n{}. {} [SUSPICIOUS]\n", i + 1, entry.name));
                if !entry.path.is_empty() {
                    output.push_str(&format!("  Path: {}\n", entry.path));
                }
                if !entry.command.is_empty() {
                    output.push_str(&format!("  Command: {}\n", entry.command));
                }
                output.push_str(&format!("  Source: {}\n", entry.source));
                output.push_str(&format!("  Reason: {}\n", entry.reason));
            }
            output.push_str("\nSummary: Review suspicious persistence entries.");
        } else {
            output.push_str("\nNo suspicious persistence mechanisms found.");
            info!("No suspicious persistence mechanisms found");
        }
        if show_all && !legitimate.is_empty() {
            output.push_str(&format!("\nLegitimate entries: {}\n", legitimate.len()));
            for entry in legitimate {
                output.push_str(&format!("  - {} ({})\n", entry.name, entry.path));
            }
        }
        return Ok(output);
    }
}
