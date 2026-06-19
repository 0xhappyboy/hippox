//! Service logs Driver

use super::common::get_service_logs;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceLogsDriver;

#[async_trait::async_trait]
impl Driver for ServiceLogsDriver {
    fn name(&self) -> &str {
        "service_logs"
    }

    fn description(&self) -> &str {
        "View service logs"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to view recent logs from a service. Default shows 50 lines."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_logs",
            "parameters": {
                "service_name": "nginx",
                "lines": 50
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx logs (last 50 lines):\n[2024-01-01 00:00:00] Started service\n[2024-01-01 00:00:01] Listening on port 80".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemServices
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let lines = parameters
            .get("lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;
        let logs = get_service_logs(service_name, lines)?;
        if logs.is_empty() {
            return Ok(format!("No logs found for service {}", service_name));
        }
        let mut result = format!(
            "Service {} logs (last {} lines):\n",
            service_name,
            logs.len()
        );
        for entry in logs {
            result.push_str(&format!("[{}] {}\n", entry.timestamp, entry.message));
        }
        Ok(result)
    }
}
