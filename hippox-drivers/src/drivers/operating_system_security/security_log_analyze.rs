//! Security log analysis driver
//!
//! This driver provides functionality to analyze security logs for threats and anomalies.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::analyze_security_logs,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for analyzing security logs
#[derive(Debug)]
pub struct SecurityLogAnalyzeDriver;
#[async_trait::async_trait]
impl Driver for SecurityLogAnalyzeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_log_analyze"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Analyze security logs for threats and anomalies"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to analyze system logs for security threats like failed logins, suspicious commands, and unauthorized access"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_log_analyze",
            "parameters": {
                "time_range": 24,
                "show_details": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Security Log Analysis Results:\n\nTime range: Last 24 hours\n\nFindings:\n  - Found 15 failed login attempts\n  - Found 8 sudo commands executed\n  - Found 3 suspicious log entries\n\nSummary: 26 potential security events detected".to_string();
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
        debug!("Executing security_log_analyze driver");
        let time_range = parameters.get("time_range").and_then(|v| v.as_u64()).unwrap_or(24);
        info!("Analyzing security logs for last {} hours", time_range);
        let findings = analyze_security_logs(time_range);
        let mut output = String::new();
        output.push_str(&format!("Security Log Analysis Results:\n\nTime range: Last {} hours\n\n", time_range));
        if findings.is_empty() {
            output.push_str("No security events found.");
            info!("No security events found in log analysis");
        } else {
            info!("Found {} security events", findings.len());
            output.push_str("Findings:\n");
            for finding in &findings {
                output.push_str(&format!("  - {}\n", finding));
            }
        }
        return Ok(output);
    }
}
