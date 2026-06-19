//! System log query Driver

use crate::DriverContext;
use crate::{
    DriverCallback, DriverCategory,
    operating_system_security::common::query_system_logs,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SyslogQueryDriver;

#[async_trait::async_trait]
impl Driver for SyslogQueryDriver {
    fn name(&self) -> &str {
        "security_syslog_query"
    }

    fn description(&self) -> &str {
        "Query system logs for security-related events"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to search system logs for specific events like logins, errors, or authentication failures"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_syslog_query",
            "parameters": {
                "filter": "Failed password",
                "max_entries": 20
            }
        })
    }

    fn example_output(&self) -> String {
        "System Log Query Results:\n\nFilter: Failed password\nTotal entries: 15\n\n2024-01-01 10:00:00 localhost sshd[1234]: Failed password for root from 192.168.1.100 port 22\n2024-01-01 10:01:00 localhost sshd[1235]: Failed password for admin from 192.168.1.100 port 22".to_string()
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
        let filter = parameters
            .get("filter")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let max_entries = parameters
            .get("max_entries")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let result = query_system_logs(filter, max_entries);

        let mut output = String::new();
        output.push_str(&format!(
            "System Log Query Results:\n\nFilter: {}\n",
            result.query
        ));
        output.push_str(&format!("Total entries: {}\n\n", result.total_entries));

        if result.entries.is_empty() {
            output.push_str("No log entries found.");
        } else {
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

        Ok(output)
    }
}
