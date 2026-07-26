//! Service dependencies Driver - list service dependencies
use super::common::get_service_dependencies;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing service dependencies
#[derive(Debug)]
pub struct ServiceDependenciesDriver;
#[async_trait::async_trait]
impl Driver for ServiceDependenciesDriver {
    fn name(&self) -> &str {
        return "service_dependencies";
    }
    fn description(&self) -> &str {
        return "List dependencies of a system service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see what other services a service depends on.";
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
            "action": "service_dependencies",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx depends on:\n1. network.target\n2. systemd-journald.service".to_string();
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
        debug!("Executing service_dependencies driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let deps = get_service_dependencies(service_name).map_err(|e| {
            debug!("Failed to get dependencies for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get dependencies: {}", e));
        })?;
        if deps.is_empty() {
            info!("Service {} has no dependencies", service_name);
            return Ok(format!("Service {} has no dependencies", service_name));
        }
        let mut result = format!("Service {} depends on:\n", service_name);
        for (i, dep) in deps.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, dep.dependency_name));
        }
        info!("Found {} dependencies for service {}", deps.len(), service_name);
        return Ok(result);
    }
}
