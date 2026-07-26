//! Service start type Driver - get service start type
use super::common::get_service_start_type;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting service start type
#[derive(Debug)]
pub struct ServiceStartTypeDriver;
#[async_trait::async_trait]
impl Driver for ServiceStartTypeDriver {
    fn name(&self) -> &str {
        return "service_start_type";
    }
    fn description(&self) -> &str {
        return "Get the start type of a service (automatic/manual/disabled)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to check how a service is configured to start.";
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
            "action": "service_start_type",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx start type: automatic".to_string();
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
        debug!("Executing service_start_type driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let start_type = get_service_start_type(service_name).map_err(|e| {
            debug!("Failed to get start type for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get start type: {}", e));
        })?;
        if let Some(start_type) = start_type {
            info!("Service {} start type: {}", service_name, start_type);
            return Ok(format!("Service {} start type: {}", service_name, start_type));
        } else {
            info!("No start type information available for service {}", service_name);
            return Ok(format!("No start type information available for service {}", service_name));
        }
    }
}
