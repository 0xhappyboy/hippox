//! Service list skill - list all system services

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::list_all_services;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceListSkill;

#[async_trait::async_trait]
impl Skill for ServiceListSkill {
    fn name(&self) -> &str {
        "service_list"
    }

    fn description(&self) -> &str {
        "List all system services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see all services on the system."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_list"
        })
    }

    fn example_output(&self) -> String {
        "Found 5 services:\n1. ssh - SSH Server (running)\n2. nginx - Web Server (stopped)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let services = list_all_services()?;
        if services.is_empty() {
            return Ok("No services found".to_string());
        }
        let mut result = format!("Found {} services:\n", services.len());
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