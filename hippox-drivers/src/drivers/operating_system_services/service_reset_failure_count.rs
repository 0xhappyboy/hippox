//! Service reset failure count Driver - reset service failure counter
use super::common::reset_failure_count;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for resetting service failure count
#[derive(Debug)]
pub struct ServiceResetFailureCountDriver;
#[async_trait::async_trait]
impl Driver for ServiceResetFailureCountDriver {
    fn name(&self) -> &str {
        return "service_reset_failure_count";
    }
    fn description(&self) -> &str {
        return "Reset service failure count";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to reset the failure counter for a service.";
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
            "action": "service_reset_failure_count",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx failure count reset".to_string();
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
        debug!("Executing service_reset_failure_count driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        reset_failure_count(service_name).map_err(|e| {
            debug!("Failed to reset failure count for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to reset failure count: {}", e));
        })?;
        info!("Service {} failure count reset", service_name);
        return Ok(format!("Service {} failure count reset", service_name));
    }
}
