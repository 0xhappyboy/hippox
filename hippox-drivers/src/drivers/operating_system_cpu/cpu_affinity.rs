//! CPU affinity driver module
//!
//! This module provides functionality to get or set CPU affinity for processes,
//! binding them to specific CPU cores.
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::{Pid, ProcessRefreshKind, System};
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for getting/setting CPU affinity
#[derive(Debug)]
pub struct CpuAffinityDriver;
#[async_trait::async_trait]
impl Driver for CpuAffinityDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "cpu_affinity";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get or set CPU affinity for a process (bind to specific cores)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to control which CPU cores a process runs on";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "cores".to_string(),
                param_type: "string".to_string(),
                description: "Comma-separated list of core numbers to bind to (e.g., '0,2,4')".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0,2,4".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "cpu_affinity",
            "parameters": {
                "pid": 1234,
                "cores": "0,2,4"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Process 1234 is bound to cores: 0, 2, 4"#.to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemCpu;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing cpu_affinity driver");
        let pid = parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))? as u32;
        let cores = parameters.get("cores").and_then(|v| v.as_str());
        if let Some(cores_str) = cores {
            debug!("Setting affinity for process {} to cores: {}", pid, cores_str);
            set_affinity(pid, cores_str)?;
            let result = format!("Process {} bound to cores: {}", pid, cores_str);
            info!("{}", result);
            return Ok(result);
        } else {
            debug!("Getting affinity for process {}", pid);
            let affinity = get_affinity(pid)?;
            let result = if affinity.is_empty() {
                format!("Process {} has no specific CPU affinity", pid)
            } else {
                format!("Process {} is bound to cores: {}", pid, affinity.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "))
            };
            info!("{}", result);
            return Ok(result);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))?;
        return Ok(());
    }
}
#[cfg(target_os = "linux")]
fn get_affinity(pid: u32) -> DriverResult<Vec<usize>> {
    let path = format!("/proc/{}/status", pid);
    let content = std::fs::read_to_string(&path).map_err(|e| DriverError::execution(format!("Failed to read /proc/{}/status: {}", pid, e)))?;
    for line in content.lines() {
        if line.starts_with("Cpus_allowed_list:") {
            let affinity_str = line.split(':').nth(1).unwrap_or("").trim();
            if affinity_str.contains('-') {
                let parts: Vec<&str> = affinity_str.split('-').collect();
                if parts.len() == 2 {
                    let start = parts[0].parse::<usize>().unwrap_or(0);
                    let end = parts[1].parse::<usize>().unwrap_or(0);
                    return Ok((start..=end).collect());
                }
            } else {
                let cores: Result<Vec<usize>, _> = affinity_str.split(',').map(|s| s.trim().parse::<usize>()).collect();
                if let Ok(cores) = cores {
                    return Ok(cores);
                }
            }
            break;
        }
    }
    return Ok(Vec::new());
}
#[cfg(target_os = "linux")]
fn set_affinity(pid: u32, cores_str: &str) -> DriverResult<()> {
    use libc::{CPU_SET, CPU_ZERO, cpu_set_t, pid_t, sched_setaffinity};
    use std::mem;
    let cores: Result<Vec<usize>, _> = cores_str.split(',').map(|s| s.trim().parse::<usize>()).collect();
    let cores = cores.map_err(|_| DriverError::execution("Invalid core list".to_string()))?;
    unsafe {
        let mut cpuset: cpu_set_t = mem::zeroed();
        CPU_ZERO(&mut cpuset);
        for &core in &cores {
            CPU_SET(core, &mut cpuset);
        }
        let result = sched_setaffinity(pid as pid_t, mem::size_of::<cpu_set_t>(), &cpuset);
        if result != 0 {
            return Err(DriverError::execution(format!("Failed to set affinity: {}", std::io::Error::last_os_error())));
        }
    }
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
fn get_affinity(_pid: u32) -> DriverResult<Vec<usize>> {
    return Ok(Vec::new());
}
#[cfg(not(target_os = "linux"))]
fn set_affinity(_pid: u32, _cores_str: &str) -> DriverResult<()> {
    return Err(DriverError::execution("CPU affinity is only supported on Linux".to_string()));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_affinity_metadata() {
        let driver = CpuAffinityDriver;
        assert_eq!(driver.name(), "cpu_affinity");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
