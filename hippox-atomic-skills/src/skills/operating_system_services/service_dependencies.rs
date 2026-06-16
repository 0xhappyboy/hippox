//! Service dependencies skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_dependencies;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceDependenciesSkill;

#[async_trait::async_trait]
impl Skill for ServiceDependenciesSkill {
    fn name(&self) -> &str {
        "service_dependencies"
    }

    fn description(&self) -> &str {
        "List dependencies of a system service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what other services a service depends on."
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
            "action": "service_dependencies",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx depends on:\n1. network.target\n2. systemd-journald.service".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let deps = get_service_dependencies(service_name)?;
        if deps.is_empty() {
            return Ok(format!("Service {} has no dependencies", service_name));
        }
        let mut result = format!("Service {} depends on:\n", service_name);
        for (i, dep) in deps.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, dep.dependency_name));
        }
        Ok(result)
    }
}