//! Service running Driver - list currently running services
use super::common::list_running_services;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing running services
#[derive(Debug)]
pub struct ServiceRunningDriver;
#[async_trait::async_trait]
impl Driver for ServiceRunningDriver {
    fn name(&self) -> &str {
        return "service_running";
    }
    fn description(&self) -> &str {
        return "List currently running services";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see which services are currently running.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_running"
        }));
    }
    fn example_output(&self) -> String {
        return "Running services:\n1. ssh - SSH Server\n2. systemd-logind".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemServices;
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing service_running driver");
        let services = list_running_services().map_err(|e| {
            debug!("Failed to list running services: {}", e);
            return DriverError::execution(format!("Failed to list running services: {}", e));
        })?;
        if services.is_empty() {
            info!("No running services found");
            return Ok("No running services found".to_string());
        }
        let mut result = format!("Running services:\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {} - {}\n", i + 1, svc.name, svc.description));
        }
        info!("Found {} running services", services.len());
        return Ok(result);
    }
}
