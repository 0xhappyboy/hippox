// application_control/application_control_list_running.rs
//! Application list running skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::list_running_processes;

#[derive(Debug)]
pub struct ApplicationControlListRunningSkill;

#[async_trait::async_trait]
impl Skill for ApplicationControlListRunningSkill {
    fn name(&self) -> &str {
        "application_control_list_running"
    }

    fn description(&self) -> &str {
        "List all currently running applications/processes"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what applications are currently active on the system."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Optional filter to narrow results".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("chrome".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of results to return".to_string(),
                required: false,
                default: Some(Value::Number(50.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_list_running"
        })
    }

    fn example_output(&self) -> String {
        "Found 127 processes:\n1. System (PID: 4)\n2. notepad.exe (PID: 12345)\n...".to_string()
    }

    fn category(&self) -> &str {
        "application_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let filter = parameters.get("filter").and_then(|v| v.as_str());
        let limit = parameters.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;
        
        let mut processes = list_running_processes()?;
        
        if let Some(f) = filter {
            processes.retain(|p| p.name.to_lowercase().contains(&f.to_lowercase()));
        }
        
        processes.truncate(limit);
        
        if processes.is_empty() {
            return Ok("No processes found".to_string());
        }
        
        let mut result = format!("Found {} processes:\n", processes.len());
        for (i, proc) in processes.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} (PID: {})\n",
                i + 1,
                proc.name,
                proc.pid
            ));
        }
        
        Ok(result)
    }
}