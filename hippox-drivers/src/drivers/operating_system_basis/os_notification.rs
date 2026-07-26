//! OS notification driver
//!
//! This driver provides functionality to send desktop notifications.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for sending desktop notifications
#[derive(Debug)]
pub struct OsNotificationDriver;
#[async_trait::async_trait]
impl Driver for OsNotificationDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_notification"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a desktop notification"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to display notifications to the user"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Notification title".to_string(),
                required: true,
                default: None,
                example: Some(json!("Task Complete")),
                enum_values: None,
            },
            DriverParameter {
                name: "message".to_string(),
                param_type: "string".to_string(),
                description: "Notification message body".to_string(),
                required: true,
                default: None,
                example: Some(json!("Your task has finished successfully")),
                enum_values: None,
            },
            DriverParameter {
                name: "urgency".to_string(),
                param_type: "string".to_string(),
                description: "Urgency level: low, normal, critical".to_string(),
                required: false,
                default: Some(json!("normal")),
                example: Some(json!("critical")),
                enum_values: Some(vec!["low".to_string(), "normal".to_string(), "critical".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_notification",
            "parameters": {
                "title": "Alert",
                "message": "Something happened"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Notification sent".to_string();
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
        debug!("Executing os_notification driver");
        let title = parameters.get("title").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("title"))?;
        let message = parameters.get("message").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("message"))?;
        let urgency = parameters.get("urgency").and_then(|v| v.as_str()).unwrap_or("normal");
        info!("Sending notification: title='{}', urgency='{}'", title, urgency);
        #[cfg(target_os = "linux")]
        {
            debug!("Sending notification on Linux");
            let urgency_flag = match urgency {
                "critical" => "--urgency=critical",
                "low" => "--urgency=low",
                _ => "--urgency=normal",
            };
            let _ = exec_async("notify-send", &[urgency_flag, title, message], None).await;
            info!("Notification sent on Linux");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Sending notification on macOS");
            let _ = exec_async("osascript", &["-e", &format!("display notification \"{}\" with title \"{}\"", message, title)], None).await;
            info!("Notification sent on macOS");
        }
        #[cfg(target_os = "windows")]
        {
            debug!("Sending notification on Windows");
            let _ = exec_async("powershell", &["-Command", &format!("New-BurntToastNotification -Text \"{}\", \"{}\"", title, message)], None).await;
            info!("Notification sent on Windows");
        }
        return Ok("Notification sent".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_notification_metadata() {
        let driver = OsNotificationDriver;
        assert_eq!(driver.name(), "os_notification");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
