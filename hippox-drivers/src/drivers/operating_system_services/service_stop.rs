//! Service stop Driver - stop a running system service
use super::common::stop_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for stopping a service
#[derive(Debug)]
pub struct ServiceStopDriver;
#[async_trait::async_trait]
impl Driver for ServiceStopDriver {
    fn name(&self) -> &str {
        return "service_stop";
    }
    fn description(&self) -> &str {
        return "Stop a running system service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to stop a running service.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to stop".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_stop",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx stopped successfully".to_string();
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
        debug!("Executing service_stop driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        stop_service(service_name).map_err(|e| {
            debug!("Failed to stop service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to stop service: {}", e));
        })?;
        info!("Service {} stopped successfully", service_name);
        return Ok(format!("Service {} stopped successfully", service_name));
    }
}
