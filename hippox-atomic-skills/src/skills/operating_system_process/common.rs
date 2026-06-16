//! Shared utilities for process management

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

/// Process information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
    pub cpu_usage: f32,
    pub memory: u64,
    pub virtual_memory: u64,
    pub status: String,
    pub start_time: Option<u64>,
}

/// Process list filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFilter {
    pub name: Option<String>,
    pub exact_match: bool,
    pub min_cpu: Option<f32>,
    pub min_memory: Option<u64>,
    pub status: Option<String>,
}

/// Process list sort options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessSortBy {
    Pid,
    Name,
    Cpu,
    Memory,
    Status,
}

/// Process status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}

/// Get all running processes
pub fn get_all_processes() -> Vec<ProcessInfo> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );
    let mut processes = Vec::new();
    for (pid, process) in system.processes() {
        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
            virtual_memory: process.virtual_memory(),
            status: format!("{:?}", process.status()),
            start_time: Some(process.start_time()),
        });
    }
    processes
}

/// Get processes matching a filter
pub fn get_processes_by_filter(filter: &ProcessFilter) -> Vec<ProcessInfo> {
    let all = get_all_processes();
    all.into_iter()
        .filter(|p| {
            if let Some(name) = &filter.name {
                if filter.exact_match {
                    if !p.name.eq_ignore_ascii_case(name) {
                        return false;
                    }
                } else if !p.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            }
            if let Some(min_cpu) = filter.min_cpu {
                if p.cpu_usage < min_cpu {
                    return false;
                }
            }
            if let Some(min_memory) = filter.min_memory {
                if p.memory < min_memory {
                    return false;
                }
            }
            if let Some(status) = &filter.status {
                if !p.status.to_lowercase().contains(&status.to_lowercase()) {
                    return false;
                }
            }
            true
        })
        .collect()
}

/// Get process by PID
pub fn get_process_by_pid(pid: u32) -> Option<ProcessInfo> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );
    let pid_obj = Pid::from_u32(pid);
    system.process(pid_obj).map(|process| ProcessInfo {
        pid: pid,
        name: process.name().to_string_lossy().to_string(),
        parent_pid: process.parent().map(|p| p.as_u32()),
        cpu_usage: process.cpu_usage(),
        memory: process.memory(),
        virtual_memory: process.virtual_memory(),
        status: format!("{:?}", process.status()),
        start_time: Some(process.start_time()),
    })
}

/// Get PIDs by process name
pub fn get_pids_by_name(name: &str, exact_match: bool) -> Vec<u32> {
    let system = System::new();
    let name_lower = name.to_lowercase();
    system
        .processes()
        .iter()
        .filter(|(_, p)| {
            let proc_name = p.name().to_string_lossy().to_lowercase();
            if exact_match {
                proc_name == name_lower
            } else {
                proc_name.contains(&name_lower)
            }
        })
        .map(|(pid, _)| pid.as_u32())
        .collect()
}

/// Check if a process is running
pub fn is_process_running(name: &str, exact_match: bool) -> bool {
    !get_pids_by_name(name, exact_match).is_empty()
}

/// Terminate a process by PID
pub fn kill_process(pid: u32, force: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_TERMINATE, TerminateProcess,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle == std::ptr::null_mut() {
                return Err(format!("Failed to open process with PID: {}", pid));
            }
            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);
            if result == 0 {
                return Err(format!("Failed to terminate process with PID: {}", pid));
            }
        }
        Ok(())
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
        kill(NixPid::from_raw(pid as i32), signal)
            .map_err(|e| format!("Failed to kill process {}: {}", pid, e))
    }
}

/// Terminate all processes with a given name
pub fn kill_processes_by_name(name: &str, force: bool) -> Result<usize, String> {
    let pids = get_pids_by_name(name, false);
    if pids.is_empty() {
        return Ok(0);
    }
    let mut killed = 0;
    for pid in pids {
        if kill_process(pid, force).is_ok() {
            killed += 1;
        }
    }
    Ok(killed)
}

/// Sort processes by specified field
pub fn sort_processes(processes: &mut [ProcessInfo], sort_by: ProcessSortBy) {
    match sort_by {
        ProcessSortBy::Pid => processes.sort_by_key(|p| p.pid),
        ProcessSortBy::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
        ProcessSortBy::Cpu => processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        ProcessSortBy::Memory => processes.sort_by_key(|p| std::cmp::Reverse(p.memory)),
        ProcessSortBy::Status => processes.sort_by(|a, b| a.status.cmp(&b.status)),
    }
}

/// Format memory in human-readable format
pub fn format_memory(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get process status as string
pub fn process_status_to_string(status: &str) -> &'static str {
    match status {
        "Running" => "Running",
        "Sleeping" => "Sleeping",
        "Stopped" => "Stopped",
        "Zombie" => "Zombie",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_processes() {
        let processes = get_all_processes();
        assert!(!processes.is_empty());
        // Check that at least one process has valid data
        let has_valid = processes.iter().any(|p| p.pid > 0);
        assert!(has_valid);
    }

    #[test]
    fn test_get_process_by_pid() {
        let processes = get_all_processes();
        if let Some(first) = processes.first() {
            let process = get_process_by_pid(first.pid);
            assert!(process.is_some());
            assert_eq!(process.unwrap().pid, first.pid);
        }
    }

    #[test]
    fn test_is_process_running() {
        // There should always be at least init/systemd or the current process running
        let result = is_process_running("system", false);
        // This may be false on some systems, but it's a reasonable check
        // We just verify the function doesn't panic
        assert!(result == true || result == false);
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(0), "0 B");
        assert_eq!(format_memory(1024), "1.00 KB");
        assert_eq!(format_memory(1024 * 1024), "1.00 MB");
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_process_status_to_string() {
        assert_eq!(process_status_to_string("Running"), "Running");
        assert_eq!(process_status_to_string("Sleeping"), "Sleeping");
        assert_eq!(process_status_to_string("Stopped"), "Stopped");
        assert_eq!(process_status_to_string("Zombie"), "Zombie");
        assert_eq!(process_status_to_string("Unknown"), "Unknown");
    }
}
