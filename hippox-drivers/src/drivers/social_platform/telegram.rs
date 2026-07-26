//! Telegram message driver
//!
//! This driver provides functionality to send messages via Telegram Bot.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, RequestConfig, execute, types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Helper function to get a string parameter
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name))
}
/// Helper function to get a bool parameter with default
fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    params.get(name).and_then(|v| v.as_bool()).unwrap_or(default)
}
/// Driver for sending Telegram messages
#[derive(Debug)]
pub struct SendTelegramDriver;
#[async_trait::async_trait]
impl Driver for SendTelegramDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "send_telegram"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a message via Telegram Bot"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to send a Telegram message"
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::SocialPlatform
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "bot_token".to_string(),
                param_type: "string".to_string(),
                description: "Telegram bot token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("123456:ABC-DEF1234".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "chat_id".to_string(),
                param_type: "string".to_string(),
                description: "Telegram chat ID".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("123456789".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Message text to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello from Hippo!".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "parse_mode".to_string(),
                param_type: "string".to_string(),
                description: "Parse mode: 'HTML', 'MarkdownV2', or 'Markdown'".to_string(),
                required: false,
                default: Some(Value::String("HTML".to_string())),
                example: Some(Value::String("Markdown".to_string())),
                enum_values: Some(vec!["HTML".to_string(), "MarkdownV2".to_string(), "Markdown".to_string()]),
            },
            DriverParameter {
                name: "disable_notification".to_string(),
                param_type: "boolean".to_string(),
                description: "Send silently".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "send_telegram",
            "parameters": {
                "bot_token": "123456:ABC",
                "chat_id": "123456789",
                "text": "Hello"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Telegram message sent successfully".to_string()
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing send_telegram driver");
        let bot_token = get_param_string(parameters, "bot_token")?;
        let chat_id = get_param_string(parameters, "chat_id")?;
        let text = get_param_string(parameters, "text")?;
        let parse_mode = parameters.get("parse_mode").and_then(|v| v.as_str()).unwrap_or("HTML");
        let disable_notification = get_param_bool(parameters, "disable_notification", false);
        info!("Telegram send: bot_token={}, chat_id={}, text_len={}", bot_token, chat_id, text.len());
        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
        let mut body = HashMap::new();
        body.insert("chat_id".to_string(), json!(chat_id));
        body.insert("text".to_string(), json!(text));
        body.insert("parse_mode".to_string(), json!(parse_mode));
        body.insert("disable_notification".to_string(), json!(disable_notification));
        let http_config = RequestConfig {
            url,
            method: "POST".to_string(),
            headers: Some([("Content-Type".to_string(), "application/json".to_string())].into()),
            body: Some(serde_json::to_string(&body).map_err(|e| DriverError::execution(format!("Failed to serialize body: {}", e)))?),
            timeout_secs: Some(30),
        };
        let response = execute(&http_config).await?;
        if response.is_success {
            info!("Telegram message sent successfully to chat {}", chat_id);
            Ok(format!("Telegram message sent successfully to chat {}", chat_id))
        } else {
            Err(DriverError::execution(format!("Failed to send Telegram message: {}", response.body)))
        }
    }
}
