//! Service status Driver - query service status
use super::common::get_service_status;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting service status
#[derive(Debug)]
pub struct ServiceStatusDriver;
#[async_trait::async_trait]
impl Driver for ServiceStatusDriver {
    fn name(&self) -> &str {
        return "service_status";
    }
    fn description(&self) -> &str {
        return "Query the status of a system service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to check if a service is running.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to check".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_status",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx status: running".to_string();
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
        debug!("Executing service_status driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let status = get_service_status(service_name).map_err(|e| {
            debug!("Failed to get status for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get service status: {}", e));
        })?;
        info!("Service {} status: {}", service_name, status);
        return Ok(format!("Service {} status: {}", service_name, status));
    }
}
