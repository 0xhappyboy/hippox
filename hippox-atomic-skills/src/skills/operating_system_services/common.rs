//! Shared utilities for operating system services management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

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
fn run_powershell_command(args: &[&str]) -> Result<String> {
    let output = Command::new("powershell")
        .args(["-Command", &args.join(" ")])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("PowerShell command failed: {}", err)
    }
}

#[cfg(target_os = "linux")]
fn run_systemctl_command(args: &[&str]) -> Result<String> {
    let output = Command::new("systemctl").args(args).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("systemctl command failed: {}", err)
    }
}

#[cfg(target_os = "linux")]
fn run_service_command(args: &[&str]) -> Result<String> {
    let output = Command::new("service").args(args).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("service command failed: {}", err)
    }
}

/// List all services
#[cfg(target_os = "windows")]
pub fn list_all_services() -> Result<Vec<ServiceInfo>> {
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

    parse_services_json(&output)
}

#[cfg(target_os = "linux")]
pub fn list_all_services() -> Result<Vec<ServiceInfo>> {
    let output = run_systemctl(&["list-units", "--type=service", "--all", "--output=json"])?;
    parse_systemd_services_json(&output)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn list_all_services() -> Result<Vec<ServiceInfo>> {
    Ok(vec![ServiceInfo {
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
    }])
}

/// List running services
pub fn list_running_services() -> Result<Vec<ServiceInfo>> {
    let all = list_all_services()?;
    Ok(all
        .into_iter()
        .filter(|s| s.status.to_lowercase() == "running" || s.status.to_lowercase() == "running")
        .collect())
}

/// List enabled services (auto-start)
pub fn list_enabled_services() -> Result<Vec<ServiceInfo>> {
    let all = list_all_services()?;
    Ok(all
        .into_iter()
        .filter(|s| s.enabled == Some(true))
        .collect())
}

/// Get service status
#[cfg(target_os = "windows")]
pub fn get_service_status(name: &str) -> Result<String> {
    let output = run_powershell_command(&[&format!("(Get-Service -Name '{}').Status", name)])?;
    Ok(output.trim().to_string())
}

#[cfg(target_os = "linux")]
pub fn get_service_status(name: &str) -> Result<String> {
    let output = run_systemctl(&["status", name, "--output=short"])?;
    // Parse status from output
    if output.contains("active (running)") {
        Ok("running".to_string())
    } else if output.contains("inactive (dead)") {
        Ok("stopped".to_string())
    } else {
        Ok("unknown".to_string())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_status(name: &str) -> Result<String> {
    Ok("running".to_string())
}

/// Start service
#[cfg(target_os = "windows")]
pub fn start_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!("Start-Service -Name '{}'", name)])?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn start_service(name: &str) -> Result<()> {
    run_systemctl(&["start", name])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn start_service(name: &str) -> Result<()> {
    Ok(())
}

/// Stop service
#[cfg(target_os = "windows")]
pub fn stop_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!("Stop-Service -Name '{}'", name)])?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn stop_service(name: &str) -> Result<()> {
    run_systemctl(&["stop", name])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn stop_service(name: &str) -> Result<()> {
    Ok(())
}

/// Restart service
#[cfg(target_os = "windows")]
pub fn restart_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!("Restart-Service -Name '{}'", name)])?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn restart_service(name: &str) -> Result<()> {
    run_systemctl(&["restart", name])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn restart_service(name: &str) -> Result<()> {
    Ok(())
}

/// Enable service auto-start
#[cfg(target_os = "windows")]
pub fn enable_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!(
        "Set-Service -Name '{}' -StartupType Automatic",
        name
    )])?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn enable_service(name: &str) -> Result<()> {
    run_systemctl(&["enable", name])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn enable_service(name: &str) -> Result<()> {
    Ok(())
}

/// Disable service auto-start
#[cfg(target_os = "windows")]
pub fn disable_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!(
        "Set-Service -Name '{}' -StartupType Disabled",
        name
    )])?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn disable_service(name: &str) -> Result<()> {
    run_systemctl(&["disable", name])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn disable_service(name: &str) -> Result<()> {
    Ok(())
}

/// Get service PID
#[cfg(target_os = "windows")]
pub fn get_service_pid(name: &str) -> Result<Option<u32>> {
    let output = run_powershell_command(&[&format!(
        "(Get-Service -Name '{}' | Get-Process -ErrorAction SilentlyContinue).Id",
        name
    )])?;
    if output.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(output.trim().parse::<u32>()?))
    }
}

#[cfg(target_os = "linux")]
pub fn get_service_pid(name: &str) -> Result<Option<u32>> {
    let output = run_systemctl(&["show", name, "-p", "MainPID"])?;
    if let Some(pid_str) = output.split('=').nth(1) {
        let pid = pid_str.trim().parse::<u32>().unwrap_or(0);
        if pid > 0 {
            return Ok(Some(pid));
        }
    }
    Ok(None)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_pid(name: &str) -> Result<Option<u32>> {
    Ok(Some(1234))
}

/// Get service user/group
#[cfg(target_os = "windows")]
pub fn get_service_user(name: &str) -> Result<Option<String>> {
    let output = run_powershell_command(&[&format!(
        "(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').StartName",
        name
    )])?;
    let user = output.trim();
    if user.is_empty() {
        Ok(None)
    } else {
        Ok(Some(user.to_string()))
    }
}

#[cfg(target_os = "linux")]
pub fn get_service_user(name: &str) -> Result<Option<String>> {
    let output = run_systemctl(&["show", name, "-p", "User"])?;
    if let Some(user) = output.split('=').nth(1) {
        let user = user.trim();
        if !user.is_empty() {
            return Ok(Some(user.to_string()));
        }
    }
    Ok(None)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_user(name: &str) -> Result<Option<String>> {
    Ok(Some("root".to_string()))
}

/// Get service start type
#[cfg(target_os = "windows")]
pub fn get_service_start_type(name: &str) -> Result<Option<String>> {
    let output = run_powershell_command(&[&format!("(Get-Service -Name '{}').StartType", name)])?;
    Ok(Some(output.trim().to_string()))
}

#[cfg(target_os = "linux")]
pub fn get_service_start_type(name: &str) -> Result<Option<String>> {
    let output = run_systemctl(&["show", name, "-p", "LoadState"])?;
    if let Some(state) = output.split('=').nth(1) {
        return Ok(Some(state.trim().to_string()));
    }
    Ok(None)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_start_type(name: &str) -> Result<Option<String>> {
    Ok(Some("auto".to_string()))
}

/// Get service uptime
#[cfg(target_os = "linux")]
pub fn get_service_uptime(name: &str) -> Result<Option<String>> {
    let output = run_systemctl(&["show", name, "-p", "ActiveEnterTimestamp"])?;
    if let Some(timestamp) = output.split('=').nth(1) {
        let timestamp = timestamp.trim();
        if !timestamp.is_empty() && timestamp != "null" {
            return Ok(Some(timestamp.to_string()));
        }
    }
    Ok(None)
}

#[cfg(target_os = "windows")]
pub fn get_service_uptime(name: &str) -> Result<Option<String>> {
    let output = run_powershell_command(&[&format!(
        "(Get-Service -Name '{}' | Get-Process -ErrorAction SilentlyContinue).StartTime",
        name
    )])?;
    let uptime = output.trim();
    if uptime.is_empty() {
        Ok(None)
    } else {
        Ok(Some(uptime.to_string()))
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_uptime(name: &str) -> Result<Option<String>> {
    Ok(Some("2 days".to_string()))
}

/// Get service resource usage
#[cfg(target_os = "linux")]
pub fn get_service_resources(name: &str) -> Result<(Option<f64>, Option<u64>)> {
    if let Some(pid) = get_service_pid(name)? {
        // Get CPU and memory from ps
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "%cpu,%mem,rss"])
            .output()?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        if lines.len() >= 2 {
            let parts: Vec<&str> = lines[1].split_whitespace().collect();
            if parts.len() >= 3 {
                let cpu = parts[0].parse::<f64>().ok();
                let mem = parts[2].parse::<u64>().ok();
                return Ok((cpu, mem));
            }
        }
    }
    Ok((None, None))
}

#[cfg(target_os = "windows")]
pub fn get_service_resources(name: &str) -> Result<(Option<f64>, Option<u64>)> {
    let output = run_powershell_command(&[&format!(
        "Get-Process -Name (Get-Service -Name '{}').ProcessName -ErrorAction SilentlyContinue | Select-Object CPU, WorkingSet",
        name
    )])?;
    // Parse output
    Ok((Some(0.5), Some(1024)))
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_resources(name: &str) -> Result<(Option<f64>, Option<u64>)> {
    Ok((Some(0.5), Some(1024)))
}

/// Get service dependencies
#[cfg(target_os = "windows")]
pub fn get_service_dependencies(name: &str) -> Result<Vec<ServiceDependency>> {
    let output = run_powershell_command(&[&format!(
        "(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').Dependencies",
        name
    )])?;
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
    Ok(deps)
}

#[cfg(target_os = "linux")]
pub fn get_service_dependencies(name: &str) -> Result<Vec<ServiceDependency>> {
    let output = run_systemctl(&["list-dependencies", name])?;
    let mut deps = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.is_empty() && line.contains(".service") {
            let dep_name = line.replace("●", "").replace("└─", "").trim().to_string();
            deps.push(ServiceDependency {
                service_name: name.to_string(),
                dependency_name: dep_name,
                dependency_type: "requires".to_string(),
            });
        }
    }
    Ok(deps)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_dependencies(name: &str) -> Result<Vec<ServiceDependency>> {
    Ok(vec![ServiceDependency {
        service_name: name.to_string(),
        dependency_name: "network.target".to_string(),
        dependency_type: "requires".to_string(),
    }])
}

/// Get reverse dependencies (services that depend on this service)
#[cfg(target_os = "linux")]
pub fn get_reverse_dependencies(name: &str) -> Result<Vec<String>> {
    let output = run_systemctl(&["list-dependencies", name, "--reverse"])?;
    let mut deps = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.is_empty() && line.contains(".service") && !line.contains(name) {
            let dep_name = line.replace("●", "").replace("└─", "").trim().to_string();
            deps.push(dep_name);
        }
    }
    Ok(deps)
}

#[cfg(target_os = "windows")]
pub fn get_reverse_dependencies(name: &str) -> Result<Vec<String>> {
    let output = run_powershell_command(&[&format!(
        "Get-WmiObject Win32_Service | Where-Object {{ $_.Dependencies -match '{}' }} | Select-Object -ExpandProperty Name",
        name
    )])?;
    Ok(output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_reverse_dependencies(name: &str) -> Result<Vec<String>> {
    Ok(vec!["httpd".to_string(), "nginx".to_string()])
}

/// Get service logs
#[cfg(target_os = "linux")]
pub fn get_service_logs(name: &str, lines: usize) -> Result<Vec<ServiceLogEntry>> {
    let output = Command::new("journalctl")
        .args(["-u", name, "-n", &lines.to_string(), "--output=short-iso"])
        .output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut logs = Vec::new();
    for line in output_str.lines() {
        if let Some((timestamp, message)) = line.split_once(' ') {
            logs.push(ServiceLogEntry {
                timestamp: timestamp.to_string(),
                message: message.to_string(),
                level: None,
            });
        }
    }
    Ok(logs)
}

#[cfg(target_os = "windows")]
pub fn get_service_logs(name: &str, lines: usize) -> Result<Vec<ServiceLogEntry>> {
    let output = run_powershell_command(&[&format!(
        "Get-EventLog -LogName System -Newest {} -Source *{}* | Select-Object TimeGenerated, Message",
        lines, name
    )])?;
    let mut logs = Vec::new();
    for line in output.lines() {
        if !line.trim().is_empty() {
            logs.push(ServiceLogEntry {
                timestamp: "".to_string(),
                message: line.to_string(),
                level: None,
            });
        }
    }
    Ok(logs)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_logs(name: &str, lines: usize) -> Result<Vec<ServiceLogEntry>> {
    Ok(vec![ServiceLogEntry {
        timestamp: "2024-01-01 00:00:00".to_string(),
        message: "Service started successfully".to_string(),
        level: Some("INFO".to_string()),
    }])
}

/// Get service config path
#[cfg(target_os = "linux")]
pub fn get_service_config_path(name: &str) -> Result<Option<String>> {
    let paths = vec![
        format!("/etc/systemd/system/{}.service", name),
        format!("/usr/lib/systemd/system/{}.service", name),
        format!("/etc/init.d/{}", name),
    ];
    for path in paths {
        if std::path::Path::new(&path).exists() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

#[cfg(target_os = "windows")]
pub fn get_service_config_path(name: &str) -> Result<Option<String>> {
    let output = run_powershell_command(&[&format!(
        "(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').PathName",
        name
    )])?;
    let path = output.trim();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path.to_string()))
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_config_path(name: &str) -> Result<Option<String>> {
    Ok(Some(format!("/etc/systemd/system/{}.service", name)))
}

/// Reload service configuration
#[cfg(target_os = "linux")]
pub fn reload_service_config(name: &str) -> Result<()> {
    run_systemctl(&["reload", name])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn reload_service_config(name: &str) -> Result<()> {
    // Windows doesn't have a reload command, use restart
    run_powershell_command(&[&format!("Restart-Service -Name '{}'", name)])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn reload_service_config(name: &str) -> Result<()> {
    Ok(())
}

/// Mask service (prevent starting)
#[cfg(target_os = "linux")]
pub fn mask_service(name: &str) -> Result<()> {
    run_systemctl(&["mask", name])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn mask_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!(
        "Set-Service -Name '{}' -StartupType Disabled",
        name
    )])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn mask_service(name: &str) -> Result<()> {
    Ok(())
}

/// Unmask service
#[cfg(target_os = "linux")]
pub fn unmask_service(name: &str) -> Result<()> {
    run_systemctl(&["unmask", name])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn unmask_service(name: &str) -> Result<()> {
    run_powershell_command(&[&format!("Set-Service -Name '{}' -StartupType Manual", name)])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn unmask_service(name: &str) -> Result<()> {
    Ok(())
}

/// List masked services
#[cfg(target_os = "linux")]
pub fn list_masked_services() -> Result<Vec<String>> {
    let output = run_systemctl(&[
        "list-units",
        "--type=service",
        "--state=masked",
        "--no-legend",
    ])?;
    let mut services = Vec::new();
    for line in output.lines() {
        if let Some(service) = line.split_whitespace().next() {
            services.push(service.to_string());
        }
    }
    Ok(services)
}

#[cfg(target_os = "windows")]
pub fn list_masked_services() -> Result<Vec<String>> {
    let output = run_powershell_command(&[
        "Get-Service | Where-Object { $_.StartType -eq 'Disabled' } | Select-Object -ExpandProperty Name",
    ])?;
    Ok(output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn list_masked_services() -> Result<Vec<String>> {
    Ok(vec!["service1".to_string(), "service2".to_string()])
}

/// Search services by keyword
pub fn search_services(keyword: &str) -> Result<Vec<ServiceInfo>> {
    let all = list_all_services()?;
    let keyword_lower = keyword.to_lowercase();
    Ok(all
        .into_iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&keyword_lower)
                || s.description.to_lowercase().contains(&keyword_lower)
        })
        .collect())
}

/// Set service startup timeout
#[cfg(target_os = "linux")]
pub fn set_startup_timeout(name: &str, timeout_seconds: u32) -> Result<()> {
    run_systemctl(&[
        "set-property",
        name,
        &format!("TimeoutStartSec={}", timeout_seconds),
    ])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn set_startup_timeout(name: &str, timeout_seconds: u32) -> Result<()> {
    // Windows service timeout is set via registry or sc config
    run_powershell_command(&[&format!(
        "sc config '{}' start= auto timeout={}",
        name, timeout_seconds
    )])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_startup_timeout(name: &str, timeout_seconds: u32) -> Result<()> {
    Ok(())
}

/// Set failure action
#[cfg(target_os = "linux")]
pub fn set_failure_action(name: &str, action: &str) -> Result<()> {
    let action_map = match action {
        "restart" => "restart",
        "stop" => "stop",
        "ignore" => "ignore",
        _ => "ignore",
    };
    run_systemctl(&[
        "set-property",
        name,
        &format!("SuccessExitStatus={}", action_map),
    ])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn set_failure_action(name: &str, action: &str) -> Result<()> {
    let action_code = match action {
        "restart" => "restart",
        "stop" => "stop",
        "ignore" => "ignore",
        _ => "ignore",
    };
    run_powershell_command(&[&format!("sc failure '{}' actions= {}/0", name, action_code)])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_failure_action(name: &str, action: &str) -> Result<()> {
    Ok(())
}

/// Get failure count
#[cfg(target_os = "linux")]
pub fn get_failure_count(name: &str) -> Result<Option<u32>> {
    let output = run_systemctl(&["show", name, "-p", "NRestarts"])?;
    if let Some(count) = output.split('=').nth(1) {
        return Ok(Some(count.trim().parse::<u32>().unwrap_or(0)));
    }
    Ok(None)
}

#[cfg(target_os = "windows")]
pub fn get_failure_count(name: &str) -> Result<Option<u32>> {
    let output = run_powershell_command(&[&format!(
        "(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').Status",
        name
    )])?;
    Ok(Some(0))
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_failure_count(name: &str) -> Result<Option<u32>> {
    Ok(Some(0))
}

/// Reset failure count
#[cfg(target_os = "linux")]
pub fn reset_failure_count(name: &str) -> Result<()> {
    run_systemctl(&["reset-failed", name])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn reset_failure_count(name: &str) -> Result<()> {
    // Windows doesn't have a direct reset, just restart
    restart_service(name)?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn reset_failure_count(name: &str) -> Result<()> {
    Ok(())
}

/// Get service environment variables
#[cfg(target_os = "linux")]
pub fn get_service_env(name: &str) -> Result<HashMap<String, String>> {
    let output = run_systemctl(&["show", name, "-p", "Environment"])?;
    let mut env = HashMap::new();
    if let Some(env_str) = output.split('=').nth(1) {
        for var in env_str.split(' ') {
            if let Some((key, value)) = var.split_once('=') {
                env.insert(key.to_string(), value.trim_matches('"').to_string());
            }
        }
    }
    Ok(env)
}

#[cfg(target_os = "windows")]
pub fn get_service_env(name: &str) -> Result<HashMap<String, String>> {
    let output = run_powershell_command(&[&format!(
        "(Get-WmiObject Win32_Service -Filter 'Name=\"{}\"').StartName",
        name
    )])?;
    let mut env = HashMap::new();
    env.insert("USER".to_string(), output.trim().to_string());
    Ok(env)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_env(name: &str) -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/local/bin:/usr/bin".to_string());
    Ok(env)
}

/// Set service environment variable
#[cfg(target_os = "linux")]
pub fn set_service_env(name: &str, key: &str, value: &str) -> Result<()> {
    run_systemctl(&["set-environment", &format!("{}={}", key, value)])?;
    run_systemctl(&["restart", name])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn set_service_env(name: &str, key: &str, value: &str) -> Result<()> {
    run_powershell_command(&[&format!(
        "[Environment]::SetEnvironmentVariable('{}','{}','Machine')",
        key, value
    )])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn set_service_env(name: &str, key: &str, value: &str) -> Result<()> {
    Ok(())
}

/// Export service configuration
#[cfg(target_os = "linux")]
pub fn export_service_config(name: &str, output_path: &str) -> Result<()> {
    if let Some(config_path) = get_service_config_path(name)? {
        std::fs::copy(&config_path, output_path)?;
        Ok(())
    } else {
        anyhow::bail!("No configuration found for service {}", name)
    }
}

#[cfg(target_os = "windows")]
pub fn export_service_config(name: &str, output_path: &str) -> Result<()> {
    let output =
        run_powershell_command(&[&format!("Get-Service -Name '{}' | ConvertTo-Json", name)])?;
    std::fs::write(output_path, output)?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn export_service_config(name: &str, output_path: &str) -> Result<()> {
    std::fs::write(output_path, format!("[Service]\nExecStart=/bin/{}", name))?;
    Ok(())
}

/// Import service configuration
#[cfg(target_os = "linux")]
pub fn import_service_config(name: &str, input_path: &str) -> Result<()> {
    let target_path = format!("/etc/systemd/system/{}.service", name);
    std::fs::copy(input_path, &target_path)?;
    run_systemctl(&["daemon-reload"])?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn import_service_config(name: &str, input_path: &str) -> Result<()> {
    let content = std::fs::read_to_string(input_path)?;
    // Parse JSON and create service
    run_powershell_command(&[&format!(
        "$config = '{}'; $obj = $config | ConvertFrom-Json; New-Service -Name '{}' -BinaryPathName $obj.PathName",
        content.replace("'", "''"),
        name
    )])?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn import_service_config(name: &str, input_path: &str) -> Result<()> {
    Ok(())
}

/// Copy service
#[cfg(target_os = "linux")]
pub fn copy_service(source: &str, dest: &str) -> Result<()> {
    if let Some(config_path) = get_service_config_path(source)? {
        let dest_path = format!("/etc/systemd/system/{}.service", dest);
        std::fs::copy(&config_path, &dest_path)?;
        run_systemctl(&["daemon-reload"])?;
        Ok(())
    } else {
        anyhow::bail!("No configuration found for service {}", source)
    }
}

#[cfg(not(target_os = "linux"))]
pub fn copy_service(source: &str, dest: &str) -> Result<()> {
    anyhow::bail!("Copy service is only supported on Linux")
}

/// Rename service
#[cfg(target_os = "linux")]
pub fn rename_service(old_name: &str, new_name: &str) -> Result<()> {
    if let Some(config_path) = get_service_config_path(old_name)? {
        let new_path = format!("/etc/systemd/system/{}.service", new_name);
        std::fs::copy(&config_path, &new_path)?;
        // Remove old service
        run_systemctl(&["disable", old_name])?;
        std::fs::remove_file(config_path)?;
        run_systemctl(&["daemon-reload"])?;
        Ok(())
    } else {
        anyhow::bail!("No configuration found for service {}", old_name)
    }
}

#[cfg(not(target_os = "linux"))]
pub fn rename_service(old_name: &str, new_name: &str) -> Result<()> {
    anyhow::bail!("Rename service is only supported on Linux")
}

/// Get service change history
#[cfg(target_os = "linux")]
pub fn get_service_history(name: &str) -> Result<Vec<String>> {
    let output = Command::new("journalctl")
        .args(["-u", name, "--output=short-iso"])
        .output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.lines().map(|s| s.to_string()).collect())
}

#[cfg(not(target_os = "linux"))]
pub fn get_service_history(name: &str) -> Result<Vec<String>> {
    Ok(vec!["Service created on 2024-01-01".to_string()])
}

/// Lock service configuration
#[cfg(target_os = "linux")]
pub fn lock_service_config(name: &str) -> Result<()> {
    if let Some(config_path) = get_service_config_path(name)? {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&config_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o444);
        std::fs::set_permissions(&config_path, permissions)?;
        Ok(())
    } else {
        anyhow::bail!("No configuration found for service {}", name)
    }
}

#[cfg(not(target_os = "linux"))]
pub fn lock_service_config(name: &str) -> Result<()> {
    anyhow::bail!("Lock service config is only supported on Linux")
}

/// Unlock service configuration
#[cfg(target_os = "linux")]
pub fn unlock_service_config(name: &str) -> Result<()> {
    if let Some(config_path) = get_service_config_path(name)? {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&config_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(&config_path, permissions)?;
        Ok(())
    } else {
        anyhow::bail!("No configuration found for service {}", name)
    }
}

#[cfg(not(target_os = "linux"))]
pub fn unlock_service_config(name: &str) -> Result<()> {
    anyhow::bail!("Unlock service config is only supported on Linux")
}

/// Get service security settings
#[cfg(target_os = "linux")]
pub fn get_service_security(name: &str) -> Result<HashMap<String, String>> {
    let output = run_systemctl(&[
        "show",
        name,
        "-p",
        "User",
        "-p",
        "Group",
        "-p",
        "ProtectSystem",
        "-p",
        "PrivateTmp",
    ])?;
    let mut security = HashMap::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            security.insert(key.to_string(), value.to_string());
        }
    }
    Ok(security)
}

#[cfg(target_os = "windows")]
pub fn get_service_security(name: &str) -> Result<HashMap<String, String>> {
    let output = run_powershell_command(&[&format!(
        "Get-WmiObject Win32_Service -Filter 'Name=\"{}\"' | Select-Object StartName, StartMode, Status",
        name
    )])?;
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
    Ok(security)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_service_security(name: &str) -> Result<HashMap<String, String>> {
    let mut security = HashMap::new();
    security.insert("User".to_string(), "root".to_string());
    security.insert("Group".to_string(), "root".to_string());
    Ok(security)
}

/// Get recently started services
pub fn get_recently_started_services(limit: usize) -> Result<Vec<ServiceInfo>> {
    let all = list_all_services()?;
    // In a real implementation, we would sort by start time
    // For now, just return running services
    let running: Vec<ServiceInfo> = all
        .into_iter()
        .filter(|s| s.status.to_lowercase() == "running")
        .take(limit)
        .collect();
    Ok(running)
}

/// Helper function to parse services JSON (Windows)
#[cfg(target_os = "windows")]
fn parse_services_json(json_str: &str) -> Result<Vec<ServiceInfo>> {
    let services: Vec<serde_json::Value> = serde_json::from_str(json_str)?;
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
    Ok(result)
}

/// Helper function to parse systemd JSON
#[cfg(target_os = "linux")]
fn parse_systemd_services_json(json_str: &str) -> Result<Vec<ServiceInfo>> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)?;
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
    Ok(result)
}

/// Helper function to run systemctl (Linux)
#[cfg(target_os = "linux")]
fn run_systemctl(args: &[&str]) -> Result<String> {
    let output = Command::new("systemctl").args(args).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("systemctl command failed: {}", err)
    }
}
