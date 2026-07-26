//! Service config path Driver - get service configuration file path
use super::common::get_service_config_path;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting service configuration path
#[derive(Debug)]
pub struct ServiceConfigPathDriver;
#[async_trait::async_trait]
impl Driver for ServiceConfigPathDriver {
    fn name(&self) -> &str {
        return "service_config_path";
    }
    fn description(&self) -> &str {
        return "Get the configuration file path of a service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to find where a service's configuration is stored.";
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
            "action": "service_config_path",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx config path: /etc/systemd/system/nginx.service".to_string();
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
        debug!("Executing service_config_path driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let path = get_service_config_path(service_name).map_err(|e| {
            debug!("Failed to get config path for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get config path: {}", e));
        })?;
        if let Some(path) = path {
            info!("Service {} config path: {}", service_name, path);
            return Ok(format!("Service {} config path: {}", service_name, path));
        } else {
            info!("No configuration file found for service {}", service_name);
            return Ok(format!("No configuration file found for service {}", service_name));
        }
    }
}
