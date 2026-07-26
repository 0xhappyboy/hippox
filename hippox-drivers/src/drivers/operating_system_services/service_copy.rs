//! Service copy Driver - copy service configuration to create new service
use super::common::copy_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for copying service configuration
#[derive(Debug)]
pub struct ServiceCopyDriver;
#[async_trait::async_trait]
impl Driver for ServiceCopyDriver {
    fn name(&self) -> &str {
        return "service_copy";
    }
    fn description(&self) -> &str {
        return "Copy a service configuration to create a new service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to duplicate an existing service configuration.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "source_service".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service to copy from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "dest_service".to_string(),
                param_type: "string".to_string(),
                description: "Name of the new service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_copy",
            "parameters": {
                "source_service": "nginx",
                "dest_service": "myapp"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service myapp copied from nginx".to_string();
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
        debug!("Executing service_copy driver");
        let source = parameters.get("source_service").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'source_service' parameter");
            return DriverError::missing_parameter("source_service");
        })?;
        let dest = parameters.get("dest_service").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'dest_service' parameter");
            return DriverError::missing_parameter("dest_service");
        })?;
        copy_service(source, dest).map_err(|e| {
            debug!("Failed to copy service from {} to {}: {}", source, dest, e);
            return DriverError::execution(format!("Failed to copy service: {}", e));
        })?;
        info!("Service {} copied from {}", dest, source);
        return Ok(format!("Service {} copied from {}", dest, source));
    }
}
