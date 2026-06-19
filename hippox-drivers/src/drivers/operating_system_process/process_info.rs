//! Detailed process information skill

use crate::{DriverCallback, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    DriverCategory,
    operating_system_process::common::{format_memory, get_process_by_pid},
    types::{Driver, DriverParameter},
};

/// Driver for getting detailed information about a process
#[derive(Debug)]
pub struct ProcessInfoDriver;

#[async_trait::async_trait]
impl Driver for ProcessInfoDriver {
    fn name(&self) -> &str {
        "process_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a process"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need detailed process metrics like CPU, memory, disk I/O"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "pid".to_string(),
            param_type: "integer".to_string(),
            description: "Process ID".to_string(),
            required: true,
            default: None,
            example: Some(json!(1)),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_info",
            "parameters": {
                "pid": 1
            }
        })
    }

    fn example_output(&self) -> String {
        "Process: systemd\nPID: 1\nCPU: 0.1%\nMemory: 12.5 MB\nStatus: Running\nStart Time: 2024-01-15 10:30:00".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemProcess
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;

        let process = get_process_by_pid(pid)
            .ok_or_else(|| anyhow::anyhow!("Process with PID {} not found", pid))?;

        let mut info = Vec::new();
        info.push(format!("Process: {}", process.name));
        info.push(format!("PID: {}", process.pid));
        info.push(format!(
            "Parent PID: {}",
            process
                .parent_pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "None".to_string())
        ));
        info.push(format!("CPU Usage: {:.1}%", process.cpu_usage));
        info.push(format!("Memory: {}", format_memory(process.memory)));
        info.push(format!(
            "Virtual Memory: {}",
            format_memory(process.virtual_memory)
        ));
        info.push(format!("Status: {}", process.status));

        if let Some(start_time) = process.start_time {
            #[cfg(not(target_os = "windows"))]
            {
                use std::time::SystemTime;
                let duration = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default();
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

        Ok(info.join("\n"))
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
