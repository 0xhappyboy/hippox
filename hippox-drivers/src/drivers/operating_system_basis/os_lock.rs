//! OS lock driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsLockDriver;
#[async_trait::async_trait]
impl Driver for OsLockDriver {
    fn name(&self) -> &str {
        "os_lock"
    }
    fn description(&self) -> &str {
        "Lock the system screen"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to secure the system without logging out"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_lock"
        })
    }
    fn example_output(&self) -> String {
        "Screen locked".to_string()
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
            exec_async("rundll32.exe", &["user32.dll,LockWorkStation"], None).await?;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = exec_async("osascript", &["-e", "tell application \"System Events\" to keystroke \"q\" using {command down, control down}"], None).await;
        }
        #[cfg(target_os = "linux")]
        {
            let _ = exec_async("gnome-screensaver-command", &["-l"], None).await;
            let _ = exec_async("xdg-screensaver", &["lock"], None).await;
        }
        Ok("Screen locked".to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_lock_metadata() {
        let driver = OsLockDriver;
        assert_eq!(driver.name(), "os_lock");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
