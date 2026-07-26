//! CPU usage driver module
//!
//! This module provides functionality to get current CPU usage percentage
//! for overall and per-core usage.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting CPU usage
#[derive(Debug)]
pub struct CpuUsageDriver;
#[async_trait::async_trait]
impl Driver for CpuUsageDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "cpu_usage";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get current CPU usage percentage for overall and per-core usage";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to monitor CPU load and identify performance bottlenecks";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "interval_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Measurement interval in milliseconds (default: 500)".to_string(),
                required: false,
                default: Some(Value::Number(500.into())),
                example: Some(Value::Number(1000.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "per_core".to_string(),
                param_type: "boolean".to_string(),
                description: "Show per-core usage (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "cpu_usage",
            "parameters": {
                "interval_ms": 500,
                "per_core": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Overall CPU Usage: 45.2%
Per-Core Usage:
Core 0: 32.1%
Core 1: 67.4%
Core 2: 12.8%
Core 3: 89.2%"#
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemCpu;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing cpu_usage driver");
        let interval = parameters.get("interval_ms").and_then(|v| v.as_u64()).unwrap_or(500) as u64;
        let per_core = parameters.get("per_core").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!("Measuring CPU usage with interval {}ms, per_core: {}", interval, per_core);
        let mut system = System::new_all();
        system.refresh_cpu_usage();
        thread::sleep(Duration::from_millis(interval));
        system.refresh_cpu_usage();
        let overall = system.global_cpu_usage();
        let mut output = format!("Overall CPU Usage: {:.1}%\n\n", overall);
        if per_core {
            output.push_str("Per-Core Usage:\n");
            let cpus = system.cpus();
            for (i, cpu) in cpus.iter().enumerate() {
                output.push_str(&format!("Core {}: {:.1}%\n", i, cpu.cpu_usage()));
            }
        }
        info!("CPU usage retrieved: {:.1}% overall", overall);
        return Ok(output);
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
    fn test_cpu_usage_metadata() {
        let driver = CpuUsageDriver;
        assert_eq!(driver.name(), "cpu_usage");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
