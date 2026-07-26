//! Service reload Driver - reload service configuration
use super::common::reload_service_config;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reloading service configuration
#[derive(Debug)]
pub struct ServiceReloadDriver;
#[async_trait::async_trait]
impl Driver for ServiceReloadDriver {
    fn name(&self) -> &str {
        return "service_reload";
    }
    fn description(&self) -> &str {
        return "Reload service configuration without restarting";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to apply configuration changes without restarting the service.";
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
            "action": "service_reload",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx configuration reloaded".to_string();
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
        debug!("Executing service_reload driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        reload_service_config(service_name).map_err(|e| {
            debug!("Failed to reload service {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to reload service: {}", e));
        })?;
        info!("Service {} configuration reloaded", service_name);
        return Ok(format!("Service {} configuration reloaded", service_name));
    }
}
