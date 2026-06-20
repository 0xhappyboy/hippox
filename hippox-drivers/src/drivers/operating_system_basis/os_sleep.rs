//! OS sleep driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsSleepDriver;
#[async_trait::async_trait]
impl Driver for OsSleepDriver {
    fn name(&self) -> &str {
        "os_sleep"
    }
    fn description(&self) -> &str {
        "Put the system to sleep (suspend to RAM)"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to save power by putting the system into low-power sleep mode"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_sleep"
        })
    }
    fn example_output(&self) -> String {
        "System is going to sleep".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async(
                "rundll32.exe",
                &["powrprof.dll,SetSuspendState", "0", "1", "0"],
                None,
            )
            .await?;
        }
        #[cfg(target_os = "macos")]
        {
            exec_async("pmset", &["sleepnow"], None).await?;
        }
        #[cfg(target_os = "linux")]
        {
            exec_async("systemctl", &["suspend"], None).await?;
        }
        Ok("System is going to sleep".to_string())
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
