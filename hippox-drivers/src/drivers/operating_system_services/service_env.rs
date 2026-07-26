//! Service environment variables Driver
use super::common::get_service_env;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for viewing service environment variables
#[derive(Debug)]
pub struct ServiceEnvDriver;
#[async_trait::async_trait]
impl Driver for ServiceEnvDriver {
    fn name(&self) -> &str {
        return "service_env";
    }
    fn description(&self) -> &str {
        return "View service environment variables";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see the environment variables available to a service.";
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
            "action": "service_env",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx environment variables:\nPATH=/usr/local/bin:/usr/bin\nUSER=www-data".to_string();
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
        debug!("Executing service_env driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let env = get_service_env(service_name).map_err(|e| {
            debug!("Failed to get environment for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get environment: {}", e));
        })?;
        if env.is_empty() {
            info!("No environment variables found for service {}", service_name);
            return Ok(format!("No environment variables found for service {}", service_name));
        }
        let mut result = format!("Service {} environment variables:\n", service_name);
        let env_size = env.len();
        for (key, value) in env {
            result.push_str(&format!("{}={}\n", key, value));
        }
        info!("Retrieved {} environment variables for service {}", env_size, service_name);
        return Ok(result);
    }
}
