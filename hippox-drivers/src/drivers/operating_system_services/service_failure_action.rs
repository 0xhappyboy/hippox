//! Service failure action Driver - set action on service failure
use super::common::set_failure_action;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting service failure action
#[derive(Debug)]
pub struct ServiceFailureActionDriver;
#[async_trait::async_trait]
impl Driver for ServiceFailureActionDriver {
    fn name(&self) -> &str {
        return "service_failure_action";
    }
    fn description(&self) -> &str {
        return "Set action to take when service fails";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to configure what happens when a service fails. Options: restart, stop, ignore";
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
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action on failure: restart, stop, ignore".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("restart".to_string())),
                enum_values: Some(vec!["restart".to_string(), "stop".to_string(), "ignore".to_string()]),
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_failure_action",
            "parameters": {
                "service_name": "nginx",
                "action": "restart"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Service nginx failure action set to restart".to_string();
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
        debug!("Executing service_failure_action driver");
        let service_name = parameters.get("service_name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_name' parameter");
            return DriverError::missing_parameter("service_name");
        })?;
        let action = parameters.get("action").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'action' parameter");
            return DriverError::missing_parameter("action");
        })?;
        set_failure_action(service_name, action).map_err(|e| {
            debug!("Failed to set failure action for {}: {}", service_name, e);
            return DriverError::execution(format!("Failed to set failure action: {}", e));
        })?;
        info!("Service {} failure action set to {}", service_name, action);
        return Ok(format!("Service {} failure action set to {}", service_name, action));
    }
}
