//! Service rename skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::rename_service;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceRenameSkill;

#[async_trait::async_trait]
impl Skill for ServiceRenameSkill {
    fn name(&self) -> &str {
        "service_rename"
    }

    fn description(&self) -> &str {
        "Rename an existing service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to rename a service."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "old_name".to_string(),
                param_type: "string".to_string(),
                description: "Current service name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "new_name".to_string(),
                param_type: "string".to_string(),
                description: "New service name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("webserver".to_string())),
                enum_values: None,
            }
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_rename",
            "parameters": {
                "old_name": "nginx",
                "new_name": "webserver"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx renamed to webserver".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let old_name = parameters
            .get("old_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'old_name' parameter"))?;
        let new_name = parameters
            .get("new_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'new_name' parameter"))?;
        rename_service(old_name, new_name)?;
        Ok(format!("Service {} renamed to {}", old_name, new_name))
    }
}