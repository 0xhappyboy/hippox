//! OS sleep driver
//!
//! This driver provides functionality to put the system to sleep (suspend to RAM).
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for putting the system to sleep
#[derive(Debug)]
pub struct OsSleepDriver;
#[async_trait::async_trait]
impl Driver for OsSleepDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_sleep"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Put the system to sleep (suspend to RAM)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to save power by putting the system into low-power sleep mode"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_sleep"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System is going to sleep".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_sleep driver");
        #[cfg(target_os = "windows")]
        {
            debug!("Putting system to sleep on Windows");
            exec_async("rundll32.exe", &["powrprof.dll,SetSuspendState", "0", "1", "0"], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to sleep on Windows: {}", e)))?;
            info!("System is going to sleep on Windows");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Putting system to sleep on macOS");
            exec_async("pmset", &["sleepnow"], None).await.map_err(|e| DriverError::execution(format!("Failed to sleep on macOS: {}", e)))?;
            info!("System is going to sleep on macOS");
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Putting system to sleep on Linux");
            exec_async("systemctl", &["suspend"], None).await.map_err(|e| DriverError::execution(format!("Failed to sleep on Linux: {}", e)))?;
            info!("System is going to sleep on Linux");
        }
        return Ok("System is going to sleep".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_sleep_metadata() {
        let driver = OsSleepDriver;
        assert_eq!(driver.name(), "os_sleep");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
