//! Service export Driver - export service configuration to file
use super::common::export_service_config;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for exporting service configuration
#[derive(Debug)]
pub struct ServiceExportDriver;
#[async_trait::async_trait]
impl Driver for ServiceExportDriver {
    fn name(&self) -> &str {
        return "service_export";
    }
    fn description(&self) -> &str {
        return "Export service configuration to file";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to backup a service's configuration.";
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
                name: "output_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to export configuration to".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/nginx.service.backup".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_export",
            "parameters": {
                "service_name": "nginx",
                "output_path": "/tmp/nginx.service.backup"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx configuration exported to /tmp/nginx.service.backup".to_string();
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
        debug!("Executing service_export driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let output_path = parameters.get("output_path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'output_path' parameter");
            return DriverError::missing_parameter("output_path");
        })?;
        export_service_config(service_name, output_path).map_err(|e| {
            debug!("Failed to export service {} to {}: {}", service_name, output_path, e);
            return DriverError::execution(format!("Failed to export service: {}", e));
        })?;
        info!("Service {} configuration exported to {}", service_name, output_path);
        return Ok(format!("Service {} configuration exported to {}", service_name, output_path));
    }
}
