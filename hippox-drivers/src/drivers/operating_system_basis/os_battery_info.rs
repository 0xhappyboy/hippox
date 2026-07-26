//! OS battery info driver
//!
//! This driver provides functionality to get battery status and information
//! for laptops, including percentage, charging status, and estimated time.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting battery information
#[derive(Debug)]
pub struct OsBatteryInfoDriver;
#[async_trait::async_trait]
impl Driver for OsBatteryInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_battery_info"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get battery status and information (for laptops)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check battery percentage, charging status, and estimated time"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "detailed".to_string(),
            param_type: "boolean".to_string(),
            description: "Show detailed battery information".to_string(),
            required: false,
            default: Some(json!(false)),
            example: Some(json!(true)),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_battery_info"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Battery: 75% (Charging)\nTime remaining: 2h 30m".to_string();
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
        debug!("Executing os_battery_info driver");
        let detailed = parameters.get("detailed").and_then(|v| v.as_bool()).unwrap_or(false);
        #[cfg(target_os = "linux")]
        {
            debug!("Detecting battery info on Linux");
            let result = exec_async("upower", &["-i", "/org/freedesktop/UPower/devices/battery_BAT0"], None).await;
            if let Ok(out) = result {
                let info = out.stdout;
                if detailed {
                    info!("Returning detailed battery info on Linux");
                    return Ok(info);
                }
                let percentage = info.lines().find(|l| l.contains("percentage")).and_then(|l| l.split(':').nth(1)).map(|s| s.trim());
                let state = info.lines().find(|l| l.contains("state")).and_then(|l| l.split(':').nth(1)).map(|s| s.trim());
                if let (Some(pct), Some(st)) = (percentage, state) {
                    info!("Battery info retrieved: {} ({})", pct, st);
                    return Ok(format!("Battery: {} ({})\nTime remaining: check detailed", pct, st));
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Detecting battery info on macOS");
            let result = exec_async("pmset", &["-g", "batt"], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to get battery info on macOS: {}", e)))?;
            let info = result.stdout;
            if detailed {
                info!("Returning detailed battery info on macOS");
                return Ok(info);
            }
            if let Some(line) = info.lines().find(|l| l.contains('%')) {
                info!("Battery info retrieved on macOS");
                return Ok(format!("Battery: {}", line.trim()));
            }
        }
        #[cfg(target_os = "windows")]
        {
            debug!("Detecting battery info on Windows");
            let result = exec_async("powercfg", &["/getbatteryreport"], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to get battery info on Windows: {}", e)))?;
            if detailed {
                info!("Returning detailed battery info on Windows");
                return Ok(result.stdout);
            }
        }
        info!("Battery information not available or system is not a laptop");
        return Ok("Battery information not available or system is not a laptop".to_string());
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
