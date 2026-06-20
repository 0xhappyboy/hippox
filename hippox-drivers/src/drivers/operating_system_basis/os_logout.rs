//! OS logout driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsLogoutDriver;
#[async_trait::async_trait]
impl Driver for OsLogoutDriver {
    fn name(&self) -> &str {
        "os_logout"
    }
    fn description(&self) -> &str {
        "Log out the current user"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to end the current user session"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "force".to_string(),
            param_type: "boolean".to_string(),
            description: "Force logout without confirmation (default: false)".to_string(),
            required: false,
            default: Some(json!(false)),
            example: Some(json!(true)),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_logout"
        })
    }
    fn example_output(&self) -> String {
        "Logging out current user".to_string()
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
        let _force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async("shutdown", &["/l"], None).await?;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = exec_async(
                "osascript",
                &["-e", "tell application \"System Events\" to log out"],
                None,
            )
            .await;
        }
        #[cfg(target_os = "linux")]
        {
            let _ = exec_async("gnome-session-quit", &["--no-prompt"], None).await;
            let _ = exec_async("pkill", &["-KILL", "-u", "$USER"], None).await;
        }
        Ok("Logging out current user".to_string())
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
