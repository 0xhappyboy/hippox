use crate::{DriverCallback, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Driver, DriverParameter};
use crate::{RequestConfig, DriverCategory, execute};

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    params
        .get(name)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

#[derive(Debug)]
pub struct SendTelegramDriver;

#[async_trait::async_trait]
impl Driver for SendTelegramDriver {
    fn name(&self) -> &str {
        "send_telegram"
    }
    fn description(&self) -> &str {
        "Send a message via Telegram Bot"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to send a Telegram message"
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::SocialPlatform
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
                enum_values: Some(vec![
                    "HTML".to_string(),
                    "MarkdownV2".to_string(),
                    "Markdown".to_string(),
                ]),
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "send_telegram", "parameters": { "bot_token": "123456:ABC", "chat_id": "123456789", "text": "Hello" } })
    }

    fn example_output(&self) -> String {
        "Telegram message sent successfully".to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let bot_token = get_param_string(parameters, "bot_token")?;
        let chat_id = get_param_string(parameters, "chat_id")?;
        let text = get_param_string(parameters, "text")?;
        let parse_mode = parameters
            .get("parse_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("HTML");
        let disable_notification = get_param_bool(parameters, "disable_notification", false);

        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
        let mut body = HashMap::new();
        body.insert("chat_id".to_string(), json!(chat_id));
        body.insert("text".to_string(), json!(text));
        body.insert("parse_mode".to_string(), json!(parse_mode));
        body.insert(
            "disable_notification".to_string(),
            json!(disable_notification),
        );

        let http_config = RequestConfig {
            url,
            method: "POST".to_string(),
            headers: Some([("Content-Type".to_string(), "application/json".to_string())].into()),
            body: Some(serde_json::to_string(&body)?),
            timeout_secs: Some(30),
        };

        let response = execute(&http_config).await?;
        if response.is_success {
            Ok(format!(
                "Telegram message sent successfully to chat {}",
                chat_id
            ))
        } else {
            Err(anyhow::anyhow!(
                "Failed to send Telegram message: {}",
                response.body
            ))
        }
    }
}
