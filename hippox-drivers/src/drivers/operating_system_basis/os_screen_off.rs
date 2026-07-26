//! OS screen off driver
//!
//! This driver provides functionality to turn off the display for power saving.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for turning off the display
#[derive(Debug)]
pub struct OsScreenOffDriver;
#[async_trait::async_trait]
impl Driver for OsScreenOffDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_screen_off"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Turn off the display (power saving)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to turn off the screen. Moving the mouse or pressing a key will turn it back on."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_screen_off"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Display turned off".to_string();
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
        debug!("Executing os_screen_off driver");
        #[cfg(target_os = "windows")]
        {
            debug!("Turning off screen on Windows");
            let _ = Command::new("powershell")
                .args([
                    "-Command",
                    "(Add-Type -MemberDefinition '[DllImport(\"user32.dll\")] public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);' -Name 'WinAPI' -Namespace WinAPI)::SendMessage(0xffff, 0x0112, 0xF170, 2)"
                ])
                .output();
            info!("Display turned off on Windows");
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Turning off screen on Linux");
            let _ = Command::new("xset").args(["dpms", "force", "off"]).output();
            info!("Display turned off on Linux");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Turning off screen on macOS");
            let _ = Command::new("pmset").args(["displaysleepnow"]).output();
            info!("Display turned off on macOS");
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            debug!("Screen off not supported on this platform");
            return Err(DriverError::execution("Screen off is not supported on this platform"));
        }
        return Ok("Display turned off".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_screen_off_metadata() {
        let driver = OsScreenOffDriver;
        assert_eq!(driver.name(), "os_screen_off");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
