//! Service failure count Driver - get service failure count
use super::common::get_failure_count;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting service failure count
#[derive(Debug)]
pub struct ServiceFailureCountDriver;
#[async_trait::async_trait]
impl Driver for ServiceFailureCountDriver {
    fn name(&self) -> &str {
        return "service_failure_count";
    }
    fn description(&self) -> &str {
        return "Get service failure count";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see how many times a service has failed.";
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
            "action": "service_failure_count",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx has failed 0 times".to_string();
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
        debug!("Executing service_failure_count driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let count = get_failure_count(service_name).map_err(|e| {
            debug!("Failed to get failure count for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get failure count: {}", e));
        })?;
        if let Some(count) = count {
            info!("Service {} has failed {} times", service_name, count);
            return Ok(format!("Service {} has failed {} times", service_name, count));
        } else {
            info!("No failure count available for service {}", service_name);
            return Ok(format!("No failure count available for service {}", service_name));
        }
    }
}
