//! Service enabled list Driver - list services that start automatically on boot
use super::common::list_enabled_services;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing enabled services
#[derive(Debug)]
pub struct ServiceEnabledListDriver;
#[async_trait::async_trait]
impl Driver for ServiceEnabledListDriver {
    fn name(&self) -> &str {
        return "service_enabled_list";
    }
    fn description(&self) -> &str {
        return "List services that start automatically on boot";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see which services are configured to start at boot.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_enabled_list"
        }));
    }
    fn example_output(&self) -> String {
        return "Enabled services (auto-start):\n1. ssh\n2. systemd-logind".to_string();
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
        debug!("Executing service_enabled_list driver");
        let services = list_enabled_services().map_err(|e| {
            debug!("Failed to list enabled services: {}", e);
            return DriverError::execution(format!("Failed to list enabled services: {}", e));
        })?;
        if services.is_empty() {
            info!("No enabled services found");
            return Ok("No enabled services found".to_string());
        }
        let mut result = format!("Enabled services (auto-start):\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, svc.name));
        }
        info!("Found {} enabled services", services.len());
        return Ok(result);
    }
}
