//! Service unlock skill

use super::common::unlock_service_config;
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
pub struct ServiceUnlockSkill;

#[async_trait::async_trait]
impl Skill for ServiceUnlockSkill {
    fn name(&self) -> &str {
        "service_unlock"
    }

    fn description(&self) -> &str {
        "Unlock service configuration"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to unlock a service configuration and allow modifications."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_unlock",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx configuration unlocked".to_string()
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
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        unlock_service_config(service_name)?;
        Ok(format!("Service {} configuration unlocked", service_name))
    }
}
