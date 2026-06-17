//! Service stdout/stderr skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_logs;
use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct ServiceStdoutSkill;

#[async_trait::async_trait]
impl Skill for ServiceStdoutSkill {
    fn name(&self) -> &str {
        "service_stdout"
    }

    fn description(&self) -> &str {
        "View service standard output/error"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to view stdout/stderr output from a service."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "service_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "lines".to_string(),
                param_type: "integer".to_string(),
                description: "Number of lines to show (default: 50)".to_string(),
                required: false,
                default: Some(Value::Number(50.into())),
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_stdout",
            "parameters": {
                "service_name": "nginx",
                "lines": 50
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx stdout/stderr:\n[2024-01-01 00:00:00] Started service\n[2024-01-01 00:00:01] Listening on port 80".to_string()
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
        let lines = parameters
            .get("lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;
        let logs = get_service_logs(service_name, lines)?;
        if logs.is_empty() {
            return Ok(format!(
                "No stdout/stderr output found for service {}",
                service_name
            ));
        }
        let mut result = format!("Service {} stdout/stderr:\n", service_name);
        for entry in logs {
            result.push_str(&format!("[{}] {}\n", entry.timestamp, entry.message));
        }
        Ok(result)
    }
}
