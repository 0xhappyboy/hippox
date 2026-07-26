//! Service start Driver - start a system service
use super::common::start_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for starting a service
#[derive(Debug)]
pub struct ServiceStartDriver;
#[async_trait::async_trait]
impl Driver for ServiceStartDriver {
    fn name(&self) -> &str {
        return "service_start";
    }
    fn description(&self) -> &str {
        return "Start a system service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to start a service like ssh, nginx, etc.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to start".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_start",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx started successfully".to_string();
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
        debug!("Executing service_start driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        start_service(service_name).map_err(|e| {
            debug!("Failed to start service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to start service: {}", e));
        })?;
        info!("Service {} started successfully", service_name);
        return Ok(format!("Service {} started successfully", service_name));
    }
}
