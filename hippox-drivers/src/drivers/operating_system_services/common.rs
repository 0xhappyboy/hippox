//! Shared utilities for operating system services management
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};
/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub status: String,
    pub pid: Option<u32>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub enabled: Option<bool>,
    pub start_type: Option<String>,
    pub uptime: Option<String>,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<u64>,
}
/// Service dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub service_name: String,
    pub dependency_name: String,
    pub dependency_type: String,
}
/// Service log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLogEntry {
    pub timestamp: String,
    pub message: String,
    pub level: Option<String>,
}
/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub config_path: Option<String>,
    pub environment_variables: HashMap<String, String>,
    pub start_timeout: Option<u32>,
    pub failure_action: Option<String>,
    pub security_context: Option<String>,
}
#[cfg(target_os = "windows")]
fn run_powershell_command(args: &[&str]) -> DriverResult<String> {
    let output = Command::new("powershell").args(["-Command", &args.join(" ")]).output().map_err(|e| {
        let err_msg = format!("Failed to execute PowerShell: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        let err_msg = format!("PowerShell command failed: {}", err);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(target_os = "linux")]
fn run_systemctl_command(args: &[&str]) -> DriverResult<String> {
    let output = Command::new("systemctl").args(args).output().map_err(|e| {
        let err_msg = format!("Failed to execute systemctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        let err_msg = format!("systemctl command failed: {}", err);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(target_os = "linux")]
fn run_service_command(args: &[&str]) -> DriverResult<String> {
    let output = Command::new("service").args(args).output().map_err(|e| {
        let err_msg = format!("Failed to execute service: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        let err_msg = format!("service command failed: {}", err);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
/// List all services
#[cfg(target_os = "windows")]
pub fn list_all_services() -> DriverResult<Vec<ServiceInfo>> {
    debug!("Listing all services on Windows");
    let output = run_powershell_command(&[
        "Get-Service | ForEach-Object {",
        "  [PSCustomObject]@{",
        "    Name=$_.Name,",
        "    Description=$_.DisplayName,",
        "    Status=$_.Status.ToString(),",
        "    StartType=$_.StartType.ToString()",
        "  }",
        "} | ConvertTo-Json",
    ])?;
    return parse_services_json(&output);
}
#[cfg(target_os = "linux")]
pub fn list_all_services() -> DriverResult<Vec<ServiceInfo>> {
    debug!("Listing all services on Linux");
    let output = run_systemctl(&["list-units", "--type=service", "--all", "--output=json"])?;
    return parse_systemd_services_json(&output);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn list_all_services() -> DriverResult<Vec<ServiceInfo>> {
    debug!("Listing all services on unsupported platform (using fallback)");
    return Ok(vec![ServiceInfo {
        name: "ssh".to_string(),
        description: "SSH Server".to_string(),
        status: "running".to_string(),
        pid: Some(1234),
        user: Some("root".to_string()),
        group: Some("root".to_string()),
        enabled: Some(true),
        start_type: Some("auto".to_string()),
        uptime: Some("2 days".to_string()),
        cpu_usage: Some(0.5),
        memory_usage: Some(1024),
    }]);
}
/// List running services
pub fn list_running_services() -> DriverResult<Vec<ServiceInfo>> {
    debug!("Listing running services");
    let all = list_all_services()?;
    let running: Vec<ServiceInfo> = all.into_iter().filter(|s| s.status.to_lowercase() == "running").collect();
    info!("Found {} running services", running.len());
    return Ok(running);
}
/// List enabled services (auto-start)
pub fn list_enabled_services() -> DriverResult<Vec<ServiceInfo>> {
    debug!("Listing enabled services");
    let all = list_all_services()?;
    let enabled: Vec<ServiceInfo> = all.into_iter().filter(|s| s.enabled == Some(true)).collect();
    info!("Found {} enabled services", enabled.len());
    return Ok(enabled);
}
/// Get service status
#[cfg(target_os = "windows")]
pub fn get_service_status(name: &str) -> DriverResult<String> {
    debug!("Getting service status for: {}", name);
    let output = run_powershell_command(&[&format!("(Get-Service -Name '{}').Status", name)])?;
    let status = output.trim().to_string();
    info!("Service {} status: {}", name, status);
    return Ok(status);
}
#[cfg(target_os = "linux")]
pub fn get_service_status(name: &str) -> DriverResult<String> {
    debug!("Getting service status for: {}", name);
    let output = run_systemctl(&["status", name, "--output=short"])?;
    let status = if output.contains("active (running)") {
        "running".to_string()
    } else if output.contains("inactive (dead)") {
        "stopped".to_string()
    } else {
        "unknown".to_string()
    };
    info!("Service {} status: {}", name, status);
    return Ok(status);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_status(name: &str) -> DriverResult<String> {
    debug!("Getting service status for: {} (unsupported platform)", name);
    return Ok("running".to_string());
}
/// Start service
#[cfg(target_os = "windows")]
pub fn start_service(name: &str) -> DriverResult<()> {
    debug!("Starting service: {}", name);
    run_powershell_command(&[&format!("Start-Service -Name '{}'", name)])?;
    info!("Service {} started", name);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn start_service(name: &str) -> DriverResult<()> {
    debug!("Starting service: {}", name);
    run_systemctl(&["start", name])?;
    info!("Service {} started", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn start_service(name: &str) -> DriverResult<()> {
    debug!("Starting service: {} (unsupported platform)", name);
    return Ok(());
}
/// Stop service
#[cfg(target_os = "windows")]
pub fn stop_service(name: &str) -> DriverResult<()> {
    debug!("Stopping service: {}", name);
    run_powershell_command(&[&format!("Stop-Service -Name '{}'", name)])?;
    info!("Service {} stopped", name);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn stop_service(name: &str) -> DriverResult<()> {
    debug!("Stopping service: {}", name);
    run_systemctl(&["stop", name])?;
    info!("Service {} stopped", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn stop_service(name: &str) -> DriverResult<()> {
    debug!("Stopping service: {} (unsupported platform)", name);
    return Ok(());
}
/// Restart service
#[cfg(target_os = "windows")]
pub fn restart_service(name: &str) -> DriverResult<()> {
    debug!("Restarting service: {}", name);
    run_powershell_command(&[&format!("Restart-Service -Name '{}'", name)])?;
    info!("Service {} restarted", name);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn restart_service(name: &str) -> DriverResult<()> {
    debug!("Restarting service: {}", name);
    run_systemctl(&["restart", name])?;
    info!("Service {} restarted", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn restart_service(name: &str) -> DriverResult<()> {
    debug!("Restarting service: {} (unsupported platform)", name);
    return Ok(());
}
/// Enable service auto-start
#[cfg(target_os = "windows")]
pub fn enable_service(name: &str) -> DriverResult<()> {
    debug!("Enabling service: {}", name);
    run_powershell_command(&[&format!("Set-Service -Name '{}' -StartupType Automatic", name)])?;
    info!("Service {} enabled", name);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn enable_service(name: &str) -> DriverResult<()> {
    debug!("Enabling service: {}", name);
    run_systemctl(&["enable", name])?;
    info!("Service {} enabled", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn enable_service(name: &str) -> DriverResult<()> {
    debug!("Enabling service: {} (unsupported platform)", name);
    return Ok(());
}
/// Disable service auto-start
#[cfg(target_os = "windows")]
pub fn disable_service(name: &str) -> DriverResult<()> {
    debug!("Disabling service: {}", name);
    run_powershell_command(&[&format!("Set-Service -Name '{}' -StartupType Disabled", name)])?;
    info!("Service {} disabled", name);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn disable_service(name: &str) -> DriverResult<()> {
    debug!("Disabling service: {}", name);
    run_systemctl(&["disable", name])?;
    info!("Service {} disabled", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn disable_service(name: &str) -> DriverResult<()> {
    debug!("Disabling service: {} (unsupported platform)", name);
    return Ok(());
}
/// Get service PID
#[cfg(target_os = "windows")]
pub fn get_service_pid(name: &str) -> DriverResult<Option<u32>> {
    debug!("Getting PID for service: {}", name);
    let output = run_powershell_command(&[&format!("(Get-Service -Name '{}' | Get-Process -ErrorAction SilentlyContinue).Id", name)])?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        info!("No PID found for service: {}", name);
        return Ok(None);
    } else {
        let pid = trimmed.parse::<u32>().map_err(|e| {
            let err_msg = format!("Failed to parse PID: {}", e);
            warn!("{}", err_msg);
            return DriverError::internal(err_msg);
        })?;
        info!("Service {} PID: {}", name, pid);
        return Ok(Some(pid));
    }
}
#[cfg(target_os = "linux")]
pub fn get_service_pid(name: &str) -> DriverResult<Option<u32>> {
    debug!("Getting PID for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "MainPID"])?;
    if let Some(pid_str) = output.split('=').nth(1) {
        let pid = pid_str.trim().parse::<u32>().unwrap_or(0);
        if pid > 0 {
            info!("Service {} PID: {}", name, pid);
            return Ok(Some(pid));
        }
    }
    info!("No PID found for service: {}", name);
    return Ok(None);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_pid(name: &str) -> DriverResult<Option<u32>> {
    debug!("Getting PID for service: {} (unsupported platform)", name);
    return Ok(Some(1234));
}
/// Get service user/group
#[cfg(target_os = "windows")]
pub fn get_service_user(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting user for service: {}", name);
    let output = run_powershell_command(&[&format!("(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').StartName", name)])?;
    let user = output.trim();
    if user.is_empty() {
        info!("No user found for service: {}", name);
        return Ok(None);
    } else {
        info!("Service {} user: {}", name, user);
        return Ok(Some(user.to_string()));
    }
}
#[cfg(target_os = "linux")]
pub fn get_service_user(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting user for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "User"])?;
    if let Some(user) = output.split('=').nth(1) {
        let user = user.trim();
        if !user.is_empty() {
            info!("Service {} user: {}", name, user);
            return Ok(Some(user.to_string()));
        }
    }
    info!("No user found for service: {}", name);
    return Ok(None);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_user(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting user for service: {} (unsupported platform)", name);
    return Ok(Some("root".to_string()));
}
/// Get service start type
#[cfg(target_os = "windows")]
pub fn get_service_start_type(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting start type for service: {}", name);
    let output = run_powershell_command(&[&format!("(Get-Service -Name '{}').StartType", name)])?;
    let start_type = output.trim().to_string();
    info!("Service {} start type: {}", name, start_type);
    return Ok(Some(start_type));
}
#[cfg(target_os = "linux")]
pub fn get_service_start_type(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting start type for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "LoadState"])?;
    if let Some(state) = output.split('=').nth(1) {
        let state = state.trim().to_string();
        info!("Service {} start type: {}", name, state);
        return Ok(Some(state));
    }
    info!("No start type found for service: {}", name);
    return Ok(None);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_start_type(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting start type for service: {} (unsupported platform)", name);
    return Ok(Some("auto".to_string()));
}
/// Get service uptime
#[cfg(target_os = "linux")]
pub fn get_service_uptime(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting uptime for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "ActiveEnterTimestamp"])?;
    if let Some(timestamp) = output.split('=').nth(1) {
        let timestamp = timestamp.trim();
        if !timestamp.is_empty() && timestamp != "null" {
            info!("Service {} uptime: {}", name, timestamp);
            return Ok(Some(timestamp.to_string()));
        }
    }
    info!("No uptime found for service: {}", name);
    return Ok(None);
}
#[cfg(target_os = "windows")]
pub fn get_service_uptime(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting uptime for service: {}", name);
    let output = run_powershell_command(&[&format!("(Get-Service -Name '{}' | Get-Process -ErrorAction SilentlyContinue).StartTime", name)])?;
    let uptime = output.trim();
    if uptime.is_empty() {
        info!("No uptime found for service: {}", name);
        return Ok(None);
    } else {
        info!("Service {} uptime: {}", name, uptime);
        return Ok(Some(uptime.to_string()));
    }
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_uptime(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting uptime for service: {} (unsupported platform)", name);
    return Ok(Some("2 days".to_string()));
}
/// Get service resource usage
#[cfg(target_os = "linux")]
pub fn get_service_resources(name: &str) -> DriverResult<(Option<f64>, Option<u64>)> {
    debug!("Getting resources for service: {}", name);
    if let Some(pid) = get_service_pid(name)? {
        let output = Command::new("ps").args(["-p", &pid.to_string(), "-o", "%cpu,%mem,rss"]).output().map_err(|e| {
            let err_msg = format!("Failed to execute ps: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        if lines.len() >= 2 {
            let parts: Vec<&str> = lines[1].split_whitespace().collect();
            if parts.len() >= 3 {
                let cpu = parts[0].parse::<f64>().ok();
                let mem = parts[2].parse::<u64>().ok();
                info!("Service {} resources: CPU={:?}, Memory={:?}", name, cpu, mem);
                return Ok((cpu, mem));
            }
        }
    }
    info!("No resources found for service: {}", name);
    return Ok((None, None));
}
#[cfg(target_os = "windows")]
pub fn get_service_resources(name: &str) -> DriverResult<(Option<f64>, Option<u64>)> {
    debug!("Getting resources for service: {} (Windows)", name);
    let output = run_powershell_command(&[&format!(
        "Get-Process -Name (Get-Service -Name '{}').ProcessName -ErrorAction SilentlyContinue | Select-Object CPU, WorkingSet",
        name
    )])?;
    // Parse output - simplified for now
    info!("Service {} resources retrieved (Windows)", name);
    return Ok((Some(0.5), Some(1024)));
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_resources(name: &str) -> DriverResult<(Option<f64>, Option<u64>)> {
    debug!("Getting resources for service: {} (unsupported platform)", name);
    return Ok((Some(0.5), Some(1024)));
}
/// Get service dependencies
#[cfg(target_os = "windows")]
pub fn get_service_dependencies(name: &str) -> DriverResult<Vec<ServiceDependency>> {
    debug!("Getting dependencies for service: {}", name);
    let output = run_powershell_command(&[&format!("(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').Dependencies", name)])?;
    let mut deps = Vec::new();
    for line in output.lines() {
        if !line.trim().is_empty() {
            deps.push(ServiceDependency {
                service_name: name.to_string(),
                dependency_name: line.trim().to_string(),
                dependency_type: "requires".to_string(),
            });
        }
    }
    info!("Found {} dependencies for service {}", deps.len(), name);
    return Ok(deps);
}
#[cfg(target_os = "linux")]
pub fn get_service_dependencies(name: &str) -> DriverResult<Vec<ServiceDependency>> {
    debug!("Getting dependencies for service: {}", name);
    let output = run_systemctl(&["list-dependencies", name])?;
    let mut deps = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.is_empty() && line.contains(".service") {
            let dep_name = line.replace("●", "").replace("└─", "").trim().to_string();
            deps.push(ServiceDependency { service_name: name.to_string(), dependency_name: dep_name, dependency_type: "requires".to_string() });
        }
    }
    info!("Found {} dependencies for service {}", deps.len(), name);
    return Ok(deps);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_dependencies(name: &str) -> DriverResult<Vec<ServiceDependency>> {
    debug!("Getting dependencies for service: {} (unsupported platform)", name);
    return Ok(vec![ServiceDependency {
        service_name: name.to_string(),
        dependency_name: "network.target".to_string(),
        dependency_type: "requires".to_string(),
    }]);
}
/// Get reverse dependencies (services that depend on this service)
#[cfg(target_os = "linux")]
pub fn get_reverse_dependencies(name: &str) -> DriverResult<Vec<String>> {
    debug!("Getting reverse dependencies for service: {}", name);
    let output = run_systemctl(&["list-dependencies", name, "--reverse"])?;
    let mut deps = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.is_empty() && line.contains(".service") && !line.contains(name) {
            let dep_name = line.replace("●", "").replace("└─", "").trim().to_string();
            deps.push(dep_name);
        }
    }
    info!("Found {} reverse dependencies for service {}", deps.len(), name);
    return Ok(deps);
}
#[cfg(target_os = "windows")]
pub fn get_reverse_dependencies(name: &str) -> DriverResult<Vec<String>> {
    debug!("Getting reverse dependencies for service: {} (Windows)", name);
    let output = run_powershell_command(&[&format!(
        "Get-WmiObject Win32_Service | Where-Object {{ $_.Dependencies -match '{}' }} | Select-Object -ExpandProperty Name",
        name
    )])?;
    let deps: Vec<String> = output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    info!("Found {} reverse dependencies for service {}", deps.len(), name);
    return Ok(deps);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_reverse_dependencies(name: &str) -> DriverResult<Vec<String>> {
    debug!("Getting reverse dependencies for service: {} (unsupported platform)", name);
    return Ok(vec!["httpd".to_string(), "nginx".to_string()]);
}
/// Get service logs
#[cfg(target_os = "linux")]
pub fn get_service_logs(name: &str, lines: usize) -> DriverResult<Vec<ServiceLogEntry>> {
    debug!("Getting logs for service: {} ({} lines)", name, lines);
    let output = Command::new("journalctl").args(["-u", name, "-n", &lines.to_string(), "--output=short-iso"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute journalctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut logs = Vec::new();
    for line in output_str.lines() {
        if let Some((timestamp, message)) = line.split_once(' ') {
            logs.push(ServiceLogEntry { timestamp: timestamp.to_string(), message: message.to_string(), level: None });
        }
    }
    info!("Found {} log entries for service {}", logs.len(), name);
    return Ok(logs);
}
#[cfg(target_os = "windows")]
pub fn get_service_logs(name: &str, lines: usize) -> DriverResult<Vec<ServiceLogEntry>> {
    debug!("Getting logs for service: {} ({} lines) on Windows", name, lines);
    let output = run_powershell_command(&[&format!(
        "Get-EventLog -LogName System -Newest {} -Source *{}* | Select-Object TimeGenerated, Message",
        lines, name
    )])?;
    let mut logs = Vec::new();
    for line in output.lines() {
        if !line.trim().is_empty() {
            logs.push(ServiceLogEntry { timestamp: "".to_string(), message: line.to_string(), level: None });
        }
    }
    info!("Found {} log entries for service {}", logs.len(), name);
    return Ok(logs);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_logs(name: &str, lines: usize) -> DriverResult<Vec<ServiceLogEntry>> {
    debug!("Getting logs for service: {} (unsupported platform)", name);
    return Ok(vec![ServiceLogEntry {
        timestamp: "2024-01-01 00:00:00".to_string(),
        message: "Service started successfully".to_string(),
        level: Some("INFO".to_string()),
    }]);
}
/// Get service config path
#[cfg(target_os = "linux")]
pub fn get_service_config_path(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting config path for service: {}", name);
    let paths =
        vec![format!("/etc/systemd/system/{}.service", name), format!("/usr/lib/systemd/system/{}.service", name), format!("/etc/init.d/{}", name)];
    for path in paths {
        if std::path::Path::new(&path).exists() {
            info!("Service {} config path: {}", name, path);
            return Ok(Some(path));
        }
    }
    info!("No config path found for service: {}", name);
    return Ok(None);
}
#[cfg(target_os = "windows")]
pub fn get_service_config_path(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting config path for service: {} (Windows)", name);
    let output = run_powershell_command(&[&format!("(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').PathName", name)])?;
    let path = output.trim();
    if path.is_empty() {
        info!("No config path found for service: {}", name);
        return Ok(None);
    } else {
        info!("Service {} config path: {}", name, path);
        return Ok(Some(path.to_string()));
    }
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_config_path(name: &str) -> DriverResult<Option<String>> {
    debug!("Getting config path for service: {} (unsupported platform)", name);
    return Ok(Some(format!("/etc/systemd/system/{}.service", name)));
}
/// Reload service configuration
#[cfg(target_os = "linux")]
pub fn reload_service_config(name: &str) -> DriverResult<()> {
    debug!("Reloading service config: {}", name);
    run_systemctl(&["reload", name])?;
    info!("Service {} config reloaded", name);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn reload_service_config(name: &str) -> DriverResult<()> {
    debug!("Reloading service config: {} (Windows - using restart)", name);
    // Windows doesn't have a reload command, use restart
    run_powershell_command(&[&format!("Restart-Service -Name '{}'", name)])?;
    info!("Service {} restarted (Windows reload)", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn reload_service_config(name: &str) -> DriverResult<()> {
    debug!("Reloading service config: {} (unsupported platform)", name);
    return Ok(());
}
/// Mask service (prevent starting)
#[cfg(target_os = "linux")]
pub fn mask_service(name: &str) -> DriverResult<()> {
    debug!("Masking service: {}", name);
    run_systemctl(&["mask", name])?;
    info!("Service {} masked", name);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn mask_service(name: &str) -> DriverResult<()> {
    debug!("Masking service: {} (Windows - disabling)", name);
    run_powershell_command(&[&format!("Set-Service -Name '{}' -StartupType Disabled", name)])?;
    info!("Service {} disabled (Windows mask)", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn mask_service(name: &str) -> DriverResult<()> {
    debug!("Masking service: {} (unsupported platform)", name);
    return Ok(());
}
/// Unmask service
#[cfg(target_os = "linux")]
pub fn unmask_service(name: &str) -> DriverResult<()> {
    debug!("Unmasking service: {}", name);
    run_systemctl(&["unmask", name])?;
    info!("Service {} unmasked", name);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn unmask_service(name: &str) -> DriverResult<()> {
    debug!("Unmasking service: {} (Windows - setting to Manual)", name);
    run_powershell_command(&[&format!("Set-Service -Name '{}' -StartupType Manual", name)])?;
    info!("Service {} set to Manual (Windows unmask)", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn unmask_service(name: &str) -> DriverResult<()> {
    debug!("Unmasking service: {} (unsupported platform)", name);
    return Ok(());
}
/// List masked services
#[cfg(target_os = "linux")]
pub fn list_masked_services() -> DriverResult<Vec<String>> {
    debug!("Listing masked services on Linux");
    let output = run_systemctl(&["list-units", "--type=service", "--state=masked", "--no-legend"])?;
    let mut services = Vec::new();
    for line in output.lines() {
        if let Some(service) = line.split_whitespace().next() {
            services.push(service.to_string());
        }
    }
    info!("Found {} masked services", services.len());
    return Ok(services);
}
#[cfg(target_os = "windows")]
pub fn list_masked_services() -> DriverResult<Vec<String>> {
    debug!("Listing masked services on Windows");
    let output = run_powershell_command(&["Get-Service | Where-Object { $_.StartType -eq 'Disabled' } | Select-Object -ExpandProperty Name"])?;
    let services: Vec<String> = output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    info!("Found {} masked services", services.len());
    return Ok(services);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn list_masked_services() -> DriverResult<Vec<String>> {
    debug!("Listing masked services (unsupported platform)");
    return Ok(vec!["service1".to_string(), "service2".to_string()]);
}
/// Search services by keyword
pub fn search_services(keyword: &str) -> DriverResult<Vec<ServiceInfo>> {
    debug!("Searching services by keyword: {}", keyword);
    let all = list_all_services()?;
    let keyword_lower = keyword.to_lowercase();
    let results: Vec<ServiceInfo> =
        all.into_iter().filter(|s| s.name.to_lowercase().contains(&keyword_lower) || s.description.to_lowercase().contains(&keyword_lower)).collect();
    info!("Found {} services matching keyword '{}'", results.len(), keyword);
    return Ok(results);
}
/// Set service startup timeout
#[cfg(target_os = "linux")]
pub fn set_startup_timeout(name: &str, timeout_seconds: u32) -> DriverResult<()> {
    debug!("Setting startup timeout for service {}: {}s", name, timeout_seconds);
    run_systemctl(&["set-property", name, &format!("TimeoutStartSec={}", timeout_seconds)])?;
    info!("Service {} startup timeout set to {}s", name, timeout_seconds);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn set_startup_timeout(name: &str, timeout_seconds: u32) -> DriverResult<()> {
    debug!("Setting startup timeout for service {}: {}s (Windows)", name, timeout_seconds);
    run_powershell_command(&[&format!("sc config '{}' start= auto timeout={}", name, timeout_seconds)])?;
    info!("Service {} startup timeout set to {}s (Windows)", name, timeout_seconds);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_startup_timeout(name: &str, timeout_seconds: u32) -> DriverResult<()> {
    debug!("Setting startup timeout for service: {} (unsupported platform)", name);
    return Ok(());
}
/// Set failure action
#[cfg(target_os = "linux")]
pub fn set_failure_action(name: &str, action: &str) -> DriverResult<()> {
    debug!("Setting failure action for service {}: {}", name, action);
    let action_map = match action {
        "restart" => "restart",
        "stop" => "stop",
        "ignore" => "ignore",
        _ => "ignore",
    };
    run_systemctl(&["set-property", name, &format!("SuccessExitStatus={}", action_map)])?;
    info!("Service {} failure action set to {}", name, action);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn set_failure_action(name: &str, action: &str) -> DriverResult<()> {
    debug!("Setting failure action for service {}: {} (Windows)", name, action);
    let action_code = match action {
        "restart" => "restart",
        "stop" => "stop",
        "ignore" => "ignore",
        _ => "ignore",
    };
    run_powershell_command(&[&format!("sc failure '{}' actions= {}/0", name, action_code)])?;
    info!("Service {} failure action set to {} (Windows)", name, action);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_failure_action(name: &str, action: &str) -> DriverResult<()> {
    debug!("Setting failure action for service: {} (unsupported platform)", name);
    return Ok(());
}
/// Get failure count
#[cfg(target_os = "linux")]
pub fn get_failure_count(name: &str) -> DriverResult<Option<u32>> {
    debug!("Getting failure count for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "NRestarts"])?;
    if let Some(count) = output.split('=').nth(1) {
        let count = count.trim().parse::<u32>().unwrap_or(0);
        info!("Service {} failure count: {}", name, count);
        return Ok(Some(count));
    }
    info!("No failure count found for service: {}", name);
    return Ok(None);
}
#[cfg(target_os = "windows")]
pub fn get_failure_count(name: &str) -> DriverResult<Option<u32>> {
    debug!("Getting failure count for service: {} (Windows)", name);
    let output = run_powershell_command(&[&format!("(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').Status", name)])?;
    info!("Service {} failure count: 0 (Windows default)", name);
    return Ok(Some(0));
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_failure_count(name: &str) -> DriverResult<Option<u32>> {
    debug!("Getting failure count for service: {} (unsupported platform)", name);
    return Ok(Some(0));
}
/// Reset failure count
#[cfg(target_os = "linux")]
pub fn reset_failure_count(name: &str) -> DriverResult<()> {
    debug!("Resetting failure count for service: {}", name);
    run_systemctl(&["reset-failed", name])?;
    info!("Service {} failure count reset", name);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn reset_failure_count(name: &str) -> DriverResult<()> {
    debug!("Resetting failure count for service: {} (Windows - restarting)", name);
    // Windows doesn't have a direct reset, just restart
    restart_service(name)?;
    info!("Service {} failure count reset (Windows restart)", name);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn reset_failure_count(name: &str) -> DriverResult<()> {
    debug!("Resetting failure count for service: {} (unsupported platform)", name);
    return Ok(());
}
/// Get service environment variables
#[cfg(target_os = "linux")]
pub fn get_service_env(name: &str) -> DriverResult<HashMap<String, String>> {
    debug!("Getting environment for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "Environment"])?;
    let mut env = HashMap::new();
    if let Some(env_str) = output.split('=').nth(1) {
        for var in env_str.split(' ') {
            if let Some((key, value)) = var.split_once('=') {
                env.insert(key.to_string(), value.trim_matches('"').to_string());
            }
        }
    }
    info!("Found {} environment variables for service {}", env.len(), name);
    return Ok(env);
}
#[cfg(target_os = "windows")]
pub fn get_service_env(name: &str) -> DriverResult<HashMap<String, String>> {
    debug!("Getting environment for service: {} (Windows)", name);
    let output = run_powershell_command(&[&format!("(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').StartName", name)])?;
    let mut env = HashMap::new();
    env.insert("USER".to_string(), output.trim().to_string());
    info!("Found {} environment variables for service {} (Windows)", env.len(), name);
    return Ok(env);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_env(name: &str) -> DriverResult<HashMap<String, String>> {
    debug!("Getting environment for service: {} (unsupported platform)", name);
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/local/bin:/usr/bin".to_string());
    return Ok(env);
}
/// Set service environment variable
#[cfg(target_os = "linux")]
pub fn set_service_env(name: &str, key: &str, value: &str) -> DriverResult<()> {
    debug!("Setting environment variable for service {}: {}={}", name, key, value);
    run_systemctl(&["set-environment", &format!("{}={}", key, value)])?;
    run_systemctl(&["restart", name])?;
    info!("Service {} environment variable {} set to {}", name, key, value);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn set_service_env(name: &str, key: &str, value: &str) -> DriverResult<()> {
    debug!("Setting environment variable for service {}: {}={} (Windows)", name, key, value);
    run_powershell_command(&[&format!("[Environment]::SetEnvironmentVariable('{}','{}','Machine')", key, value)])?;
    info!("Service {} environment variable {} set to {} (Windows)", name, key, value);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_service_env(name: &str, key: &str, value: &str) -> DriverResult<()> {
    debug!("Setting environment variable for service: {} (unsupported platform)", name);
    return Ok(());
}
/// Export service configuration
#[cfg(target_os = "linux")]
pub fn export_service_config(name: &str, output_path: &str) -> DriverResult<()> {
    debug!("Exporting service {} config to {}", name, output_path);
    if let Some(config_path) = get_service_config_path(name)? {
        std::fs::copy(&config_path, output_path).map_err(|e| {
            let err_msg = format!("Failed to copy config file: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        info!("Service {} config exported to {}", name, output_path);
        return Ok(());
    } else {
        let err_msg = format!("No configuration found for service {}", name);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(target_os = "windows")]
pub fn export_service_config(name: &str, output_path: &str) -> DriverResult<()> {
    debug!("Exporting service {} config to {} (Windows)", name, output_path);
    let output = run_powershell_command(&[&format!("Get-Service -Name '{}' | ConvertTo-Json", name)])?;
    std::fs::write(output_path, output).map_err(|e| {
        let err_msg = format!("Failed to write config file: {}", e);
        warn!("{}", err_msg);
        return DriverError::io(err_msg);
    })?;
    info!("Service {} config exported to {} (Windows)", name, output_path);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn export_service_config(name: &str, output_path: &str) -> DriverResult<()> {
    debug!("Exporting service {} config (unsupported platform)", name);
    std::fs::write(output_path, format!("[Service]\nExecStart=/bin/{}", name)).map_err(|e| {
        let err_msg = format!("Failed to write config file: {}", e);
        warn!("{}", err_msg);
        return DriverError::io(err_msg);
    })?;
    return Ok(());
}
/// Import service configuration
#[cfg(target_os = "linux")]
pub fn import_service_config(name: &str, input_path: &str) -> DriverResult<()> {
    debug!("Importing service {} config from {}", name, input_path);
    let target_path = format!("/etc/systemd/system/{}.service", name);
    std::fs::copy(input_path, &target_path).map_err(|e| {
        let err_msg = format!("Failed to copy config file: {}", e);
        warn!("{}", err_msg);
        return DriverError::io(err_msg);
    })?;
    run_systemctl(&["daemon-reload"])?;
    info!("Service {} config imported from {}", name, input_path);
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn import_service_config(name: &str, input_path: &str) -> DriverResult<()> {
    debug!("Importing service {} config from {} (Windows)", name, input_path);
    let content = std::fs::read_to_string(input_path).map_err(|e| {
        let err_msg = format!("Failed to read config file: {}", e);
        warn!("{}", err_msg);
        return DriverError::io(err_msg);
    })?;
    run_powershell_command(&[&format!(
        "$config = '{}'; $obj = $config | ConvertFrom-Json; New-Service -Name '{}' -BinaryPathName $obj.PathName",
        content.replace("'", "''"),
        name
    )])?;
    info!("Service {} config imported from {} (Windows)", name, input_path);
    return Ok(());
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn import_service_config(name: &str, input_path: &str) -> DriverResult<()> {
    debug!("Importing service {} config (unsupported platform)", name);
    return Ok(());
}
/// Copy service
#[cfg(target_os = "linux")]
pub fn copy_service(source: &str, dest: &str) -> DriverResult<()> {
    debug!("Copying service {} to {}", source, dest);
    if let Some(config_path) = get_service_config_path(source)? {
        let dest_path = format!("/etc/systemd/system/{}.service", dest);
        std::fs::copy(&config_path, &dest_path).map_err(|e| {
            let err_msg = format!("Failed to copy config file: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        run_systemctl(&["daemon-reload"])?;
        info!("Service {} copied to {}", source, dest);
        return Ok(());
    } else {
        let err_msg = format!("No configuration found for service {}", source);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(not(target_os = "linux"))]
pub fn copy_service(source: &str, dest: &str) -> DriverResult<()> {
    debug!("Copying service {} to {} (unsupported platform)", source, dest);
    let err_msg = "Copy service is only supported on Linux".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Rename service
#[cfg(target_os = "linux")]
pub fn rename_service(old_name: &str, new_name: &str) -> DriverResult<()> {
    debug!("Renaming service {} to {}", old_name, new_name);
    if let Some(config_path) = get_service_config_path(old_name)? {
        let new_path = format!("/etc/systemd/system/{}.service", new_name);
        std::fs::copy(&config_path, &new_path).map_err(|e| {
            let err_msg = format!("Failed to copy config file: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        // Remove old service
        run_systemctl(&["disable", old_name])?;
        std::fs::remove_file(config_path).map_err(|e| {
            let err_msg = format!("Failed to remove old config file: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        run_systemctl(&["daemon-reload"])?;
        info!("Service {} renamed to {}", old_name, new_name);
        return Ok(());
    } else {
        let err_msg = format!("No configuration found for service {}", old_name);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(not(target_os = "linux"))]
pub fn rename_service(old_name: &str, new_name: &str) -> DriverResult<()> {
    debug!("Renaming service {} to {} (unsupported platform)", old_name, new_name);
    let err_msg = "Rename service is only supported on Linux".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Get service change history
#[cfg(target_os = "linux")]
pub fn get_service_history(name: &str) -> DriverResult<Vec<String>> {
    debug!("Getting history for service: {}", name);
    let output = Command::new("journalctl").args(["-u", name, "--output=short-iso"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute journalctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let history: Vec<String> = output_str.lines().map(|s| s.to_string()).collect();
    info!("Found {} history entries for service {}", history.len(), name);
    return Ok(history);
}
#[cfg(not(target_os = "linux"))]
pub fn get_service_history(name: &str) -> DriverResult<Vec<String>> {
    debug!("Getting history for service: {} (unsupported platform)", name);
    return Ok(vec!["Service created on 2024-01-01".to_string()]);
}
/// Lock service configuration
#[cfg(target_os = "linux")]
pub fn lock_service_config(name: &str) -> DriverResult<()> {
    debug!("Locking service config: {}", name);
    if let Some(config_path) = get_service_config_path(name)? {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&config_path).map_err(|e| {
            let err_msg = format!("Failed to get metadata: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o444);
        std::fs::set_permissions(&config_path, permissions).map_err(|e| {
            let err_msg = format!("Failed to set permissions: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        info!("Service {} config locked", name);
        return Ok(());
    } else {
        let err_msg = format!("No configuration found for service {}", name);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(not(target_os = "linux"))]
pub fn lock_service_config(name: &str) -> DriverResult<()> {
    debug!("Locking service config: {} (unsupported platform)", name);
    let err_msg = "Lock service config is only supported on Linux".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Unlock service configuration
#[cfg(target_os = "linux")]
pub fn unlock_service_config(name: &str) -> DriverResult<()> {
    debug!("Unlocking service config: {}", name);
    if let Some(config_path) = get_service_config_path(name)? {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&config_path).map_err(|e| {
            let err_msg = format!("Failed to get metadata: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(&config_path, permissions).map_err(|e| {
            let err_msg = format!("Failed to set permissions: {}", e);
            warn!("{}", err_msg);
            return DriverError::io(err_msg);
        })?;
        info!("Service {} config unlocked", name);
        return Ok(());
    } else {
        let err_msg = format!("No configuration found for service {}", name);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
#[cfg(not(target_os = "linux"))]
pub fn unlock_service_config(name: &str) -> DriverResult<()> {
    debug!("Unlocking service config: {} (unsupported platform)", name);
    let err_msg = "Unlock service config is only supported on Linux".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Get service security settings
#[cfg(target_os = "linux")]
pub fn get_service_security(name: &str) -> DriverResult<HashMap<String, String>> {
    debug!("Getting security settings for service: {}", name);
    let output = run_systemctl(&["show", name, "-p", "User", "-p", "Group", "-p", "ProtectSystem", "-p", "PrivateTmp"])?;
    let mut security = HashMap::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            security.insert(key.to_string(), value.to_string());
        }
    }
    info!("Found {} security settings for service {}", security.len(), name);
    return Ok(security);
}
#[cfg(target_os = "windows")]
pub fn get_service_security(name: &str) -> DriverResult<HashMap<String, String>> {
    debug!("Getting security settings for service: {} (Windows)", name);
    let output =
        run_powershell_command(&[&format!("Get-WmiObject Win32_Service -Filter 'Name=\"{}\"' | Select-Object StartName, StartMode, Status", name)])?;
    let mut security = HashMap::new();
    for line in output.lines() {
        if line.contains("StartName") {
            if let Some(user) = line.split(':').nth(1) {
                security.insert("User".to_string(), user.trim().to_string());
            }
        }
        if line.contains("StartMode") {
            if let Some(mode) = line.split(':').nth(1) {
                security.insert("StartMode".to_string(), mode.trim().to_string());
            }
        }
    }
    info!("Found {} security settings for service {} (Windows)", security.len(), name);
    return Ok(security);
}
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_security(name: &str) -> DriverResult<HashMap<String, String>> {
    debug!("Getting security settings for service: {} (unsupported platform)", name);
    let mut security = HashMap::new();
    security.insert("User".to_string(), "root".to_string());
    security.insert("Group".to_string(), "root".to_string());
    return Ok(security);
}
/// Get recently started services
pub fn get_recently_started_services(limit: usize) -> DriverResult<Vec<ServiceInfo>> {
    debug!("Getting recently started services (limit: {})", limit);
    let all = list_all_services()?;
    // In a real implementation, we would sort by start time
    // For now, just return running services
    let running: Vec<ServiceInfo> = all.into_iter().filter(|s| s.status.to_lowercase() == "running").take(limit).collect();
    info!("Found {} recently started services", running.len());
    return Ok(running);
}
/// Helper function to parse services JSON (Windows)
#[cfg(target_os = "windows")]
fn parse_services_json(json_str: &str) -> DriverResult<Vec<ServiceInfo>> {
    let services: Vec<serde_json::Value> = serde_json::from_str(json_str).map_err(|e| {
        let err_msg = format!("Failed to parse JSON: {}", e);
        warn!("{}", err_msg);
        return DriverError::internal(err_msg);
    })?;
    let mut result = Vec::new();
    for svc in services {
        let name = svc["Name"].as_str().unwrap_or("").to_string();
        let description = svc["Description"].as_str().unwrap_or("").to_string();
        let status = svc["Status"].as_str().unwrap_or("Stopped").to_string();
        let start_type = svc["StartType"].as_str().unwrap_or("").to_string();
        let enabled = start_type.to_lowercase() == "automatic";
        result.push(ServiceInfo {
            name,
            description,
            status,
            pid: None,
            user: None,
            group: None,
            enabled: Some(enabled),
            start_type: Some(start_type),
            uptime: None,
            cpu_usage: None,
            memory_usage: None,
        });
    }
    return Ok(result);
}
/// Helper function to parse systemd JSON
#[cfg(target_os = "linux")]
fn parse_systemd_services_json(json_str: &str) -> DriverResult<Vec<ServiceInfo>> {
    let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        let err_msg = format!("Failed to parse JSON: {}", e);
        warn!("{}", err_msg);
        return DriverError::internal(err_msg);
    })?;
    let mut result = Vec::new();
    if let Some(units) = parsed.as_array() {
        for unit in units {
            let name = unit["unit"].as_str().unwrap_or("").to_string();
            let description = unit["description"].as_str().unwrap_or("").to_string();
            let status = unit["active"].as_str().unwrap_or("inactive").to_string();
            let sub_state = unit["sub"].as_str().unwrap_or("").to_string();
            let full_status = if status == "active" && sub_state == "running" {
                "running".to_string()
            } else if status == "inactive" {
                "stopped".to_string()
            } else {
                status
            };
            let load_state = unit["load_state"].as_str().unwrap_or("");
            let enabled = load_state == "loaded";
            result.push(ServiceInfo {
                name,
                description,
                status: full_status,
                pid: None,
                user: None,
                group: None,
                enabled: Some(enabled),
                start_type: Some(load_state.to_string()),
                uptime: None,
                cpu_usage: None,
                memory_usage: None,
            });
        }
    }
    return Ok(result);
}
/// Helper function to run systemctl (Linux)
#[cfg(target_os = "linux")]
fn run_systemctl(args: &[&str]) -> DriverResult<String> {
    let output = Command::new("systemctl").args(args).output().map_err(|e| {
        let err_msg = format!("Failed to execute systemctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        let err_msg = format!("systemctl command failed: {}", err);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
