//! Service restart Driver - restart a system service
use super::common::restart_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for restarting a service
#[derive(Debug)]
pub struct ServiceRestartDriver;
#[async_trait::async_trait]
impl Driver for ServiceRestartDriver {
    fn name(&self) -> &str {
        return "service_restart";
    }
    fn description(&self) -> &str {
        return "Restart a system service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to restart a service to apply changes.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to restart".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_restart",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx restarted successfully".to_string();
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
        debug!("Executing service_restart driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        restart_service(service_name).map_err(|e| {
            debug!("Failed to restart service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to restart service: {}", e));
        })?;
        info!("Service {} restarted successfully", service_name);
        return Ok(format!("Service {} restarted successfully", service_name));
    }
}
