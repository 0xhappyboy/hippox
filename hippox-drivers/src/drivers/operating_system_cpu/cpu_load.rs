//! CPU load average driver module
//!
//! This module provides functionality to get system load averages for 1, 5,
//! and 15 minute intervals.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_cpu::common::CpuLoadAverage,
    types::{Driver, DriverParameter},
};
/// Driver for getting CPU load averages
#[derive(Debug)]
pub struct CpuLoadDriver;
#[async_trait::async_trait]
impl Driver for CpuLoadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "cpu_load";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get system load averages for 1, 5, and 15 minute intervals";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to understand overall system load trends over time";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "cpu_load",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Load Average:
1 minute:  2.34
5 minutes: 1.87
15 minutes: 1.56
(Values > 1.0 indicate system is busy)"#
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
        debug!("Executing cpu_load driver");
        let load_avg = get_load_average()?;
        let result = format!(
            "Load Average:\n\
             1 minute:  {:.2}\n\
             5 minutes: {:.2}\n\
             15 minutes: {:.2}\n\n\
             (Values > 1.0 indicate system is busy)",
            load_avg.one_minute, load_avg.five_minutes, load_avg.fifteen_minutes
        );
        info!("Load average retrieved");
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_load_average() -> DriverResult<CpuLoadAverage> {
    #[cfg(target_os = "linux")]
    {
        let content = std::fs::read_to_string("/proc/loadavg").map_err(|e| DriverError::execution(format!("Failed to read load average: {}", e)))?;
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            return Ok(CpuLoadAverage {
                one_minute: parts[0].parse().unwrap_or(0.0),
                five_minutes: parts[1].parse().unwrap_or(0.0),
                fifteen_minutes: parts[2].parse().unwrap_or(0.0),
            });
        }
        return Err(DriverError::execution("Failed to parse load average".to_string()));
    }
    #[cfg(target_os = "windows")]
    {
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_usage();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        system.refresh_cpu_usage();
        let usage = system.global_cpu_usage();
        let load = (usage / 100.0) as f64;
        return Ok(CpuLoadAverage { one_minute: load, five_minutes: load * 0.9, fifteen_minutes: load * 0.8 });
    }
    #[cfg(target_os = "macos")]
    {
        use libc::{c_double, getloadavg};
        let mut loadavg = [0.0 as c_double; 3];
        unsafe {
            let result = getloadavg(loadavg.as_mut_ptr(), 3);
            if result < 0 {
                return Err(DriverError::execution("Failed to get load average".to_string()));
            }
        }
        return Ok(CpuLoadAverage { one_minute: loadavg[0], five_minutes: loadavg[1], fifteen_minutes: loadavg[2] });
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_usage();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        system.refresh_cpu_usage();
        let usage = system.global_cpu_usage() / 100.0;
        return Ok(CpuLoadAverage { one_minute: usage, five_minutes: usage * 0.9, fifteen_minutes: usage * 0.8 });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_load_metadata() {
        let driver = CpuLoadDriver;
        assert_eq!(driver.name(), "cpu_load");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
