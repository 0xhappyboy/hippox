//! Service user Driver - get service user/group
use super::common::get_service_user;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting service user information
#[derive(Debug)]
pub struct ServiceUserDriver;
#[async_trait::async_trait]
impl Driver for ServiceUserDriver {
    fn name(&self) -> &str {
        return "service_user";
    }
    fn description(&self) -> &str {
        return "Get the user/group under which a service runs";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see which user account a service is running as.";
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
            "action": "service_user",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx runs as user: www-data, group: www-data".to_string();
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
        debug!("Executing service_user driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let user = get_service_user(service_name).map_err(|e| {
            debug!("Failed to get user for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get user: {}", e));
        })?;
        if let Some(user) = user {
            info!("Service {} runs as user: {}", service_name, user);
            return Ok(format!("Service {} runs as user: {}", service_name, user));
        } else {
            info!("No user information available for service {}", service_name);
            return Ok(format!("No user information available for service {}", service_name));
        }
    }
}
