//! Service history Driver - view service change history
use super::common::get_service_history;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for viewing service history
#[derive(Debug)]
pub struct ServiceHistoryDriver;
#[async_trait::async_trait]
impl Driver for ServiceHistoryDriver {
    fn name(&self) -> &str {
        return "service_history";
    }
    fn description(&self) -> &str {
        return "View service change history";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see the change history of a service.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_history",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx history:\n1. 2024-01-01 00:00:00 Service created\n2. 2024-01-02 00:00:00 Configuration updated".to_string();
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
        debug!("Executing service_history driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let history = get_service_history(service_name).map_err(|e| {
            debug!("Failed to get history for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get history: {}", e));
        })?;
        if history.is_empty() {
            info!("No history found for service {}", service_name);
            return Ok(format!("No history found for service {}", service_name));
        }
        let mut result = format!("Service {} history:\n", service_name);
        for (i, entry) in history.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, entry));
        }
        info!("Found {} history entries for service {}", history.len(), service_name);
        return Ok(result);
    }
}
