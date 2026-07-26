//! OS lock driver
//!
//! This driver provides functionality to lock the system screen.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for locking the system screen
#[derive(Debug)]
pub struct OsLockDriver;
#[async_trait::async_trait]
impl Driver for OsLockDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_lock"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Lock the system screen"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to secure the system without logging out"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_lock"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Screen locked".to_string();
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
        debug!("Executing os_lock driver");
        #[cfg(target_os = "windows")]
        {
            debug!("Locking screen on Windows");
            exec_async("rundll32.exe", &["user32.dll,LockWorkStation"], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to lock screen on Windows: {}", e)))?;
            info!("Screen locked on Windows");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Locking screen on macOS");
            let _ =
                exec_async("osascript", &["-e", "tell application \"System Events\" to keystroke \"q\" using {command down, control down}"], None)
                    .await;
            info!("Screen locked on macOS");
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Locking screen on Linux");
            let _ = exec_async("gnome-screensaver-command", &["-l"], None).await;
            let _ = exec_async("xdg-screensaver", &["lock"], None).await;
            info!("Screen locked on Linux");
        }
        return Ok("Screen locked".to_string());
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
