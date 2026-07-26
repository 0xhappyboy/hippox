//! OS logout driver
//!
//! This driver provides functionality to log out the current user.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for logging out the current user
#[derive(Debug)]
pub struct OsLogoutDriver;
#[async_trait::async_trait]
impl Driver for OsLogoutDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_logout"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Log out the current user"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to end the current user session"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "force".to_string(),
            param_type: "boolean".to_string(),
            description: "Force logout without confirmation (default: false)".to_string(),
            required: false,
            default: Some(json!(false)),
            example: Some(json!(true)),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_logout"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Logging out current user".to_string();
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
        debug!("Executing os_logout driver");
        let _force = parameters.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        #[cfg(target_os = "windows")]
        {
            debug!("Logging out on Windows");
            exec_async("shutdown", &["/l"], None).await.map_err(|e| DriverError::execution(format!("Failed to logout on Windows: {}", e)))?;
            info!("Logging out current user on Windows");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Logging out on macOS");
            let _ = exec_async("osascript", &["-e", "tell application \"System Events\" to log out"], None).await;
            info!("Logging out current user on macOS");
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Logging out on Linux");
            let _ = exec_async("gnome-session-quit", &["--no-prompt"], None).await;
            let _ = exec_async("pkill", &["-KILL", "-u", "$USER"], None).await;
            info!("Logging out current user on Linux");
        }
        return Ok("Logging out current user".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_logout_metadata() {
        let driver = OsLogoutDriver;
        assert_eq!(driver.name(), "os_logout");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
