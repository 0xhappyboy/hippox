//! OS screen on driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsScreenOnDriver;
#[async_trait::async_trait]
impl Driver for OsScreenOnDriver {
    fn name(&self) -> &str {
        "os_screen_on"
    }
    fn description(&self) -> &str {
        "Turn on the display"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to turn the screen back on after it was turned off"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_screen_on"
        })
    }
    fn example_output(&self) -> String {
        "Display turned on".to_string()
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
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xset")
                .args(["dpms", "force", "on"])
                .output();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("caffeinate")
                .args(["-u", "-t", "1"])
                .output();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("powershell")
                .args([
                    "-Command",
                    "(Add-Type -MemberDefinition '[DllImport(\"user32.dll\")] public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);' -Name 'WinAPI' -Namespace WinAPI)::SendMessage(0xffff, 0x0112, 0xF170, -1)"
                ])
                .output();
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            return Err(anyhow::anyhow!("Screen on is not supported on this platform"));
        }
        Ok("Display turned on".to_string())
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