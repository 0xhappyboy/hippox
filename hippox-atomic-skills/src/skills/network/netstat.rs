use crate::common::net::get_network_connections;
use crate::types::{Skill, SkillParameter};
use crate::{SkillCallback, SkillCategory, SkillContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct NetstatSkill;

#[async_trait::async_trait]
impl Skill for NetstatSkill {
    fn name(&self) -> &str {
        "netstat"
    }

    fn description(&self) -> &str {
        "View network connections and listening ports on the local system"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to check what ports are open or what connections are active on the local machine"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "netstat"
        })
    }

    fn example_output(&self) -> String {
        "Network connections:\nlocal: 0.0.0.0:22 remote: 0.0.0.0:* state: LISTEN\nlocal: 127.0.0.1:5432 remote: 0.0.0.0:* state: LISTEN".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Getting network connections".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        let connections = get_network_connections()?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Found {} connections", connections.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
        }
        let mut output = format!("Network connections ({}):\n", connections.len());
        for conn in connections.clone() {
            let parts: Vec<String> = conn.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
            output.push_str(&format!("  {}\n", parts.join(" ")));
        }
        if connections.is_empty() {
            output.push_str("  No connections found\n");
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Connections retrieved".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("netstat".to_string()),
                Some(output.clone()),
            );
        }
        Ok(output)
    }
}
