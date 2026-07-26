//! OS screen on driver
//!
//! This driver provides functionality to turn on the display.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for turning on the display
#[derive(Debug)]
pub struct OsScreenOnDriver;
#[async_trait::async_trait]
impl Driver for OsScreenOnDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_screen_on"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Turn on the display"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to turn the screen back on after it was turned off"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_screen_on"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display turned on".to_string();
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
        debug!("Executing os_screen_on driver");
        #[cfg(target_os = "linux")]
        {
            debug!("Turning on screen on Linux");
            let _ = Command::new("xset").args(["dpms", "force", "on"]).output();
            info!("Display turned on on Linux");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Turning on screen on macOS");
            let _ = Command::new("caffeinate").args(["-u", "-t", "1"]).output();
            info!("Display turned on on macOS");
        }
        #[cfg(target_os = "windows")]
        {
            debug!("Turning on screen on Windows");
            let _ = Command::new("powershell")
                .args([
                    "-Command",
                    "(Add-Type -MemberDefinition '[DllImport(\"user32.dll\")] public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);' -Name 'WinAPI' -Namespace WinAPI)::SendMessage(0xffff, 0x0112, 0xF170, -1)"
                ])
                .output();
            info!("Display turned on on Windows");
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            debug!("Screen on not supported on this platform");
            return Err(DriverError::execution("Screen on is not supported on this platform"));
        }
        return Ok("Display turned on".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_screen_on_metadata() {
        let driver = OsScreenOnDriver;
        assert_eq!(driver.name(), "os_screen_on");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
