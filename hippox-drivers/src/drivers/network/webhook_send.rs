use crate::common::http::send_webhook;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverCategory, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WebhookSendDriver;

#[async_trait::async_trait]
impl Driver for WebhookSendDriver {
    fn name(&self) -> &str {
        "webhook_send"
    }

    fn description(&self) -> &str {
        "Send a webhook notification via HTTP POST with JSON payload"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to send a notification or event to a webhook endpoint"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "Webhook endpoint URL".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://hooks.slack.com/XXXXX".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "payload".to_string(),
                param_type: "object".to_string(),
                description: "JSON payload to send".to_string(),
                required: true,
                default: None,
                example: Some(json!({"text": "Hello from Hippox"})),
                enum_values: None,
            },
            DriverParameter {
                name: "headers".to_string(),
                param_type: "object".to_string(),
                description: "HTTP headers as key-value pairs".to_string(),
                required: false,
                default: None,
                example: Some(json!({"X-API-Key": "secret"})),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "webhook_send",
            "parameters": {
                "url": "https://hooks.slack.com/XXXXX",
                "payload": {"text": "Hello from Hippox"}
            }
        })
    }

    fn example_output(&self) -> String {
        "Webhook sent successfully (status: 200)".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Sending webhook".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }

        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;

        let payload = parameters
            .get("payload")
            .ok_or_else(|| anyhow::anyhow!("Missing 'payload' parameter"))?;

        let headers = parameters
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        map.insert(k.clone(), s.to_string());
                    }
                }
                map
            });

        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), driver_index, Some(format!("URL: {}", url)));
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Payload: {}", payload)),
            );
            if let Some(h) = &headers {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("Headers: {:?}", h)),
                );
            }
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Sending request...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }

        let result = send_webhook(url, payload, headers).await?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Webhook sent".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("webhook_send".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
    }
}
