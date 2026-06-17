//! Process termination by PID skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCallback, SkillCategory, SkillContext, operating_system_process::common::{get_process_by_pid, kill_process}, types::{Skill, SkillParameter}
};

/// Skill for terminating a process by PID
#[derive(Debug)]
pub struct ProcessKillSkill;

#[async_trait::async_trait]
impl Skill for ProcessKillSkill {
    fn name(&self) -> &str {
        "process_kill"
    }

    fn description(&self) -> &str {
        "Terminate a process by its PID"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to stop a misbehaving or unwanted process"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID to terminate".to_string(),
                required: true,
                default: None,
                example: Some(json!(1234)),
                enum_values: None,
            },
            SkillParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force kill (SIGKILL instead of SIGTERM)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_kill",
            "parameters": {
                "pid": 1234
            }
        })
    }

    fn example_output(&self) -> String {
        "Process 1234 terminated successfully".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemProcess
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;

        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check if process exists
        if get_process_by_pid(pid).is_none() {
            return Err(anyhow::anyhow!("Process with PID {} not found", pid));
        }

        kill_process(pid, force).map_err(|e| anyhow::anyhow!(e))?;
        Ok(format!("Process {} terminated successfully", pid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_kill_invalid_pid() {
        let skill = ProcessKillSkill;
        let mut params = HashMap::new();
        params.insert("pid".to_string(), json!(99999999));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_err());
    }
}
