//! System log query driver
//!
//! This driver provides functionality to query system logs for security-related events.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::query_system_logs,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for querying system logs
#[derive(Debug)]
pub struct SyslogQueryDriver;
#[async_trait::async_trait]
impl Driver for SyslogQueryDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_syslog_query"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Query system logs for security-related events"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to search system logs for specific events like logins, errors, or authentication failures"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Filter string to search in logs (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Failed password".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "max_entries".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of log entries to return (default: 50)".to_string(),
                required: false,
                default: Some(Value::Number(50.into())),
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_syslog_query",
            "parameters": {
                "filter": "Failed password",
                "max_entries": 20
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System Log Query Results:\n\nFilter: Failed password\nTotal entries: 15\n\n2024-01-01 10:00:00 localhost sshd[1234]: Failed password for root from 192.168.1.100 port 22\n2024-01-01 10:01:00 localhost sshd[1235]: Failed password for admin from 192.168.1.100 port 22".to_string();
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
        debug!("Executing security_syslog_query driver");
        let filter = parameters.get("filter").and_then(|v| v.as_str()).unwrap_or("");
        let max_entries = parameters.get("max_entries").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        info!("Querying system logs: filter='{}', max_entries={}", filter, max_entries);
        let result = query_system_logs(filter, max_entries);
        let mut output = String::new();
        output.push_str(&format!("System Log Query Results:\n\nFilter: {}\n", result.query));
        output.push_str(&format!("Total entries: {}\n\n", result.total_entries));
        if result.entries.is_empty() {
            output.push_str("No log entries found.");
            info!("No log entries found");
        } else {
            info!("Found {} log entries", result.entries.len());
            for entry in &result.entries {
                if !entry.timestamp.is_empty() {
                    output.push_str(&format!("{} ", entry.timestamp));
                }
                if !entry.program.is_empty() {
                    output.push_str(&format!("{}", entry.program));
                    if let Some(pid) = entry.pid {
                        output.push_str(&format!("[{}]", pid));
                    }
                    output.push_str(": ");
                }
                output.push_str(&entry.message);
                output.push('\n');
            }
        }
        return Ok(output);
    }
}
