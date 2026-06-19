//! Operating system utilities for system control and management.
//!
//! This module provides comprehensive OS management skills:
//! - `OsRebootDriver`: Reboot the system
//! - `OsShutdownDriver`: Shutdown the system
//! - `OsSleepDriver`: Put the system to sleep/hibernate
//! - `OsLockDriver`: Lock the system screen
//! - `OsLogoutDriver`: Log out current user
//! - `OsHibernateDriver`: Hibernate the system
//! - `OsGetUptimeDriver`: Get system uptime
//! - `OsGetLoadAverageDriver`: Get system load average
//! - `OsGetHostnameDriver`: Get/set system hostname
//! - `OsGetTimeDriver`: Get system time and timezone
//! - `OsSetTimeDriver`: Set system time
//! - `OsGetUserDriver`: Get current user info
//! - `OsDiskUsageDriver`: Get disk usage information
//! - `OsMemoryInfoDriver`: Get memory information
//! - `OsCpuInfoDriver`: Get CPU information
//! - `OsNetworkInfoDriver`: Get network interface information
//! - `OsBatteryInfoDriver`: Get battery status (laptops)
//! - `OsNotificationDriver`: Send desktop notifications

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::{Disks, Networks, System, Users};

/// A skill for rebooting the system.
#[derive(Debug)]
pub struct OsRebootDriver;

#[async_trait::async_trait]
impl Driver for OsRebootDriver {
    fn name(&self) -> &str {
        "os_reboot"
    }

    fn description(&self) -> &str {
        "Reboot the system"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to restart the system for updates or troubleshooting"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "delay".to_string(),
                param_type: "integer".to_string(),
                description: "Delay in seconds before reboot (default: 0)".to_string(),
                required: false,
                default: Some(json!(0)),
                example: Some(json!(60)),
                enum_values: None,
            },
            DriverParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force reboot without asking (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_reboot",
            "parameters": {
                "delay": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "System will reboot in 10 seconds".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let delay = parameters
            .get("delay")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            let mut args: Vec<String> = vec!["/r".to_string()];
            if delay > 0 {
                args.push("/t".to_string());
                args.push(delay.to_string());
            }
            if force {
                args.push("/f".to_string());
            }
            let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            exec_async("shutdown", &args_ref, None).await?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let mut args = vec!["shutdown"];
            if delay > 0 {
                args.push("-h");
                args.push(&format!("+{}", delay / 60));
            } else {
                args.push("-r");
                args.push("now");
            }
            if force {
                args.push("-f");
            }
            let _ = exec_async("sudo", &args, None).await;
        }
        Ok(format!("System will reboot in {} seconds", delay))
    }
}

/// A skill for shutting down the system.
#[derive(Debug)]
pub struct OsShutdownDriver;

#[async_trait::async_trait]
impl Driver for OsShutdownDriver {
    fn name(&self) -> &str {
        "os_shutdown"
    }

    fn description(&self) -> &str {
        "Shutdown the system"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to power off the system"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "delay".to_string(),
                param_type: "integer".to_string(),
                description: "Delay in seconds before shutdown (default: 0)".to_string(),
                required: false,
                default: Some(json!(0)),
                example: Some(json!(120)),
                enum_values: None,
            },
            DriverParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force shutdown without asking (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_shutdown",
            "parameters": {
                "delay": 30
            }
        })
    }

    fn example_output(&self) -> String {
        "System will shutdown in 30 seconds".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let delay = parameters
            .get("delay")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            let mut args: Vec<String> = vec!["/r".to_string()];
            if delay > 0 {
                args.push("/t".to_string());
                args.push(delay.to_string());
            }
            if force {
                args.push("/f".to_string());
            }
            let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            exec_async("shutdown", &args_ref, None).await?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let mut args = vec!["shutdown"];
            if delay > 0 {
                args.push("-h");
                args.push(&format!("+{}", delay / 60));
            } else {
                args.push("-h");
                args.push("now");
            }
            if force {
                args.push("-f");
            }
            let _ = exec_async("sudo", &args, None).await;
        }
        Ok(format!("System will shutdown in {} seconds", delay))
    }
}

/// A skill for putting the system to sleep.
#[derive(Debug)]
pub struct OsSleepDriver;

#[async_trait::async_trait]
impl Driver for OsSleepDriver {
    fn name(&self) -> &str {
        "os_sleep"
    }

    fn description(&self) -> &str {
        "Put the system to sleep (suspend to RAM)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to save power by putting the system into low-power sleep mode"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_sleep"
        })
    }

    fn example_output(&self) -> String {
        "System is going to sleep".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async(
                "rundll32.exe",
                &["powrprof.dll,SetSuspendState", "0", "1", "0"],
                None,
            )
            .await?;
        }
        #[cfg(target_os = "macos")]
        {
            exec_async("pmset", &["sleepnow"], None).await?;
        }
        #[cfg(target_os = "linux")]
        {
            exec_async("systemctl", &["suspend"], None).await?;
        }
        Ok("System is going to sleep".to_string())
    }
}

/// A skill for locking the system screen.
#[derive(Debug)]
pub struct OsLockDriver;

#[async_trait::async_trait]
impl Driver for OsLockDriver {
    fn name(&self) -> &str {
        "os_lock"
    }

    fn description(&self) -> &str {
        "Lock the system screen"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to secure the system without logging out"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_lock"
        })
    }

    fn example_output(&self) -> String {
        "Screen locked".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async("rundll32.exe", &["user32.dll,LockWorkStation"], None).await?;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = exec_async("osascript", &["-e", "tell application \"System Events\" to keystroke \"q\" using {command down, control down}"], None).await;
        }
        #[cfg(target_os = "linux")]
        {
            let _ = exec_async("gnome-screensaver-command", &["-l"], None).await;
            let _ = exec_async("xdg-screensaver", &["lock"], None).await;
        }
        Ok("Screen locked".to_string())
    }
}

/// A skill for logging out the current user.
#[derive(Debug)]
pub struct OsLogoutDriver;

#[async_trait::async_trait]
impl Driver for OsLogoutDriver {
    fn name(&self) -> &str {
        "os_logout"
    }

    fn description(&self) -> &str {
        "Log out the current user"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to end the current user session"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "force".to_string(),
            param_type: "boolean".to_string(),
            description: "Force logout without confirmation (default: false)".to_string(),
            required: false,
            default: Some(json!(false)),
            example: Some(json!(true)),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_logout"
        })
    }

    fn example_output(&self) -> String {
        "Logging out current user".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let _force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async("shutdown", &["/l"], None).await?;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = exec_async(
                "osascript",
                &["-e", "tell application \"System Events\" to log out"],
                None,
            )
            .await;
        }
        #[cfg(target_os = "linux")]
        {
            let _ = exec_async("gnome-session-quit", &["--no-prompt"], None).await;
            let _ = exec_async("pkill", &["-KILL", "-u", "$USER"], None).await;
        }
        Ok("Logging out current user".to_string())
    }
}

/// A skill for hibernating the system.
#[derive(Debug)]
pub struct OsHibernateDriver;

#[async_trait::async_trait]
impl Driver for OsHibernateDriver {
    fn name(&self) -> &str {
        "os_hibernate"
    }

    fn description(&self) -> &str {
        "Hibernate the system (suspend to disk)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to save power while preserving system state to disk"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_hibernate"
        })
    }

    fn example_output(&self) -> String {
        "System is hibernating".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async("shutdown", &["/h"], None).await?;
        }
        #[cfg(target_os = "linux")]
        {
            exec_async("systemctl", &["hibernate"], None).await?;
        }
        #[cfg(target_os = "macos")]
        {
            exec_async("pmset", &["sleepnow"], None).await?;
        }
        Ok("System is hibernating".to_string())
    }
}

/// A skill for getting system uptime.
#[derive(Debug)]
pub struct OsGetUptimeDriver;

#[async_trait::async_trait]
impl Driver for OsGetUptimeDriver {
    fn name(&self) -> &str {
        "os_get_uptime"
    }

    fn description(&self) -> &str {
        "Get system uptime information"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check how long the system has been running"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "human_readable".to_string(),
            param_type: "boolean".to_string(),
            description: "Return human-readable format (default: true)".to_string(),
            required: false,
            default: Some(json!(true)),
            example: Some(json!(false)),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_uptime"
        })
    }

    fn example_output(&self) -> String {
        "System uptime: 5 days, 3 hours, 22 minutes".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let human_readable = parameters
            .get("human_readable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mut system = System::new();
        system.refresh_all();
        let uptime_secs = System::uptime();
        if human_readable {
            let days = uptime_secs / 86400;
            let hours = (uptime_secs % 86400) / 3600;
            let minutes = (uptime_secs % 3600) / 60;
            let seconds = uptime_secs % 60;
            let mut parts = Vec::new();
            if days > 0 {
                parts.push(format!("{} days", days));
            }
            if hours > 0 {
                parts.push(format!("{} hours", hours));
            }
            if minutes > 0 {
                parts.push(format!("{} minutes", minutes));
            }
            if seconds > 0 && days == 0 && hours == 0 {
                parts.push(format!("{} seconds", seconds));
            }
            Ok(format!("System uptime: {}", parts.join(", ")))
        } else {
            Ok(format!("System uptime: {} seconds", uptime_secs))
        }
    }
}

/// A skill for getting system load average.
#[derive(Debug)]
pub struct OsGetLoadAverageDriver;

#[async_trait::async_trait]
impl Driver for OsGetLoadAverageDriver {
    fn name(&self) -> &str {
        "os_get_load_average"
    }

    fn description(&self) -> &str {
        "Get system load average (1, 5, 15 minutes)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check system load and performance"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_load_average"
        })
    }

    fn example_output(&self) -> String {
        "Load average: 1 min: 2.5, 5 min: 1.8, 15 min: 1.2".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let load_avg = System::load_average();
        Ok(format!(
            "Load average: 1 min: {:.2}, 5 min: {:.2}, 15 min: {:.2}",
            load_avg.one, load_avg.five, load_avg.fifteen
        ))
    }
}

/// A skill for getting system hostname.
#[derive(Debug)]
pub struct OsGetHostnameDriver;

#[async_trait::async_trait]
impl Driver for OsGetHostnameDriver {
    fn name(&self) -> &str {
        "os_get_hostname"
    }

    fn description(&self) -> &str {
        "Get or set the system hostname"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the current hostname or set a new one (requires admin privileges)"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "set_hostname".to_string(),
            param_type: "string".to_string(),
            description: "New hostname to set (requires admin)".to_string(),
            required: false,
            default: None,
            example: Some(json!("my-server")),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_hostname"
        })
    }

    fn example_output(&self) -> String {
        "Current hostname: my-computer".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let new_hostname = parameters.get("set_hostname").and_then(|v| v.as_str());
        if let Some(name) = new_hostname {
            #[cfg(not(target_os = "windows"))]
            {
                let _ = exec_async("sudo", &["hostname", name], None).await;
            }
            #[cfg(target_os = "windows")]
            {
                use crate::exec_async;
                let _ = exec_async(
                    "powershell",
                    &["-Command", &format!("Rename-Computer -NewName '{}'", name)],
                    None,
                )
                .await;
            }
            Ok(format!("Hostname changed to: {}", name))
        } else {
            let hostname = System::host_name();
            Ok(format!(
                "Current hostname: {}",
                hostname.unwrap_or_else(|| "unknown".to_string())
            ))
        }
    }
}

/// A skill for getting current user information.
#[derive(Debug)]
pub struct OsGetUserDriver;

#[async_trait::async_trait]
impl Driver for OsGetUserDriver {
    fn name(&self) -> &str {
        "os_get_user"
    }

    fn description(&self) -> &str {
        "Get current user information"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the current username, home directory, and user ID"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_user"
        })
    }

    fn example_output(&self) -> String {
        "Username: john\nUID: 1000\nGroups: sudo, docker".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let users = Users::new_with_refreshed_list();
        let current_user = users.iter().next();
        if let Some(user) = current_user {
            let groups: Vec<String> = user.groups().iter().map(|g| g.name().to_string()).collect();
            Ok(format!(
                "Username: {}\nUID: {}\nGroups: {}",
                user.name().to_string(),
                user.id().to_string(),
                groups.join(", ")
            ))
        } else {
            Ok("User information could not be found.".to_string())
        }
    }
}

/// A skill for getting disk usage information.
#[derive(Debug)]
pub struct OsDiskUsageDriver;

#[async_trait::async_trait]
impl Driver for OsDiskUsageDriver {
    fn name(&self) -> &str {
        "os_disk_usage"
    }

    fn description(&self) -> &str {
        "Get disk usage information for all mounted filesystems"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check available disk space and usage percentages"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Specific path to check (default: all mounts)".to_string(),
            required: false,
            default: None,
            example: Some(json!("/")),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_disk_usage",
            "parameters": {
                "path": "/home"
            }
        })
    }

    fn example_output(&self) -> String {
        "Filesystem      Size  Used  Avail  Use%\n/dev/sda1       100G   45G    55G    45%"
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let _specific_path = parameters.get("path").and_then(|v| v.as_str());
        let disks = Disks::new_with_refreshed_list();
        let mut output = vec![format!(
            "{:<20} {:<10} {:<10} {:<10} {:<6}",
            "Filesystem", "Total", "Used", "Avail", "Use%"
        )];
        output.push("-".repeat(56));
        for disk in disks.list() {
            let total_gb = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = total_gb - available_gb;
            let used_percent = if total_gb > 0.0 {
                (used_gb / total_gb) * 100.0
            } else {
                0.0
            };
            let mount_point = disk.mount_point().display().to_string();
            if mount_point.len() > 20 {
                output.push(format!(
                    "{}\n{:<20} {:<10.1}G {:<10.1}G {:<10.1}G {:<6.1}%",
                    mount_point, "", total_gb, used_gb, available_gb, used_percent
                ));
            } else {
                output.push(format!(
                    "{:<20} {:<10.1}G {:<10.1}G {:<10.1}G {:<6.1}%",
                    mount_point, total_gb, used_gb, available_gb, used_percent
                ));
            }
        }
        Ok(output.join("\n"))
    }
}

/// A skill for getting memory information.
#[derive(Debug)]
pub struct OsMemoryInfoDriver;

#[async_trait::async_trait]
impl Driver for OsMemoryInfoDriver {
    fn name(&self) -> &str {
        "os_memory_info"
    }

    fn description(&self) -> &str {
        "Get system memory (RAM) information"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check total, used, and available memory"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_memory_info"
        })
    }

    fn example_output(&self) -> String {
        "Total Memory: 16.0 GB\nUsed Memory: 8.2 GB (51%)\nAvailable Memory: 7.8 GB".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let mut system = System::new();
        system.refresh_memory();
        let total_gb = system.total_memory() as f64 / (1024.0 * 1024.0);
        let used_gb = system.used_memory() as f64 / (1024.0 * 1024.0);
        let free_gb = system.free_memory() as f64 / (1024.0 * 1024.0);
        let used_percent = (used_gb / total_gb) * 100.0;
        Ok(format!(
            "Total Memory: {:.1} GB\nUsed Memory: {:.1} GB ({:.0}%)\nFree Memory: {:.1} GB\nAvailable Memory: {:.1} GB",
            total_gb,
            used_gb,
            used_percent,
            free_gb,
            system.available_memory() as f64 / (1024.0 * 1024.0)
        ))
    }
}

/// A skill for getting CPU information.
#[derive(Debug)]
pub struct OsCpuInfoDriver;

#[async_trait::async_trait]
impl Driver for OsCpuInfoDriver {
    fn name(&self) -> &str {
        "os_cpu_info"
    }

    fn description(&self) -> &str {
        "Get CPU information including cores, frequency, and usage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check CPU specifications and current usage"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_cpu_info"
        })
    }

    fn example_output(&self) -> String {
        "CPU cores: 8\nPhysical cores: 4\nCPU usage: 15%\nFrequency: 2.4 GHz".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let mut system = System::new();
        system.refresh_cpu_all();
        let physical_cores = System::physical_core_count().unwrap_or(0);
        let total_cores = system.cpus().len();
        let cpu_usage = system.global_cpu_usage();
        let frequency_info = if let Some(freq_mhz) = system.cpus().first().map(|c| c.frequency()) {
            format!("Frequency: {:.1} GHz", freq_mhz as f64 / 1000.0)
        } else {
            "Frequency: unknown".to_string()
        };
        Ok(format!(
            "CPU cores: {}\nPhysical cores: {}\nCPU usage: {:.1}%\n{}",
            total_cores, physical_cores, cpu_usage, frequency_info
        ))
    }
}

/// A skill for getting network interface information.
#[derive(Debug)]
pub struct OsNetworkInfoDriver;

#[async_trait::async_trait]
impl Driver for OsNetworkInfoDriver {
    fn name(&self) -> &str {
        "os_network_info"
    }

    fn description(&self) -> &str {
        "Get network interface information"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check network interfaces, IP addresses, and MAC addresses"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "interface".to_string(),
            param_type: "string".to_string(),
            description: "Specific interface to show (default: all)".to_string(),
            required: false,
            default: None,
            example: Some(json!("eth0")),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_network_info"
        })
    }

    fn example_output(&self) -> String {
        "eth0: 192.168.1.100 (MAC: 00:11:22:33:44:55)\nwlan0: 10.0.0.1 (MAC: AA:BB:CC:DD:EE:FF)"
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let filter_interface = parameters.get("interface").and_then(|v| v.as_str());
        let networks = Networks::new_with_refreshed_list();
        let mut output = Vec::new();
        for (interface_name, data) in networks.list() {
            if let Some(filter) = filter_interface {
                if !interface_name.to_string().contains(filter) {
                    continue;
                }
            }
            let ips: Vec<String> = data
                .ip_networks()
                .iter()
                .map(|ip| ip.addr.to_string())
                .collect();
            let mac = data.mac_address().to_string();
            let mac = if mac == "00:00:00:00:00:00" {
                "N/A".to_string()
            } else {
                mac
            };
            output.push(format!(
                "{}: {} (MAC: {})",
                interface_name.to_string(),
                ips.join(", "),
                mac
            ));
        }
        if output.is_empty() {
            Ok("No network interfaces found".to_string())
        } else {
            Ok(output.join("\n"))
        }
    }
}

/// A skill for getting battery information (laptops).
#[derive(Debug)]
pub struct OsBatteryInfoDriver;

#[async_trait::async_trait]
impl Driver for OsBatteryInfoDriver {
    fn name(&self) -> &str {
        "os_battery_info"
    }

    fn description(&self) -> &str {
        "Get battery status and information (for laptops)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check battery percentage, charging status, and estimated time"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "detailed".to_string(),
            param_type: "boolean".to_string(),
            description: "Show detailed battery information".to_string(),
            required: false,
            default: Some(json!(false)),
            example: Some(json!(true)),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_battery_info"
        })
    }

    fn example_output(&self) -> String {
        "Battery: 75% (Charging)\nTime remaining: 2h 30m".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let detailed = parameters
            .get("detailed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        #[cfg(target_os = "linux")]
        {
            let result = exec_async(
                "upower",
                &["-i", "/org/freedesktop/UPower/devices/battery_BAT0"],
                None,
            )
            .await;
            if let Ok(out) = result {
                let info = out.stdout;
                if detailed {
                    return Ok(info);
                }
                let percentage = info
                    .lines()
                    .find(|l| l.contains("percentage"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim());
                let state = info
                    .lines()
                    .find(|l| l.contains("state"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim());
                if let (Some(pct), Some(st)) = (percentage, state) {
                    return Ok(format!(
                        "Battery: {} ({})\nTime remaining: check detailed",
                        pct, st
                    ));
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            let result = exec_async("pmset", &["-g", "batt"], None).await?;
            let info = result.stdout;
            if detailed {
                return Ok(info);
            }
            if let Some(line) = info.lines().find(|l| l.contains('%')) {
                return Ok(format!("Battery: {}", line.trim()));
            }
        }
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            let result = exec_async("powercfg", &["/getbatteryreport"], None).await?;
            if detailed {
                return Ok(result.stdout);
            }
        }
        Ok("Battery information not available or system is not a laptop".to_string())
    }
}

/// A skill for sending desktop notifications.
#[derive(Debug)]
pub struct OsNotificationDriver;

#[async_trait::async_trait]
impl Driver for OsNotificationDriver {
    fn name(&self) -> &str {
        "os_notification"
    }

    fn description(&self) -> &str {
        "Send a desktop notification"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to display notifications to the user"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Notification title".to_string(),
                required: true,
                default: None,
                example: Some(json!("Task Complete")),
                enum_values: None,
            },
            DriverParameter {
                name: "message".to_string(),
                param_type: "string".to_string(),
                description: "Notification message body".to_string(),
                required: true,
                default: None,
                example: Some(json!("Your task has finished successfully")),
                enum_values: None,
            },
            DriverParameter {
                name: "urgency".to_string(),
                param_type: "string".to_string(),
                description: "Urgency level: low, normal, critical".to_string(),
                required: false,
                default: Some(json!("normal")),
                example: Some(json!("critical")),
                enum_values: Some(vec![
                    "low".to_string(),
                    "normal".to_string(),
                    "critical".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_notification",
            "parameters": {
                "title": "Alert",
                "message": "Something happened"
            }
        })
    }

    fn example_output(&self) -> String {
        "Notification sent".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let title = parameters
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;
        let message = parameters
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;
        let urgency = parameters
            .get("urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        #[cfg(target_os = "linux")]
        {
            let urgency_flag = match urgency {
                "critical" => "--urgency=critical",
                "low" => "--urgency=low",
                _ => "--urgency=normal",
            };
            let _ = exec_async("notify-send", &[urgency_flag, title, message], None).await;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = exec_async(
                "osascript",
                &[
                    "-e",
                    &format!(
                        "display notification \"{}\" with title \"{}\"",
                        message, title
                    ),
                ],
                None,
            )
            .await;
        }
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            let _ = exec_async(
                "powershell",
                &[
                    "-Command",
                    &format!(
                        "New-BurntToastNotification -Text \"{}\", \"{}\"",
                        title, message
                    ),
                ],
                None,
            )
            .await;
        }
        Ok("Notification sent".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_os_get_uptime() {
        let skill = OsGetUptimeDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("uptime"));
    }

    #[tokio::test]
    async fn test_os_get_load_average() {
        let skill = OsGetLoadAverageDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("Load average"));
    }

    #[tokio::test]
    async fn test_os_memory_info() {
        let skill = OsMemoryInfoDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("Memory"));
    }

    #[tokio::test]
    async fn test_os_cpu_info() {
        let skill = OsCpuInfoDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("CPU"));
    }
}
