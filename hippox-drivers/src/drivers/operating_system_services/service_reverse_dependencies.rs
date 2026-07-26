//! Service reverse dependencies Driver - list services that depend on this service
use super::common::get_reverse_dependencies;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing reverse dependencies
#[derive(Debug)]
pub struct ServiceReverseDependenciesDriver;
#[async_trait::async_trait]
impl Driver for ServiceReverseDependenciesDriver {
    fn name(&self) -> &str {
        return "service_reverse_dependencies";
    }
    fn description(&self) -> &str {
        return "List services that depend on this service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see which other services depend on this service.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("network.target".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_reverse_dependencies",
            "parameters": {
                "service_name": "network.target"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Services depending on network.target:\n1. ssh\n2. nginx".to_string();
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
        debug!("Executing service_reverse_dependencies driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let deps = get_reverse_dependencies(service_name).map_err(|e| {
            debug!("Failed to get reverse dependencies for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get reverse dependencies: {}", e));
        })?;
        if deps.is_empty() {
            info!("No services depend on {}", service_name);
            return Ok(format!("No services depend on {}", service_name));
        }
        let mut result = format!("Services depending on {}:\n", service_name);
        for (i, dep) in deps.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, dep));
        }
        info!("Found {} reverse dependencies for service {}", deps.len(), service_name);
        return Ok(result);
    }
}
