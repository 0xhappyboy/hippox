//! Service resources skill

use super::common::get_service_resources;
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
pub struct ServiceResourcesSkill;

#[async_trait::async_trait]
impl Skill for ServiceResourcesSkill {
    fn name(&self) -> &str {
        "service_resources"
    }

    fn description(&self) -> &str {
        "View service resource usage (CPU, memory)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check a service's CPU and memory usage."
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
            "action": "service_resources",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx resource usage:\nCPU: 0.5%\nMemory: 1024 KB".to_string()
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
        let (cpu, mem) = get_service_resources(service_name)?;
        let mut result = format!("Service {} resource usage:\n", service_name);
        if let Some(cpu) = cpu {
            result.push_str(&format!("CPU: {}%\n", cpu));
        } else {
            result.push_str("CPU: N/A\n");
        }
        if let Some(mem) = mem {
            result.push_str(&format!("Memory: {} KB", mem));
        } else {
            result.push_str("Memory: N/A");
        }
        Ok(result)
    }
}
