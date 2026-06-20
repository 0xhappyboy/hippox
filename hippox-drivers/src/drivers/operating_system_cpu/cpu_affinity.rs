//! CPU affinity driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::{Pid, ProcessRefreshKind, System};

/// Driver for getting/setting CPU affinity
#[derive(Debug)]
pub struct CpuAffinityDriver;

#[async_trait::async_trait]
impl Driver for CpuAffinityDriver {
    fn name(&self) -> &str {
        "cpu_affinity"
    }

    fn description(&self) -> &str {
        "Get or set CPU affinity for a process (bind to specific cores)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control which CPU cores a process runs on"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
                description: "Comma-separated list of core numbers to bind to (e.g., '0,2,4')"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0,2,4".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_affinity",
            "parameters": {
                "pid": 1234,
                "cores": "0,2,4"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"Process 1234 is bound to cores: 0, 2, 4"#.to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;
        let cores = parameters.get("cores").and_then(|v| v.as_str());
        if let Some(cores_str) = cores {
            // Set affinity
            set_affinity(pid, cores_str)?;
            Ok(format!("Process {} bound to cores: {}", pid, cores_str))
        } else {
            // Get affinity
            let affinity = get_affinity(pid)?;
            if affinity.is_empty() {
                Ok(format!("Process {} has no specific CPU affinity", pid))
            } else {
                Ok(format!(
                    "Process {} is bound to cores: {}",
                    pid,
                    affinity
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn get_affinity(pid: u32) -> Result<Vec<usize>> {
    let path = format!("/proc/{}/status", pid);
    let content = std::fs::read_to_string(&path)?;

    for line in content.lines() {
        if line.starts_with("Cpus_allowed_list:") {
            let affinity_str = line.split(':').nth(1).unwrap_or("").trim();
            if affinity_str.contains('-') {
                // Range format: "0-7"
                let parts: Vec<&str> = affinity_str.split('-').collect();
                if parts.len() == 2 {
                    let start = parts[0].parse::<usize>().unwrap_or(0);
                    let end = parts[1].parse::<usize>().unwrap_or(0);
                    return Ok((start..=end).collect());
                }
            } else {
                // Comma-separated list
                let cores: Result<Vec<usize>, _> = affinity_str
                    .split(',')
                    .map(|s| s.trim().parse::<usize>())
                    .collect();
                if let Ok(cores) = cores {
                    return Ok(cores);
                }
            }
            break;
        }
    }
    Ok(Vec::new())
}

#[cfg(target_os = "linux")]
fn set_affinity(pid: u32, cores_str: &str) -> Result<()> {
    use libc::{CPU_SET, CPU_ZERO, cpu_set_t, pid_t, sched_setaffinity};
    use std::mem;

    let cores: Result<Vec<usize>, _> = cores_str
        .split(',')
        .map(|s| s.trim().parse::<usize>())
        .collect();
    let cores = cores.map_err(|_| anyhow::anyhow!("Invalid core list"))?;

    unsafe {
        let mut cpuset: cpu_set_t = mem::zeroed();
        CPU_ZERO(&mut cpuset);

        for &core in &cores {
            CPU_SET(core, &mut cpuset);
        }

        let result = sched_setaffinity(pid as pid_t, mem::size_of::<cpu_set_t>(), &cpuset);
        if result != 0 {
            return Err(anyhow::anyhow!(
                "Failed to set affinity: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn get_affinity(_pid: u32) -> Result<Vec<usize>> {
    Ok(Vec::new())
}

#[cfg(not(target_os = "linux"))]
fn set_affinity(_pid: u32, _cores_str: &str) -> Result<()> {
    Err(anyhow::anyhow!("CPU affinity is only supported on Linux"))
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
