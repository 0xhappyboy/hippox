//! Service masked list Driver - list all masked services
use super::common::list_masked_services;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing masked services
#[derive(Debug)]
pub struct ServiceMaskedListDriver;
#[async_trait::async_trait]
impl Driver for ServiceMaskedListDriver {
    fn name(&self) -> &str {
        return "service_masked_list";
    }
    fn description(&self) -> &str {
        return "List all masked services";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see which services are currently masked.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_masked_list"
        }));
    }
    fn example_output(&self) -> String {
        return "Masked services:\n1. service1\n2. service2".to_string();
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
        debug!("Executing service_masked_list driver");
        let services = list_masked_services().map_err(|e| {
            debug!("Failed to list masked services: {}", e);
            return DriverError::execution(format!("Failed to list masked services: {}", e));
        })?;
        if services.is_empty() {
            info!("No masked services found");
            return Ok("No masked services found".to_string());
        }
        let mut result = format!("Masked services:\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, svc));
        }
        info!("Found {} masked services", services.len());
        return Ok(result);
    }
}
