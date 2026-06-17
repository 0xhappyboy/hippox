//! Service masked list skill

use super::common::list_masked_services;
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
pub struct ServiceMaskedListSkill;

#[async_trait::async_trait]
impl Skill for ServiceMaskedListSkill {
    fn name(&self) -> &str {
        "service_masked_list"
    }

    fn description(&self) -> &str {
        "List all masked services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which services are currently masked."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_masked_list"
        })
    }

    fn example_output(&self) -> String {
        "Masked services:\n1. service1\n2. service2".to_string()
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
        let services = list_masked_services()?;
        if services.is_empty() {
            return Ok("No masked services found".to_string());
        }
        let mut result = format!("Masked services:\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, svc));
        }
        Ok(result)
    }
}
