//! Service PID Driver - get service process ID
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{debug, info};
use super::common::get_service_pid;
use crate::{
    types::{Driver, DriverParameter},
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
};
/// Driver for getting service PID
#[derive(Debug)]
pub struct ServicePidDriver;
#[async_trait::async_trait]
impl Driver for ServicePidDriver {
    fn name(&self) -> &str {
        return "service_pid";
    }
    fn description(&self) -> &str {
        return "Get the PID (Process ID) of a service";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to get the main process ID of a service.";
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
            "action": "service_pid",
            "parameters": {
                "service_name": "nginx"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx PID: 1234".to_string();
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
        debug!("Executing service_pid driver");
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'service_name' parameter");
                return DriverError::missing_parameter("service_name");
            })?;
        let pid = get_service_pid(service_name).map_err(|e| {
            debug!("Failed to get PID for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to get PID: {}", e));
        })?;
        if let Some(pid) = pid {
            info!("Service {} PID: {}", service_name, pid);
            return Ok(format!("Service {} PID: {}", service_name, pid));
        } else {
            info!("Service {} is not running or no PID available", service_name);
            return Ok(format!(
                "Service {} is not running or no PID available",
                service_name
            ));
        }
    }
}