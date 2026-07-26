//! Service set environment variable Driver
use super::common::set_service_env;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting service environment variable
#[derive(Debug)]
pub struct ServiceSetEnvDriver;
#[async_trait::async_trait]
impl Driver for ServiceSetEnvDriver {
    fn name(&self) -> &str {
        return "service_set_env";
    }
    fn description(&self) -> &str {
        return "Set service environment variable";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to set or update an environment variable for a service.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "service_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Environment variable name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MY_VAR".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Environment variable value".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("my_value".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_set_env",
            "parameters": {
                "service_name": "nginx",
                "key": "MY_VAR",
                "value": "my_value"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx environment variable MY_VAR set to my_value".to_string();
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
        debug!("Executing service_set_env driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let key = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            return DriverError::missing_parameter("key");
        })?;
        let value = parameters.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'value' parameter");
            return DriverError::missing_parameter("value");
        })?;
        set_service_env(service_name, key, value).map_err(|e| {
            debug!("Failed to set environment variable for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to set environment variable: {}", e));
        })?;
        info!("Service {} environment variable {} set to {}", service_name, key, value);
        return Ok(format!("Service {} environment variable {} set to {}", service_name, key, value));
    }
}
