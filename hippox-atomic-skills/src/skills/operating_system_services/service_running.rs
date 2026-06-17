//! Service running skill

use super::common::list_running_services;
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceRunningSkill;

#[async_trait::async_trait]
impl Skill for ServiceRunningSkill {
    fn name(&self) -> &str {
        "service_running"
    }

    fn description(&self) -> &str {
        "List currently running services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which services are currently running."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
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
