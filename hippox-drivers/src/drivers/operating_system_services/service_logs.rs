//! Service logs Driver - view service logs
use super::common::get_service_logs;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for viewing service logs
#[derive(Debug)]
pub struct ServiceLogsDriver;
#[async_trait::async_trait]
impl Driver for ServiceLogsDriver {
    fn name(&self) -> &str {
        return "service_logs";
    }
    fn description(&self) -> &str {
        return "View service logs";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to view recent logs from a service. Default shows 50 lines.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "service_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "lines".to_string(),
                param_type: "integer".to_string(),
                description: "Number of log lines to show (default: 50)".to_string(),
                required: false,
                default: Some(Value::Number(50.into())),
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_logs",
            "parameters": {
                "service_name": "nginx",
                "lines": 50
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx logs (last 50 lines):\n[2024-01-01 00:00:00] Started service\n[2024-01-01 00:00:01] Listening on port 80".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemServices;
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing service_logs driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let lines = parameters.get("lines").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let logs = get_service_logs(service_name, lines).map_err(|e| {
            debug!("Failed to get logs for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get logs: {}", e));
        })?;
        if logs.is_empty() {
            info!("No logs found for service {}", service_name);
            return Ok(format!("No logs found for service {}", service_name));
        }
        let mut result = format!("Service {} logs (last {} lines):\n", service_name, logs.len());
        let logs_size = logs.len();
        for entry in logs {
            result.push_str(&format!("[{}] {}\n", entry.timestamp, entry.message));
        }
        info!("Retrieved {} log entries for service {}", logs_size, service_name);
        return Ok(result);
    }
}
