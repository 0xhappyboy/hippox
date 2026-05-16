use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use sysinfo::{Disks, System};

use crate::executors::types::Skill;

#[derive(Debug)]
pub struct SystemInfoSkill;

#[async_trait::async_trait]
impl Skill for SystemInfoSkill {
    fn name(&self) -> &str {
        "system_info"
    }

    fn description(&self) -> &str {
        "Get system information like OS, CPU, memory, and disk usage"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let info_type = parameters
            .get("info_type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");
        let mut sys = System::new_all();
        sys.refresh_all();
        match info_type {
            "os" => Ok(get_os_info(&sys)),
            "cpu" => Ok(get_cpu_info(&sys)),
            "memory" => Ok(get_memory_info(&sys)),
            "disk" => Ok(get_disk_info(&sys)),
            "hostname" => Ok(get_hostname()),
            _ => Ok(get_all_info(&sys)),
        }
    }

    fn validate(&self, _parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}

fn get_os_info(sys: &System) -> String {
    let name = sysinfo::System::name().unwrap_or_else(|| "Unknown".to_string());
    let kernel = sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let os_version = sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "Unknown".to_string());
    format!(
        "OS: {} {}\nKernel: {}\nHostname: {}",
        name, os_version, kernel, hostname
    )
}

fn get_cpu_info(sys: &System) -> String {
    let cpu_count = sys.cpus().len();
    let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32;
    if let Some(cpu) = sys.cpus().first() {
        let brand = cpu.brand();
        format!(
            "CPU: {} ({} cores)\nUsage: {:.1}%",
            brand, cpu_count, cpu_usage
        )
    } else {
        format!("CPU: {} cores\nUsage: {:.1}%", cpu_count, cpu_usage)
    }
}

fn get_memory_info(sys: &System) -> String {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = total - used;
    let used_percent = (used as f64 / total as f64) * 100.0;
    fn format_bytes(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;
        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else {
            format!("{:.0} MB", bytes as f64 / MB as f64)
        }
    }
    format!(
        "Memory: {} / {} ({:.1}%)",
        format_bytes(used),
        format_bytes(total),
        used_percent
    )
}

fn get_disk_info(sys: &System) -> String {
    let mut result = String::from("Disk Usage:\n");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let mount_point = disk.mount_point().to_string_lossy();
        result.push_str(&format!(
            "  {}: {:.1} GB / {:.1} GB ({:.1}%)\n",
            mount_point,
            used as f64 / (1024.0 * 1024.0 * 1024.0),
            total as f64 / (1024.0 * 1024.0 * 1024.0),
            used_percent
        ));
    }
    result
}

fn get_hostname() -> String {
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    format!("Hostname: {}", hostname)
}

fn get_all_info(sys: &System) -> String {
    format!(
        "{}\n\n{}\n\n{}\n\n{}",
        get_os_info(sys),
        get_cpu_info(sys),
        get_memory_info(sys),
        get_disk_info(sys)
    )
}
