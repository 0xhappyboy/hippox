//! Service list Driver - list all system services
use super::common::list_all_services;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing all system services
#[derive(Debug)]
pub struct ServiceListDriver;
#[async_trait::async_trait]
impl Driver for ServiceListDriver {
    fn name(&self) -> &str {
        return "service_list";
    }
    fn description(&self) -> &str {
        return "List all system services";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see all services on the system.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_list"
        }));
    }
    fn example_output(&self) -> String {
        return "Found 5 services:\n1. ssh - SSH Server (running)\n2. nginx - Web Server (stopped)".to_string();
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
        debug!("Executing service_list driver");
        let services = list_all_services().map_err(|e| {
            debug!("Failed to list all services: {}", e);
            return DriverError::execution(format!("Failed to list all services: {}", e));
        })?;
        if services.is_empty() {
            info!("No services found");
            return Ok("No services found".to_string());
        }
        let mut result = format!("Found {} services:\n", services.len());
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {} - {} ({})\n", i + 1, svc.name, svc.description, svc.status));
        }
        info!("Found {} services", services.len());
        return Ok(result);
    }
}
