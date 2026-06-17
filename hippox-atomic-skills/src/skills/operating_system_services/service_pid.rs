//! Service PID skill

use super::common::get_service_pid;
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
pub struct ServicePidSkill;

#[async_trait::async_trait]
impl Skill for ServicePidSkill {
    fn name(&self) -> &str {
        "service_pid"
    }

    fn description(&self) -> &str {
        "Get the PID (Process ID) of a service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the main process ID of a service."
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
            "action": "service_pid",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx PID: 1234".to_string()
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
        let pid = get_service_pid(service_name)?;
        if let Some(pid) = pid {
            Ok(format!("Service {} PID: {}", service_name, pid))
        } else {
            Ok(format!(
                "Service {} is not running or no PID available",
                service_name
            ))
        }
    }
}
