//! Service timeout Driver - set service startup timeout
use super::common::set_startup_timeout;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting service startup timeout
#[derive(Debug)]
pub struct ServiceSetTimeoutDriver;
#[async_trait::async_trait]
impl Driver for ServiceSetTimeoutDriver {
    fn name(&self) -> &str {
        return "service_set_timeout";
    }
    fn description(&self) -> &str {
        return "Set service startup timeout";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to configure how long to wait for a service to start.";
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
                name: "timeout_seconds".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_set_timeout",
            "parameters": {
                "service_name": "nginx",
                "timeout_seconds": 60
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx startup timeout set to 60 seconds".to_string();
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
        debug!("Executing service_set_timeout driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let timeout = parameters.get("timeout_seconds").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'timeout_seconds' parameter");
            return DriverError::missing_parameter("timeout_seconds");
        })? as u32;
        set_startup_timeout(service_name, timeout).map_err(|e| {
            debug!("Failed to set startup timeout for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to set startup timeout: {}", e));
        })?;
        info!("Service {} startup timeout set to {} seconds", service_name, timeout);
        return Ok(format!("Service {} startup timeout set to {} seconds", service_name, timeout));
    }
}
