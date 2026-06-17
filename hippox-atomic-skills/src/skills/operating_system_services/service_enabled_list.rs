//! Service enabled list skill

use super::common::list_enabled_services;
use crate::{SkillCallback, SkillContext};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceEnabledListSkill;

#[async_trait::async_trait]
impl Skill for ServiceEnabledListSkill {
    fn name(&self) -> &str {
        "service_enabled_list"
    }

    fn description(&self) -> &str {
        "List services that start automatically on boot"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which services are configured to start at boot."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_enabled_list"
        })
    }

    fn example_output(&self) -> String {
        "Enabled services (auto-start):\n1. ssh\n2. systemd-logind".to_string()
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
        let services = list_enabled_services()?;
        if services.is_empty() {
            return Ok("No enabled services found".to_string());
        }
        let mut result = format!("Enabled services (auto-start):\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, svc.name));
        }
        Ok(result)
    }
}
