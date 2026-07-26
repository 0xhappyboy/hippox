//! Service uptime Driver - get service uptime
use super::common::get_service_uptime;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting service uptime
#[derive(Debug)]
pub struct ServiceUptimeDriver;
#[async_trait::async_trait]
impl Driver for ServiceUptimeDriver {
    fn name(&self) -> &str {
        return "service_uptime";
    }
    fn description(&self) -> &str {
        return "Get the uptime of a service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see how long a service has been running.";
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
            "action": "service_uptime",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx uptime: 2 days, 3 hours, 15 minutes".to_string();
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
        debug!("Executing service_uptime driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let uptime = get_service_uptime(service_name).map_err(|e| {
            debug!("Failed to get uptime for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get uptime: {}", e));
        })?;
        if let Some(uptime) = uptime {
            info!("Service {} uptime: {}", service_name, uptime);
            return Ok(format!("Service {} uptime: {}", service_name, uptime));
        } else {
            info!("Service {} is not running or no uptime information available", service_name);
            return Ok(format!("Service {} is not running or no uptime information available", service_name));
        }
    }
}
