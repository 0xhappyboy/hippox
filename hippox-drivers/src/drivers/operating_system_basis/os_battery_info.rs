//! OS battery info driver
use crate::{
    DriverCallback, DriverCategory, DriverContext, exec_async,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
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
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
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
            let result = exec_async("powercfg", &["/getbatteryreport"], None).await?;
            if detailed {
                return Ok(result.stdout);
            }
        }
        Ok("Battery information not available or system is not a laptop".to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_battery_info_metadata() {
        let driver = OsBatteryInfoDriver;
        assert_eq!(driver.name(), "os_battery_info");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
