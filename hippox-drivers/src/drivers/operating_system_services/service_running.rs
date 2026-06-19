//! Service running Driver

use super::common::list_running_services;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceRunningDriver;

#[async_trait::async_trait]
impl Driver for ServiceRunningDriver {
    fn name(&self) -> &str {
        "service_running"
    }

    fn description(&self) -> &str {
        "List currently running services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which services are currently running."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_running"
        })
    }

    fn example_output(&self) -> String {
        "Running services:\n1. ssh - SSH Server\n2. systemd-logind".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemServices
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let services = list_running_services()?;
        if services.is_empty() {
            return Ok("No running services found".to_string());
        }
        let mut result = format!("Running services:\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {} - {}\n", i + 1, svc.name, svc.description));
        }
        Ok(result)
    }
}
