//! Service resources Driver - view service CPU and memory usage
use super::common::get_service_resources;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for viewing service resource usage
#[derive(Debug)]
pub struct ServiceResourcesDriver;
#[async_trait::async_trait]
impl Driver for ServiceResourcesDriver {
    fn name(&self) -> &str {
        return "service_resources";
    }
    fn description(&self) -> &str {
        return "View service resource usage (CPU, memory)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to check a service's CPU and memory usage.";
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
            "action": "service_resources",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx resource usage:\nCPU: 0.5%\nMemory: 1024 KB".to_string();
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
        debug!("Executing service_resources driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let (cpu, mem) = get_service_resources(service_name).map_err(|e| {
            debug!("Failed to get resources for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get resources: {}", e));
        })?;
        let mut result = format!("Service {} resource usage:\n", service_name);
        if let Some(cpu) = cpu {
            result.push_str(&format!("CPU: {}%\n", cpu));
        } else {
            result.push_str("CPU: N/A\n");
        }
        if let Some(mem) = mem {
            result.push_str(&format!("Memory: {} KB", mem));
        } else {
            result.push_str("Memory: N/A");
        }
        info!("Retrieved resource usage for service {}", service_name);
        return Ok(result);
    }
}
