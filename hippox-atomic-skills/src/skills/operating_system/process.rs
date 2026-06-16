//! Process management utilities for listing, killing, and monitoring system processes.
//!
//! This module provides several skills for process management:
//! - `ProcessListSkill`: List running processes
//! - `ProcessKillSkill`: Terminate a process by PID
//! - `ProcessKillByNameSkill`: Terminate processes by name
//! - `ProcessIsRunningSkill`: Check if a process is running
//! - `ProcessGetPidSkill`: Get PID of a process by name
//! - `ProcessInfoSkill`: Get detailed information about a process

use crate::{SkillCategory, types::{Skill, SkillParameter}};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

/// A skill for listing running processes.
///
/// # Examples
/// ```
/// let result = process_list.execute(&HashMap::from([
///     ("filter".to_string(), json!("chrome")),
/// ])).await?;
/// ```
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
        SkillCategory::Os
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let filter = parameters.get("filter").and_then(|v| v.as_str());
        let top_n = parameters
            .get("top_n")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);
        let sort_by = parameters
            .get("sort_by")
            .and_then(|v| v.as_str())
            .unwrap_or("cpu");
        let mut system = System::new();
        system.refresh_all();
        let mut process_info: Vec<(String, u32, String, f32, u64)> = Vec::new();
        for (pid, process) in system.processes() {
            let name = process.name().to_string_lossy().to_string();
            if let Some(f) = filter {
                if !name.to_lowercase().contains(&f.to_lowercase()) {
                    continue;
                }
            }
            let cpu_usage = process.cpu_usage();
            let memory = process.memory();
            process_info.push((name, pid.as_u32(), pid.to_string(), cpu_usage, memory));
        }
        match sort_by {
            "memory" => process_info.sort_by(|a, b| b.4.cmp(&a.4)),
            "name" => process_info.sort_by(|a, b| a.0.cmp(&b.0)),
            "pid" => process_info.sort_by(|a, b| a.1.cmp(&b.1)),
            _ => process_info
                .sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal)),
        }
        if let Some(n) = top_n {
            process_info.truncate(n);
        }
        if process_info.is_empty() {
            return Ok("No matching processes found".to_string());
        }
        let mut output = vec![format!(
            "{:<8} {:<30} {:<10} {:<12}",
            "PID", "NAME", "CPU%", "MEMORY"
        )];
        output.push("-".repeat(62));
        for (name, pid, _, cpu, memory) in process_info {
            let mem_mb = memory as f64 / (1024.0 * 1024.0);
            output.push(format!(
                "{:<8} {:<30} {:<10.1} {:<12.1} MB",
                pid, name, cpu, mem_mb
            ));
        }
        Ok(output.join("\n"))
    }
}

/// A skill for terminating a process by PID.
///
/// # Examples
/// ```
/// let result = process_kill.execute(&HashMap::from([
///     ("pid".to_string(), json!(1234)),
/// ])).await?;
/// ```
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
        SkillCategory::Os
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pid_value = parameters
            .get("pid")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?;

        let pid = if let Some(p) = pid_value.as_u64() {
            p
        } else if let Some(p) = pid_value.as_i64() {
            p as u64
        } else {
            return Err(anyhow::anyhow!("PID must be a number"));
        };
        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mut system = System::new();
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        let pid_obj = Pid::from_u32(pid as u32);
        if !system.process(pid_obj).is_some() {
            return Err(anyhow::anyhow!("Process with PID {} not found", pid));
        }
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::{GetLastError, HANDLE};
            use windows_sys::Win32::System::Threading::PROCESS_TERMINATE;
            use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};
            let pid_u32 = pid as u32;
            unsafe {
                use windows_sys::Win32::Foundation::CloseHandle;
                let handle = OpenProcess(PROCESS_TERMINATE, 0, pid_u32);
                if handle == std::ptr::null_mut() {
                    return Err(anyhow::anyhow!(
                        "Failed to open process, error: {}",
                        GetLastError()
                    ));
                }
                let result = TerminateProcess(handle, 1);
                CloseHandle(handle);
                if result == 0 {
                    return Err(anyhow::anyhow!("Failed to terminate process"));
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            use nix::sys::signal::{Signal, kill};
            use nix::unistd::Pid as NixPid;
            let signal = if force {
                Signal::SIGKILL
            } else {
                Signal::SIGTERM
            };
            kill(NixPid::from_raw(pid as i32), signal)?;
        }
        Ok(format!("Process {} terminated successfully", pid))
    }
}

/// A skill for terminating processes by name.
///
/// # Examples
/// ```
/// let result = process_kill_by_name.execute(&HashMap::from([
///     ("name".to_string(), json!("notepad")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct ProcessKillByNameSkill;

#[async_trait::async_trait]
impl Skill for ProcessKillByNameSkill {
    fn name(&self) -> &str {
        "process_kill_by_name"
    }

    fn description(&self) -> &str {
        "Terminate all processes with a given name"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to stop all instances of an application"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to terminate (case-insensitive)".to_string(),
                required: true,
                default: None,
                example: Some(json!("chrome")),
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
            "action": "process_kill_by_name",
            "parameters": {
                "name": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Terminated 3 process(es) matching 'notepad.exe'".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Os
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mut system = System::new();
        system.refresh_all();
        let name_lower = name.to_lowercase();
        let mut killed = 0;
        for (pid, process) in system.processes() {
            let proc_name = process.name().to_string_lossy().to_lowercase();
            if proc_name.contains(&name_lower) {
                #[cfg(not(target_os = "windows"))]
                {
                    use nix::sys::signal::{Signal, kill};
                    use nix::unistd::Pid as NixPid;
                    let signal = if force {
                        Signal::SIGKILL
                    } else {
                        Signal::SIGTERM
                    };
                    let _ = kill(NixPid::from_raw(pid.as_u32() as i32), signal);
                    killed += 1;
                }
                #[cfg(target_os = "windows")]
                {
                    use windows_sys::Win32::System::Threading::PROCESS_TERMINATE;
                    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};
                    let pid_u32 = pid.as_u32();
                    unsafe {
                        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid_u32);
                        if handle != std::ptr::null_mut() {
                            use windows_sys::Win32::Foundation::CloseHandle;
                            let _ = TerminateProcess(handle, 1);
                            CloseHandle(handle);
                            killed += 1;
                        }
                    }
                }
            }
        }
        Ok(format!(
            "Terminated {} process(es) matching '{}'",
            killed, name
        ))
    }
}

/// A skill for checking if a process is running.
///
/// # Examples
/// ```
/// let result = process_is_running.execute(&HashMap::from([
///     ("name".to_string(), json!("sshd")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct ProcessIsRunningSkill;

#[async_trait::async_trait]
impl Skill for ProcessIsRunningSkill {
    fn name(&self) -> &str {
        "process_is_running"
    }

    fn description(&self) -> &str {
        "Check if a process with the given name is running"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to verify if a service or application is currently running"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to check".to_string(),
                required: true,
                default: None,
                example: Some(json!("nginx")),
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_is_running",
            "parameters": {
                "name": "docker"
            }
        })
    }

    fn example_output(&self) -> String {
        "Process 'docker' is running".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Os
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
        let mut system = System::new();
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        let name_lower = name.to_lowercase();
        let is_running = system.processes().values().any(|p| {
            let proc_name = p.name().to_string_lossy().to_lowercase();
            if exact_match {
                proc_name == name_lower
            } else {
                proc_name.contains(&name_lower)
            }
        });
        if is_running {
            Ok(format!("Process '{}' is running", name))
        } else {
            Ok(format!("Process '{}' is not running", name))
        }
    }
}

/// A skill for getting the PID of a process by name.
///
/// # Examples
/// ```
/// let result = process_get_pid.execute(&HashMap::from([
///     ("name".to_string(), json!("systemd")),
/// ])).await?;
/// ```
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
        SkillCategory::Os
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let first_only = parameters
            .get("first_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mut system = System::new();
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        let name_lower = name.to_lowercase();
        let pids: Vec<u32> = system
            .processes()
            .iter()
            .filter(|(_, p)| {
                p.name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&name_lower)
            })
            .map(|(pid, _)| pid.as_u32())
            .collect();
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

/// A skill for getting detailed information about a process.
///
/// # Examples
/// ```
/// let result = process_info.execute(&HashMap::from([
///     ("pid".to_string(), json!(1234)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct ProcessInfoSkill;

#[async_trait::async_trait]
impl Skill for ProcessInfoSkill {
    fn name(&self) -> &str {
        "process_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a process"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need detailed process metrics like CPU, memory, disk I/O"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "pid".to_string(),
            param_type: "integer".to_string(),
            description: "Process ID".to_string(),
            required: true,
            default: None,
            example: Some(json!(1234)),
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

    fn category(&self) -> SkillCategory {
        SkillCategory::Os
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pid_value = parameters
            .get("pid")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?;
        let pid = if let Some(p) = pid_value.as_u64() {
            p as u32
        } else if let Some(p) = pid_value.as_i64() {
            p as u32
        } else {
            return Err(anyhow::anyhow!("PID must be a number"));
        };
        let mut system = System::new();
        system.refresh_all();
        let pid_obj = Pid::from_u32(pid);
        if let Some(process) = system.process(pid_obj) {
            let mut info = Vec::new();
            info.push(format!("Process: {}", process.name().to_string_lossy()));
            info.push(format!("PID: {}", pid));
            info.push(format!(
                "Parent PID: {}",
                process.parent().map(|p| p.as_u32()).unwrap_or(0)
            ));
            info.push(format!("CPU Usage: {:.1}%", process.cpu_usage()));
            info.push(format!(
                "Memory: {:.2} MB",
                process.memory() as f64 / (1024.0 * 1024.0)
            ));
            info.push(format!(
                "Virtual Memory: {:.2} MB",
                process.virtual_memory() as f64 / (1024.0 * 1024.0)
            ));
            #[cfg(not(target_os = "windows"))]
            {
                use std::time::SystemTime;
                if let Some(start_time) = process.start_time() {
                    let duration = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default();
                    let uptime = duration.as_secs() - start_time;
                    info.push(format!("Uptime: {} seconds", uptime));
                }
            }
            Ok(info.join("\n"))
        } else {
            Err(anyhow::anyhow!("Process with PID {} not found", pid))
        }
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
    async fn test_process_is_running() {
        let skill = ProcessIsRunningSkill;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("operating_system"));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
    }
}
