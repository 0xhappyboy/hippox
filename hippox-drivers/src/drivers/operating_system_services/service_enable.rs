//! Service enable Driver - enable service auto-start
use super::common::enable_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for enabling service auto-start
#[derive(Debug)]
pub struct ServiceEnableDriver;
#[async_trait::async_trait]
impl Driver for ServiceEnableDriver {
    fn name(&self) -> &str {
        return "service_enable";
    }
    fn description(&self) -> &str {
        return "Enable a service to start automatically on boot";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to set a service to start automatically at system boot.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to enable".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_enable",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx enabled for auto-start".to_string();
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
        debug!("Executing service_enable driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        enable_service(service_name).map_err(|e| {
            debug!("Failed to enable service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to enable service: {}", e));
        })?;
        info!("Service {} enabled for auto-start", service_name);
        return Ok(format!("Service {} enabled for auto-start", service_name));
    }
}
