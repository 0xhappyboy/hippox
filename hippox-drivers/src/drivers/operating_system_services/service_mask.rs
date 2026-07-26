//! Service mask Driver - mask service to prevent starting
use super::common::mask_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for masking a service
#[derive(Debug)]
pub struct ServiceMaskDriver;
#[async_trait::async_trait]
impl Driver for ServiceMaskDriver {
    fn name(&self) -> &str {
        return "service_mask";
    }
    fn description(&self) -> &str {
        return "Mask a service (prevent it from starting)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to completely prevent a service from starting (stronger than disable).";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to mask".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_mask",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx masked".to_string();
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
        debug!("Executing service_mask driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        mask_service(service_name).map_err(|e| {
            debug!("Failed to mask service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to mask service: {}", e));
        })?;
        info!("Service {} masked", service_name);
        return Ok(format!("Service {} masked", service_name));
    }
}
