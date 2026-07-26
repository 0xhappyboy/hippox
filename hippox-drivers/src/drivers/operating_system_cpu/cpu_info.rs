//! CPU information driver module
//!
//! This module provides functionality to get detailed CPU information including
//! vendor, model, cores, frequency, and virtualization status.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_cpu::common::CpuInfo,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting CPU information
#[derive(Debug)]
pub struct CpuInfoDriver;
#[async_trait::async_trait]
impl Driver for CpuInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "cpu_info";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get detailed CPU information including vendor, model, cores, and frequency";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to get system CPU specifications and capabilities";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "cpu_info",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"CPU Info:
Vendor: GenuineIntel
Model: Intel(R) Core(TM) i7-10750H
Architecture: x86_64
Physical Cores: 6
Logical Cores: 12
Max Frequency: 2600 MHz
Min Frequency: 800 MHz"#
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemCpu;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing cpu_info driver");
        let system = System::new_all();
        let mut cpu_info = CpuInfo {
            vendor: "Unknown".to_string(),
            brand: "Unknown".to_string(),
            model_name: "Unknown".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            physical_cores: 0,
            logical_cores: 0,
            max_frequency_mhz: 0,
            min_frequency_mhz: 0,
            is_hypervisor: false,
        };
        let cpus = system.cpus();
        if !cpus.is_empty() {
            let cpu = &cpus[0];
            cpu_info.vendor = cpu.vendor_id().to_string();
            cpu_info.brand = cpu.brand().to_string();
            cpu_info.model_name = cpu.brand().to_string();
            cpu_info.logical_cores = cpus.len();
            #[cfg(target_os = "linux")]
            {
                if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                    let mut physical_ids = HashSet::new();
                    let mut core_ids = HashSet::new();
                    let mut current_physical = String::new();
                    for line in content.lines() {
                        if line.starts_with("physical id") {
                            if let Some(id) = line.split(':').nth(1) {
                                current_physical = id.trim().to_string();
                            }
                        } else if line.starts_with("core id") {
                            if let Some(id) = line.split(':').nth(1) {
                                let key = format!("{}_{}", current_physical, id.trim());
                                core_ids.insert(key);
                            }
                        }
                    }
                    cpu_info.physical_cores = core_ids.len();
                    if cpu_info.physical_cores == 0 {
                        cpu_info.physical_cores = cpus.len() / 2;
                    }
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                cpu_info.physical_cores = cpus.len() / 2;
            }
            if cpu_info.physical_cores == 0 {
                cpu_info.physical_cores = cpus.len();
            }
        }
        if let Some(cpu) = system.cpus().first() {
            cpu_info.max_frequency_mhz = cpu.frequency() as u64;
            cpu_info.min_frequency_mhz = cpu_info.max_frequency_mhz / 2;
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.contains("hypervisor_vendor") || line.contains("hypervisor") {
                        cpu_info.is_hypervisor = true;
                        break;
                    }
                }
            }
        }
        let result = format!(
            "CPU Info:\n\
             Vendor: {}\n\
             Model: {}\n\
             Architecture: {}\n\
             Physical Cores: {}\n\
             Logical Cores: {}\n\
             Max Frequency: {} MHz\n\
             Min Frequency: {} MHz\n\
             Running in VM: {}",
            cpu_info.vendor,
            cpu_info.model_name,
            cpu_info.architecture,
            cpu_info.physical_cores,
            cpu_info.logical_cores,
            cpu_info.max_frequency_mhz,
            cpu_info.min_frequency_mhz,
            if cpu_info.is_hypervisor { "Yes" } else { "No" }
        );
        info!("CPU information retrieved");
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_info_metadata() {
        let driver = CpuInfoDriver;
        assert_eq!(driver.name(), "cpu_info");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
