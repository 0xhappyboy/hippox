//! CPU load average driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_cpu::common::CpuLoadAverage,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting CPU load averages
#[derive(Debug)]
pub struct CpuLoadDriver;

#[async_trait::async_trait]
impl Driver for CpuLoadDriver {
    fn name(&self) -> &str {
        "cpu_load"
    }

    fn description(&self) -> &str {
        "Get system load averages for 1, 5, and 15 minute intervals"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to understand overall system load trends over time"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_load",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"Load Average:
1 minute:  2.34
5 minutes: 1.87
15 minutes: 1.56

(Values > 1.0 indicate system is busy)"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let load_avg = get_load_average()?;

        let output = format!(
            "Load Average:\n\
             1 minute:  {:.2}\n\
             5 minutes: {:.2}\n\
             15 minutes: {:.2}\n\n\
             (Values > 1.0 indicate system is busy)",
            load_avg.one_minute, load_avg.five_minutes, load_avg.fifteen_minutes
        );

        Ok(output)
    }
}

fn get_load_average() -> Result<CpuLoadAverage> {
    #[cfg(target_os = "linux")]
    {
        let content = std::fs::read_to_string("/proc/loadavg")?;
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            return Ok(CpuLoadAverage {
                one_minute: parts[0].parse().unwrap_or(0.0),
                five_minutes: parts[1].parse().unwrap_or(0.0),
                fifteen_minutes: parts[2].parse().unwrap_or(0.0),
            });
        }
        Err(anyhow::anyhow!("Failed to parse load average"))
    }

    #[cfg(target_os = "windows")]
    {
        // Windows doesn't have load average concept, calculate a pseudo-load
        // based on CPU usage over time
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_usage();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        system.refresh_cpu_usage();
        let usage = system.global_cpu_usage();
        // Convert to a load-like value
        let load = (usage / 100.0) as f64;
        Ok(CpuLoadAverage {
            one_minute: load,
            five_minutes: load * 0.9,
            fifteen_minutes: load * 0.8,
        })
    }

    #[cfg(target_os = "macos")]
    {
        use libc::{c_double, getloadavg};

        let mut loadavg = [0.0 as c_double; 3];
        unsafe {
            let result = getloadavg(loadavg.as_mut_ptr(), 3);
            if result < 0 {
                return Err(anyhow::anyhow!("Failed to get load average"));
            }
        }
        Ok(CpuLoadAverage {
            one_minute: loadavg[0],
            five_minutes: loadavg[1],
            fifteen_minutes: loadavg[2],
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        // Fallback: use sysinfo to estimate
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_usage();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        system.refresh_cpu_usage();

        let usage = system.global_cpu_usage() / 100.0;
        Ok(CpuLoadAverage {
            one_minute: usage,
            five_minutes: usage * 0.9,
            fifteen_minutes: usage * 0.8,
        })
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
