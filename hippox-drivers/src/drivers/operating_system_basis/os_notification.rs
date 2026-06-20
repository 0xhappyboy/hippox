//! OS notification driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsNotificationDriver;
#[async_trait::async_trait]
impl Driver for OsNotificationDriver {
    fn name(&self) -> &str {
        "os_notification"
    }
    fn description(&self) -> &str {
        "Send a desktop notification"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to display notifications to the user"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
                enum_values: Some(vec![
                    "low".to_string(),
                    "normal".to_string(),
                    "critical".to_string(),
                ]),
            },
        ]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_notification",
            "parameters": {
                "title": "Alert",
                "message": "Something happened"
            }
        })
    }
    fn example_output(&self) -> String {
        "Notification sent".to_string()
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
        let title = parameters
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;
        let message = parameters
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;
        let urgency = parameters
            .get("urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        #[cfg(target_os = "linux")]
        {
            let urgency_flag = match urgency {
                "critical" => "--urgency=critical",
                "low" => "--urgency=low",
                _ => "--urgency=normal",
            };
            let _ = exec_async("notify-send", &[urgency_flag, title, message], None).await;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = exec_async(
                "osascript",
                &[
                    "-e",
                    &format!(
                        "display notification \"{}\" with title \"{}\"",
                        message, title
                    ),
                ],
                None,
            )
            .await;
        }
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            let _ = exec_async(
                "powershell",
                &[
                    "-Command",
                    &format!(
                        "New-BurntToastNotification -Text \"{}\", \"{}\"",
                        title, message
                    ),
                ],
                None,
            )
            .await;
        }
        Ok("Notification sent".to_string())
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
