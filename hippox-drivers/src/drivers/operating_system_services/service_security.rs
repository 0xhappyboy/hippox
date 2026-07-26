//! Service security Driver - view service security settings
use super::common::get_service_security;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for viewing service security settings
#[derive(Debug)]
pub struct ServiceSecurityDriver;
#[async_trait::async_trait]
impl Driver for ServiceSecurityDriver {
    fn name(&self) -> &str {
        return "service_security";
    }
    fn description(&self) -> &str {
        return "View service security settings";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see the security context of a service.";
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
            "action": "service_security",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx security settings:\nUser: www-data\nGroup: www-data\nProtectSystem: full".to_string();
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
        debug!("Executing service_security driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let security = get_service_security(service_name).map_err(|e| {
            debug!("Failed to get security settings for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get security settings: {}", e));
        })?;
        if security.is_empty() {
            info!("No security settings found for service {}", service_name);
            return Ok(format!("No security settings found for service {}", service_name));
        }
        let mut result = format!("Service {} security settings:\n", service_name);
        for (key, value) in security {
            result.push_str(&format!("{}: {}\n", key, value));
        }
        info!("Retrieved security settings for service {}", service_name);
        return Ok(result);
    }
}
