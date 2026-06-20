//! CPU usage driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;

/// Driver for getting CPU usage
#[derive(Debug)]
pub struct CpuUsageDriver;

#[async_trait::async_trait]
impl Driver for CpuUsageDriver {
    fn name(&self) -> &str {
        "cpu_usage"
    }

    fn description(&self) -> &str {
        "Get current CPU usage percentage for overall and per-core usage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor CPU load and identify performance bottlenecks"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_usage",
            "parameters": {
                "interval_ms": 500,
                "per_core": true
            }
        })
    }

    fn example_output(&self) -> String {
        r#"Overall CPU Usage: 45.2%

Per-Core Usage:
Core 0: 32.1%
Core 1: 67.4%
Core 2: 12.8%
Core 3: 89.2%"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let interval = parameters
            .get("interval_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(500) as u64;
        let per_core = parameters
            .get("per_core")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        // First refresh to get baseline
        let mut system = System::new_all();
        system.refresh_cpu_usage();
        std::thread::sleep(std::time::Duration::from_millis(interval));
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
        Ok(output)
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
