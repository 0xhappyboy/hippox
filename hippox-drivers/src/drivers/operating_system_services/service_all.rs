//! Service all Driver

use crate::{DriverCallback, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::list_all_services;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceAllDriver;

#[async_trait::async_trait]
impl Driver for ServiceAllDriver {
    fn name(&self) -> &str {
        "service_all"
    }

    fn description(&self) -> &str {
        "List all services including stopped and exited ones"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see all services, including those that are stopped or exited."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_all"
        })
    }

    fn example_output(&self) -> String {
        "All services:\n1. ssh - SSH Server (running)\n2. nginx - Web Server (stopped)".to_string()
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
        let services = list_all_services()?;
        if services.is_empty() {
            return Ok("No services found".to_string());
        }
        let mut result = format!("All services:\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} - {} ({})\n",
                i + 1,
                svc.name,
                svc.description,
                svc.status
            ));
        }
        Ok(result)
    }
}
