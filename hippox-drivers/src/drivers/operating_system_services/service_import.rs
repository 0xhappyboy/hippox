//! Service import Driver - import service configuration from file
use super::common::import_service_config;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for importing service configuration
#[derive(Debug)]
pub struct ServiceImportDriver;
#[async_trait::async_trait]
impl Driver for ServiceImportDriver {
    fn name(&self) -> &str {
        return "service_import";
    }
    fn description(&self) -> &str {
        return "Import service configuration from file";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to restore a service's configuration from a backup.";
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
                name: "input_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to import configuration from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/nginx.service.backup".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_import",
            "parameters": {
                "service_name": "nginx",
                "input_path": "/tmp/nginx.service.backup"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx configuration imported from /tmp/nginx.service.backup".to_string();
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
        debug!("Executing service_import driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let input_path = parameters.get("input_path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'input_path' parameter");
            return DriverError::missing_parameter("input_path");
        })?;
        import_service_config(service_name, input_path).map_err(|e| {
            debug!("Failed to import service {} from {}: {}", service_name, input_path, e);
            return DriverError::execution(format!("Failed to import service: {}", e));
        })?;
        info!("Service {} configuration imported from {}", service_name, input_path);
        return Ok(format!("Service {} configuration imported from {}", service_name, input_path));
    }
}
