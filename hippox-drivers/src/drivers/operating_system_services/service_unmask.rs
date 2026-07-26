//! Service unmask Driver - unmask a service
use super::common::unmask_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for unmasking a service
#[derive(Debug)]
pub struct ServiceUnmaskDriver;
#[async_trait::async_trait]
impl Driver for ServiceUnmaskDriver {
    fn name(&self) -> &str {
        return "service_unmask";
    }
    fn description(&self) -> &str {
        return "Unmask a service (allow it to start again)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to remove a mask from a service and allow it to start.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to unmask".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_unmask",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx unmasked".to_string();
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
        debug!("Executing service_unmask driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        unmask_service(service_name).map_err(|e| {
            debug!("Failed to unmask service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to unmask service: {}", e));
        })?;
        info!("Service {} unmasked", service_name);
        return Ok(format!("Service {} unmasked", service_name));
    }
}
