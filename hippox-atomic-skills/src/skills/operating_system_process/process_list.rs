//! Process listing skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
    operating_system_process::common::{get_all_processes, get_processes_by_filter, sort_processes, ProcessFilter, ProcessSortBy, format_memory},
};

/// Skill for listing running processes
#[derive(Debug)]
pub struct ProcessListSkill;

#[async_trait::async_trait]
impl Skill for ProcessListSkill {
    fn name(&self) -> &str {
        "process_list"
    }

    fn description(&self) -> &str {
        "List running processes on the system"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to see what processes are running, check for specific applications, or troubleshoot system performance"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Filter processes by name (partial match)".to_string(),
                required: false,
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
                name: "top_n".to_string(),
                param_type: "integer".to_string(),
                description: "Show only top N processes by CPU usage".to_string(),
                required: false,
                default: None,
                example: Some(json!(10)),
                enum_values: None,
            },
            SkillParameter {
                name: "sort_by".to_string(),
                param_type: "string".to_string(),
                description: "Sort by: cpu, memory, name, or pid".to_string(),
                required: false,
                default: Some(json!("cpu")),
                example: Some(json!("memory")),
                enum_values: Some(vec![
                    "cpu".to_string(),
                    "memory".to_string(),
                    "name".to_string(),
                    "pid".to_string(),
                ]),
            },
            SkillParameter {
                name: "min_cpu".to_string(),
                param_type: "number".to_string(),
                description: "Minimum CPU usage percentage to include".to_string(),
                required: false,
                default: None,
                example: Some(json!(5.0)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_list",
            "parameters": {
                "sort_by": "memory",
                "top_n": 5
            }
        })
    }

    fn example_output(&self) -> String {
        "PID     NAME                          CPU%    MEMORY  \n1234    chrome                        2.5     256.3 MB\n5678    code                          1.2     180.2 MB".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemProcess
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let filter_name = parameters.get("filter").and_then(|v| v.as_str());
        let exact_match = parameters
            .get("exact_match")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let top_n = parameters
            .get("top_n")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);
        let sort_by = parameters
            .get("sort_by")
            .and_then(|v| v.as_str())
            .unwrap_or("cpu");
        let min_cpu = parameters
            .get("min_cpu")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        // Build filter
        let filter = ProcessFilter {
            name: filter_name.map(|s| s.to_string()),
            exact_match,
            min_cpu,
            min_memory: None,
            status: None,
        };

        let mut processes = if filter_name.is_some() || min_cpu.is_some() {
            get_processes_by_filter(&filter)
        } else {
            get_all_processes()
        };

        // Sort
        let sort_enum = match sort_by {
            "memory" => ProcessSortBy::Memory,
            "name" => ProcessSortBy::Name,
            "pid" => ProcessSortBy::Pid,
            _ => ProcessSortBy::Cpu,
        };
        sort_processes(&mut processes, sort_enum);

        // Limit
        if let Some(n) = top_n {
            processes.truncate(n);
        }

        if processes.is_empty() {
            return Ok("No matching processes found".to_string());
        }

        let mut output = vec![format!(
            "{:<8} {:<30} {:<10} {:<12} {:<10}",
            "PID", "NAME", "CPU%", "MEMORY", "STATUS"
        )];
        output.push("-".repeat(74));

        for p in processes {
            let mem_str = format_memory(p.memory);
            output.push(format!(
                "{:<8} {:<30} {:<10.1} {:<12} {:<10}",
                p.pid,
                p.name,
                p.cpu_usage,
                mem_str,
                p.status
            ));
        }

        Ok(output.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_list() {
        let skill = ProcessListSkill;
        let params = HashMap::new();
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("PID") || output.contains("No matching processes"));
    }

    #[tokio::test]
    async fn test_process_list_with_filter() {
        let skill = ProcessListSkill;
        let mut params = HashMap::new();
        params.insert("filter".to_string(), json!("system"));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_list_with_top_n() {
        let skill = ProcessListSkill;
        let mut params = HashMap::new();
        params.insert("top_n".to_string(), json!(5));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
    }
}