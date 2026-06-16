//! Process PID retrieval skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
    operating_system_process::common::get_pids_by_name,
};

/// Skill for getting the PID of a process by name
#[derive(Debug)]
pub struct ProcessGetPidSkill;

#[async_trait::async_trait]
impl Skill for ProcessGetPidSkill {
    fn name(&self) -> &str {
        "process_get_pid"
    }

    fn description(&self) -> &str {
        "Get the PID(s) of a process by name"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need the process ID of an application for monitoring or management"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to find".to_string(),
                required: true,
                default: None,
                example: Some(json!("python")),
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
            SkillParameter {
                name: "first_only".to_string(),
                param_type: "boolean".to_string(),
                description: "Return only the first PID found (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_get_pid",
            "parameters": {
                "name": "sshd"
            }
        })
    }

    fn example_output(&self) -> String {
        "Found PIDs: 1234, 5678".to_string()
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

        let first_only = parameters
            .get("first_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let pids = get_pids_by_name(name, exact_match);

        if pids.is_empty() {
            Ok(format!("No process found matching '{}'", name))
        } else if first_only {
            Ok(format!("PID: {}", pids[0]))
        } else {
            Ok(format!(
                "Found PIDs: {}",
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_get_pid_skill() {
        let skill = ProcessGetPidSkill;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("system"));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_get_pid_first_only() {
        let skill = ProcessGetPidSkill;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("system"));
        params.insert("first_only".to_string(), json!(true));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("PID:"));
    }
}