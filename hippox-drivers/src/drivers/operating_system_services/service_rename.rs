//! Service rename Driver - rename existing service
use super::common::rename_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for renaming a service
#[derive(Debug)]
pub struct ServiceRenameDriver;
#[async_trait::async_trait]
impl Driver for ServiceRenameDriver {
    fn name(&self) -> &str {
        return "service_rename";
    }
    fn description(&self) -> &str {
        return "Rename an existing service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to rename a service.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "old_name".to_string(),
                param_type: "string".to_string(),
                description: "Current service name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "new_name".to_string(),
                param_type: "string".to_string(),
                description: "New service name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("webserver".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_rename",
            "parameters": {
                "old_name": "nginx",
                "new_name": "webserver"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx renamed to webserver".to_string();
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
        debug!("Executing service_rename driver");
        let old_name = parameters.get("old_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'old_name' parameter");
            return DriverError::missing_parameter("old_name");
        })?;
        let new_name = parameters.get("new_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'new_name' parameter");
            return DriverError::missing_parameter("new_name");
        })?;
        rename_service(old_name, new_name).map_err(|e| {
            debug!("Failed to rename service from {} to {}: {}", old_name, new_name, e);
            return DriverError::execution(format!("Failed to rename service: {}", e));
        })?;
        info!("Service {} renamed to {}", old_name, new_name);
        return Ok(format!("Service {} renamed to {}", old_name, new_name));
    }
}
