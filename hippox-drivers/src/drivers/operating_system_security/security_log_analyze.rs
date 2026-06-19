//! Security log analysis Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    operating_system_security::common::analyze_security_logs,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct SecurityLogAnalyzeDriver;

#[async_trait::async_trait]
impl Driver for SecurityLogAnalyzeDriver {
    fn name(&self) -> &str {
        "security_log_analyze"
    }

    fn description(&self) -> &str {
        "Analyze security logs for threats and anomalies"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to analyze system logs for security threats like failed logins, suspicious commands, and unauthorized access"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "time_range".to_string(),
                param_type: "integer".to_string(),
                description: "Time range in hours to analyze (default: 24)".to_string(),
                required: false,
                default: Some(Value::Number(24.into())),
                example: Some(Value::Number(168.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "show_details".to_string(),
                param_type: "boolean".to_string(),
                description: "Show detailed findings (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_log_analyze",
            "parameters": {
                "time_range": 24,
                "show_details": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Security Log Analysis Results:\n\nTime range: Last 24 hours\n\nFindings:\n  - Found 15 failed login attempts\n  - Found 8 sudo commands executed\n  - Found 3 suspicious log entries\n\nSummary: 26 potential security events detected".to_string()
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
        let time_range = parameters
            .get("time_range")
            .and_then(|v| v.as_u64())
            .unwrap_or(24);

        let findings = analyze_security_logs(time_range);

        let mut output = String::new();
        output.push_str(&format!(
            "Security Log Analysis Results:\n\nTime range: Last {} hours\n\n",
            time_range
        ));

        if findings.is_empty() {
            output.push_str("No security events found.");
        } else {
            output.push_str("Findings:\n");
            for finding in &findings {
                output.push_str(&format!("  - {}\n", finding));
            }
        }

        Ok(output)
    }
}
