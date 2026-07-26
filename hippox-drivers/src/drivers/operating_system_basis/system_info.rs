//! System information driver
//!
//! This driver provides functionality to get system information like OS, CPU, memory, and disk usage.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::{Disks, System};
use tracing::{debug, info};
/// Driver for getting system information
#[derive(Debug)]
pub struct SystemInfoDriver;
#[async_trait::async_trait]
impl Driver for SystemInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "system_info"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get system information like OS, CPU, memory, and disk usage"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks about system specifications, hardware info, or resource usage"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "info_type".to_string(),
            param_type: "string".to_string(),
            description: "Type of info: all, os, cpu, memory, disk, hostname".to_string(),
            required: false,
            default: Some(Value::String("all".to_string())),
            example: Some(Value::String("cpu".to_string())),
            enum_values: Some(vec![
                "all".to_string(),
                "os".to_string(),
                "cpu".to_string(),
                "memory".to_string(),
                "disk".to_string(),
                "hostname".to_string(),
            ]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "system_info",
            "parameters": {
                "info_type": "all"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "OS: Linux 5.15.0\nCPU: Intel i7-10750H (12 cores)\nMemory: 8.2 GB / 16.0 GB (51.2%)\nDisk: /: 120.5 GB / 512.0 GB (23.5%)"
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing system_info driver");
        let info_type = parameters.get("info_type").and_then(|v| v.as_str()).unwrap_or("all");
        let mut sys = System::new_all();
        sys.refresh_all();
        let result = match info_type {
            "os" => {
                debug!("Getting OS info");
                get_os_info(&sys)
            }
            "cpu" => {
                debug!("Getting CPU info");
                get_cpu_info(&sys)
            }
            "memory" => {
                debug!("Getting memory info");
                get_memory_info(&sys)
            }
            "disk" => {
                debug!("Getting disk info");
                get_disk_info()
            }
            "hostname" => {
                debug!("Getting hostname info");
                get_hostname()
            }
            _ => {
                debug!("Getting all system info");
                get_all_info(&sys)
            }
        };
        info!("System info retrieved for type: {}", info_type);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating system_info parameters");
        return Ok(());
    }
}
/// Gets OS information
fn get_os_info(sys: &System) -> String {
    let name = sysinfo::System::name().unwrap_or_else(|| "Unknown".to_string());
    let kernel = sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let os_version = sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "Unknown".to_string());
    debug!("OS info: name={}, kernel={}, version={}, hostname={}", name, kernel, os_version, hostname);
    return format!("OS: {} {}\nKernel: {}\nHostname: {}", name, os_version, kernel, hostname);
}
/// Gets CPU information
fn get_cpu_info(sys: &System) -> String {
    let cpu_count = sys.cpus().len();
    let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32;
    debug!("CPU info: count={}, usage={:.1}%", cpu_count, cpu_usage);
    if let Some(cpu) = sys.cpus().first() {
        let brand = cpu.brand();
        return format!("CPU: {} ({} cores)\nUsage: {:.1}%", brand, cpu_count, cpu_usage);
    } else {
        return format!("CPU: {} cores\nUsage: {:.1}%", cpu_count, cpu_usage);
    }
}
/// Gets memory information
fn get_memory_info(sys: &System) -> String {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let used_percent = (used as f64 / total as f64) * 100.0;
    debug!("Memory info: total={}, used={}, used_percent={:.1}%", total, used, used_percent);
    fn format_bytes(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;
        if bytes >= GB { format!("{:.1} GB", bytes as f64 / GB as f64) } else { format!("{:.0} MB", bytes as f64 / MB as f64) }
    }
    return format!("Memory: {} / {} ({:.1}%)", format_bytes(used), format_bytes(total), used_percent);
}
/// Gets disk information
fn get_disk_info() -> String {
    debug!("Getting disk info");
    let mut result = String::from("Disk Usage:\n");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let used_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        let mount_point = disk.mount_point().to_string_lossy();
        debug!(
            "Disk info: mount={}, used={:.1}GB, total={:.1}GB, used_percent={:.1}%",
            mount_point,
            used as f64 / (1024.0 * 1024.0 * 1024.0),
            total as f64 / (1024.0 * 1024.0 * 1024.0),
            used_percent
        );
        result.push_str(&format!(
            "  {}: {:.1} GB / {:.1} GB ({:.1}%)\n",
            mount_point,
            used as f64 / (1024.0 * 1024.0 * 1024.0),
            total as f64 / (1024.0 * 1024.0 * 1024.0),
            used_percent
        ));
    }
    return result;
}
/// Gets hostname information
fn get_hostname() -> String {
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    debug!("Hostname: {}", hostname);
    return format!("Hostname: {}", hostname);
}
/// Gets all system information
fn get_all_info(sys: &System) -> String {
    debug!("Getting all system info");
    return format!("{}\n\n{}\n\n{}\n\n{}", get_os_info(sys), get_cpu_info(sys), get_memory_info(sys), get_disk_info());
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::DriverCategory;
    use sysinfo::System;
    #[tokio::test]
    async fn test_system_info_skill_all_types() {
        let skill = SystemInfoDriver;
        let info_types = vec!["os", "cpu", "memory", "disk", "hostname", "all"];
        for info_type in info_types {
            let mut params = HashMap::new();
            params.insert("info_type".to_string(), json!(info_type));
            let result = skill.execute(&params, None, None).await;
            assert!(result.is_ok(), "Failed for info_type: {}", info_type);
            let output = result.unwrap();
            assert!(!output.is_empty(), "Empty output for info_type: {}", info_type);
            match info_type {
                "os" => {
                    assert!(output.contains("OS:") || output.contains("Kernel:") || output.contains("Hostname:"));
                }
                "cpu" => {
                    assert!(output.contains("CPU:") || output.contains("Usage:"));
                }
                "memory" => {
                    assert!(output.contains("Memory:"));
                }
                "disk" => {
                    assert!(output.contains("Disk Usage:"));
                }
                "hostname" => {
                    assert!(output.contains("Hostname:"));
                }
                "all" => {
                    assert!(output.contains("OS:") || output.contains("CPU:") || output.contains("Memory:") || output.contains("Disk Usage:"));
                }
                _ => {}
            }
        }
    }
    #[tokio::test]
    async fn test_system_info_skill_default_parameter() {
        let skill = SystemInfoDriver;
        let empty_params = HashMap::new();
        let result = skill.execute(&empty_params, None, None).await;
        assert!(result.is_ok(), "Execution with empty parameters failed");
        let default_output = result.unwrap();
        assert!(!default_output.is_empty(), "Default output should not be empty");
        let mut all_params = HashMap::new();
        all_params.insert("info_type".to_string(), json!("all"));
        let all_result = skill.execute(&all_params, None, None).await.unwrap();
        assert!(default_output.contains("OS:") || default_output.contains("CPU:") || default_output.contains("Memory:"));
        assert_eq!(default_output.contains("OS:"), all_result.contains("OS:"), "Default and 'all' outputs should both contain OS info or both not");
    }
    #[test]
    fn test_disk_info_formatting() {
        let disk_info = get_disk_info();
        assert!(disk_info.contains("Disk Usage:"), "Disk info missing header");
        if disk_info.len() > "Disk Usage:\n".len() {
            let lines: Vec<&str> = disk_info.lines().collect();
            for line in &lines[1..] {
                if !line.trim().is_empty() {
                    assert!(line.contains(":") && (line.contains("GB") || line.contains("%")), "Disk line has unexpected format: {}", line);
                }
            }
        }
    }
    #[test]
    fn test_helper_functions_return_valid_strings() {
        let mut sys = System::new_all();
        sys.refresh_all();
        let os_info = get_os_info(&sys);
        assert!(!os_info.is_empty(), "OS info should not be empty");
        let cpu_info = get_cpu_info(&sys);
        assert!(!cpu_info.is_empty(), "CPU info should not be empty");
        let memory_info = get_memory_info(&sys);
        assert!(!memory_info.is_empty(), "Memory info should not be empty");
        let hostname_info = get_hostname();
        assert!(!hostname_info.is_empty(), "Hostname info should not be empty");
        let all_info = get_all_info(&sys);
        assert!(!all_info.is_empty(), "All info should not be empty");
    }
    #[test]
    fn test_skill_metadata() {
        let skill = SystemInfoDriver;
        assert_eq!(skill.name(), "system_info");
        assert!(!skill.description().is_empty());
        assert!(!skill.usage_hint().is_empty());
        assert_eq!(skill.category(), DriverCategory::OperatingSystemBasis);
        assert!(!skill.example_output().is_empty());
        let params = skill.parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "info_type");
        assert_eq!(params[0].param_type, "string");
        assert!(!params[0].required);
        let example_call = skill.example_call().unwrap();
        assert!(example_call.is_object());
        assert!(example_call.get("action").is_some());
        assert!(example_call.get("parameters").is_some());
    }
}
