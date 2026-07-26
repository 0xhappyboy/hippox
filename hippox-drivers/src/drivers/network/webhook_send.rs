//! Webhook send driver
//!
//! This driver provides functionality to send a webhook notification via HTTP POST with JSON payload.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};
/// Send a webhook to a URL with JSON payload
pub async fn send_webhook(url: &str, payload: &Value, headers: Option<HashMap<String, String>>) -> DriverResult<String> {
    debug!("Sending webhook to: {}", url);
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| DriverError::execution(format!("Failed to build HTTP client: {}", e)))?;
    let mut request_builder = client.post(url).json(payload);
    if let Some(headers_map) = headers {
        for (key, value) in headers_map {
            request_builder = request_builder.header(&key, value);
        }
    }
    let response = request_builder.send().await.map_err(|e| DriverError::execution(format!("Webhook request failed: {}", e)))?;
    let status = response.status().as_u16();
    info!("Webhook sent: status={}", status);
    let body = response.text().await.map_err(|e| DriverError::execution(format!("Failed to read response: {}", e)))?;
    if status >= 200 && status < 300 {
        Ok(format!("Webhook sent successfully (status: {})\nResponse: {}", status, body))
    } else {
        Err(DriverError::execution(format!("Webhook failed (status: {}): {}", status, body)))
    }
}
/// Driver for sending webhooks
#[derive(Debug)]
pub struct WebhookSendDriver;
#[async_trait::async_trait]
impl Driver for WebhookSendDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "webhook_send"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a webhook notification via HTTP POST with JSON payload"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to send a notification or event to a webhook endpoint"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "webhook_send",
            "parameters": {
                "url": "https://hooks.slack.com/XXXXX",
                "payload": {"text": "Hello from Hippox"}
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Webhook sent successfully (status: 200)".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing webhook_send driver");
        let url = parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("url"))?;
        let payload = parameters.get("payload").ok_or_else(|| DriverError::missing_parameter("payload"))?;
        let headers = parameters.get("headers").and_then(|v| v.as_object()).map(|obj| {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
            map
        });
        info!("Webhook send: url={}, payload_size={}", url, payload.to_string().len());
        let result = send_webhook(url, payload, headers).await?;
        info!("Webhook send successful: {}", result);
        return Ok(result);
    }
}
