//! Process running check skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
    operating_system_process::common::is_process_running,
};

/// Skill for checking if a process is running
#[derive(Debug)]
pub struct ProcessIsRunningSkill;

#[async_trait::async_trait]
impl Skill for ProcessIsRunningSkill {
    fn name(&self) -> &str {
        "process_is_running"
    }

    fn description(&self) -> &str {
        "Check if a process with the given name is running"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to verify if a service or application is currently running"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to check".to_string(),
                required: true,
                default: None,
                example: Some(json!("nginx")),
                enum_values: None,
            },
            SkillParameter {
                name: "exact_match".to_string(),
                param_type: "boolean".to_string(),
                description: "Require exact name match (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_is_running",
            "parameters": {
                "name": "docker"
            }
        })
    }

    fn example_output(&self) -> String {
        "Process 'docker' is running".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemProcess
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let exact_match = parameters
            .get("exact_match")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let running = is_process_running(name, exact_match);
        if running {
            Ok(format!("Process '{}' is running", name))
        } else {
            Ok(format!("Process '{}' is not running", name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_is_running_skill() {
        let skill = ProcessIsRunningSkill;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("system"));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
    }
}