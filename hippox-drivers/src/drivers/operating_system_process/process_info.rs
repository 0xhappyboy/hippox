//! Detailed process information driver
//!
//! This driver provides functionality to get detailed information about a process.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_process::common::{format_memory, get_process_by_pid},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting detailed information about a process
#[derive(Debug)]
pub struct ProcessInfoDriver;
#[async_trait::async_trait]
impl Driver for ProcessInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "process_info"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get detailed information about a process"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need detailed process metrics like CPU, memory, disk I/O"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "pid".to_string(),
            param_type: "integer".to_string(),
            description: "Process ID".to_string(),
            required: true,
            default: None,
            example: Some(json!(1)),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "process_info",
            "parameters": {
                "pid": 1
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Process: systemd\nPID: 1\nCPU: 0.1%\nMemory: 12.5 MB\nStatus: Running\nStart Time: 2024-01-15 10:30:00".to_string();
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
        debug!("Executing process_info driver");
        let pid = parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))? as u32;
        info!("Getting info for process PID: {}", pid);
        let process = get_process_by_pid(pid).ok_or_else(|| DriverError::execution(format!("Process with PID {} not found", pid)))?;
        let mut info = Vec::new();
        info.push(format!("Process: {}", process.name));
        info.push(format!("PID: {}", process.pid));
        info.push(format!("Parent PID: {}", process.parent_pid.map(|p| p.to_string()).unwrap_or_else(|| "None".to_string())));
        info.push(format!("CPU Usage: {:.1}%", process.cpu_usage));
        info.push(format!("Memory: {}", format_memory(process.memory)));
        info.push(format!("Virtual Memory: {}", format_memory(process.virtual_memory)));
        info.push(format!("Status: {}", process.status));
        if let Some(start_time) = process.start_time {
            #[cfg(not(target_os = "windows"))]
            {
                use std::time::SystemTime;
                let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
                let uptime = duration.as_secs() - start_time;
                let days = uptime / 86400;
                let hours = (uptime % 86400) / 3600;
                let minutes = (uptime % 3600) / 60;
                let mut parts = Vec::new();
                if days > 0 {
                    parts.push(format!("{}d", days));
                }
                if hours > 0 {
                    parts.push(format!("{}h", hours));
                }
                if minutes > 0 {
                    parts.push(format!("{}m", minutes));
                }
                if parts.is_empty() {
                    parts.push(format!("{}s", uptime));
                }
                info.push(format!("Uptime: {}", parts.join(" ")));
            }
        }
        info!("Process info retrieved for PID: {}", pid);
        return Ok(info.join("\n"));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_process_info_invalid_pid() {
        let skill = ProcessInfoDriver;
        let mut params = HashMap::new();
        params.insert("pid".to_string(), json!(99999999));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_err());
    }
}
