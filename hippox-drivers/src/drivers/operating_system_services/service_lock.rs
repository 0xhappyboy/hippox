//! Service lock Driver - lock service configuration
use super::common::lock_service_config;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for locking service configuration
#[derive(Debug)]
pub struct ServiceLockDriver;
#[async_trait::async_trait]
impl Driver for ServiceLockDriver {
    fn name(&self) -> &str {
        return "service_lock";
    }
    fn description(&self) -> &str {
        return "Lock service configuration to prevent modifications";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to lock a service configuration and prevent accidental changes.";
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
            "action": "service_lock",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx configuration locked".to_string();
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
        debug!("Executing service_lock driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        lock_service_config(service_name).map_err(|e| {
            debug!("Failed to lock service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to lock service: {}", e));
        })?;
        info!("Service {} configuration locked", service_name);
        return Ok(format!("Service {} configuration locked", service_name));
    }
}
