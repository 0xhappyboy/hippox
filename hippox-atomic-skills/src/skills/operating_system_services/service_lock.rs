//! Service lock skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::SkillCallback;
use crate::SkillContext;
use super::common::lock_service_config;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct ServiceLockSkill;

#[async_trait::async_trait]
impl Skill for ServiceLockSkill {
    fn name(&self) -> &str {
        "service_lock"
    }

    fn description(&self) -> &str {
        "Lock service configuration to prevent modifications"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to lock a service configuration and prevent accidental changes."
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
            "action": "service_lock",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx configuration locked".to_string()
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
        lock_service_config(service_name)?;
        Ok(format!("Service {} configuration locked", service_name))
    }
}
