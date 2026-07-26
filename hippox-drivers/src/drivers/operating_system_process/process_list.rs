//! Process listing driver
//!
//! This driver provides functionality to list running processes on the system
//! with filtering, sorting, and limiting options.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_process::common::{ProcessFilter, ProcessSortBy, format_memory, get_all_processes, get_processes_by_filter, sort_processes},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing running processes
#[derive(Debug)]
pub struct ProcessListDriver;
#[async_trait::async_trait]
impl Driver for ProcessListDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "process_list"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List running processes on the system"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to see what processes are running, check for specific applications, or troubleshoot system performance"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Filter processes by name (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(json!("python")),
                enum_values: None,
            },
            DriverParameter {
                name: "exact_match".to_string(),
                param_type: "boolean".to_string(),
                description: "Require exact name match (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "top_n".to_string(),
                param_type: "integer".to_string(),
                description: "Show only top N processes by CPU usage".to_string(),
                required: false,
                default: None,
                example: Some(json!(10)),
                enum_values: None,
            },
            DriverParameter {
                name: "sort_by".to_string(),
                param_type: "string".to_string(),
                description: "Sort by: cpu, memory, name, or pid".to_string(),
                required: false,
                default: Some(json!("cpu")),
                example: Some(json!("memory")),
                enum_values: Some(vec!["cpu".to_string(), "memory".to_string(), "name".to_string(), "pid".to_string()]),
            },
            DriverParameter {
                name: "min_cpu".to_string(),
                param_type: "number".to_string(),
                description: "Minimum CPU usage percentage to include".to_string(),
                required: false,
                default: None,
                example: Some(json!(5.0)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "process_list",
            "parameters": {
                "sort_by": "memory",
                "top_n": 5
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "PID     NAME                          CPU%    MEMORY  \n1234    chrome                        2.5     256.3 MB\n5678    code                          1.2     180.2 MB".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemProcess;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing process_list driver");
        let filter_name = parameters.get("filter").and_then(|v| v.as_str());
        let exact_match = parameters.get("exact_match").and_then(|v| v.as_bool()).unwrap_or(false);
        let top_n = parameters.get("top_n").and_then(|v| v.as_u64()).map(|n| n as usize);
        let sort_by = parameters.get("sort_by").and_then(|v| v.as_str()).unwrap_or("cpu");
        let min_cpu = parameters.get("min_cpu").and_then(|v| v.as_f64()).map(|v| v as f32);
        // Build filter
        let filter = ProcessFilter { name: filter_name.map(|s| s.to_string()), exact_match, min_cpu, min_memory: None, status: None };
        info!("Listing processes with filter: {:?}, sort_by: {}, top_n: {:?}", filter, sort_by, top_n);
        let mut processes = if filter_name.is_some() || min_cpu.is_some() { get_processes_by_filter(&filter) } else { get_all_processes() };
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
            info!("No matching processes found");
            return Ok("No matching processes found".to_string());
        }
        info!("Found {} processes matching criteria", processes.len());
        let mut output = vec![format!("{:<8} {:<30} {:<10} {:<12} {:<10}", "PID", "NAME", "CPU%", "MEMORY", "STATUS")];
        output.push("-".repeat(74));
        for p in processes {
            let mem_str = format_memory(p.memory);
            output.push(format!("{:<8} {:<30} {:<10.1} {:<12} {:<10}", p.pid, p.name, p.cpu_usage, mem_str, p.status));
        }
        return Ok(output.join("\n"));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_process_list() {
        let skill = ProcessListDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("PID") || output.contains("No matching processes"));
    }
    #[tokio::test]
    async fn test_process_list_with_filter() {
        let skill = ProcessListDriver;
        let mut params = HashMap::new();
        params.insert("filter".to_string(), json!("system"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_process_list_with_top_n() {
        let skill = ProcessListDriver;
        let mut params = HashMap::new();
        params.insert("top_n".to_string(), json!(5));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
    }
}
