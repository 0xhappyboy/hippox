//! Service unlock Driver - unlock service configuration
use super::common::unlock_service_config;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for unlocking service configuration
#[derive(Debug)]
pub struct ServiceUnlockDriver;
#[async_trait::async_trait]
impl Driver for ServiceUnlockDriver {
    fn name(&self) -> &str {
        return "service_unlock";
    }
    fn description(&self) -> &str {
        return "Unlock service configuration";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to unlock a service configuration and allow modifications.";
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
            "action": "service_unlock",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx configuration unlocked".to_string();
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
        debug!("Executing service_unlock driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        unlock_service_config(service_name).map_err(|e| {
            debug!("Failed to unlock service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to unlock service: {}", e));
        })?;
        info!("Service {} configuration unlocked", service_name);
        return Ok(format!("Service {} configuration unlocked", service_name));
    }
}
