use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    config::{get_telegram_instance, list_telegram_instances},
    executors::{
        RequestConfig, execute,
        types::{Skill, SkillParameter},
    },
};

#[derive(Debug)]
pub struct SendTelegramSkill;

#[async_trait::async_trait]
impl Skill for SendTelegramSkill {
    fn name(&self) -> &str {
        "send_telegram"
    }

    fn description(&self) -> &str {
        "Send a message via Telegram Bot"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to send a Telegram message, notify via Telegram, or send a message to a Telegram chat"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_telegram_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Telegram instance ID (use 'list_telegram_instances' to see available instances)".to_string(),
                required: false,
                default: if instance_ids.is_empty() { None } else { Some(Value::String(instance_ids[0].clone())) },
                example: Some(Value::String("telegram_bot1".to_string())),
                enum_values: if instance_ids.is_empty() { None } else { Some(instance_ids) },
            },
            SkillParameter {
                name: "chat_id".to_string(),
                param_type: "string".to_string(),
                description: "Telegram chat ID (user or group)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("123456789".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Message text to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello from Hippo!".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "parse_mode".to_string(),
                param_type: "string".to_string(),
                description: "Message parse mode: 'HTML', 'MarkdownV2', or 'Markdown'".to_string(),
                required: false,
                default: Some(Value::String("HTML".to_string())),
                example: Some(Value::String("Markdown".to_string())),
                enum_values: Some(vec![
                    "HTML".to_string(),
                    "MarkdownV2".to_string(),
                    "Markdown".to_string(),
                ]),
            },
            SkillParameter {
                name: "disable_notification".to_string(),
                param_type: "boolean".to_string(),
                description: "Send silently without notification sound".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "bot_token".to_string(),
                param_type: "string".to_string(),
                description: "Bot token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "send_telegram",
            "parameters": {
                "instance_id": "telegram_bot1",
                "chat_id": "123456789",
                "text": "Hello from Hippo!"
            }
        })
    }

    fn example_output(&self) -> String {
        "Telegram message sent successfully to chat 123456789 [instance: telegram_bot1]".to_string()
    }

    fn category(&self) -> &str {
        "messaging"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let chat_id = parameters
            .get("chat_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'chat_id' parameter"))?;
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'text' parameter"))?;
        let parse_mode = parameters
            .get("parse_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("HTML");
        let disable_notification = parameters
            .get("disable_notification")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());

        // Get instance configuration
        let instance = if let Some(id) = instance_id {
            get_telegram_instance(id)
                .ok_or_else(|| anyhow::anyhow!("Telegram instance not found: {}", id))?
        } else {
            let instances = list_telegram_instances();
            instances.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!(
                    "No Telegram instance configured. Please add a Telegram instance first."
                )
            })?
        };

        // Parameter priority: parameter > instance config
        let bot_token = parameters
            .get("bot_token")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.bot_token.as_str());

        if bot_token.is_empty() {
            anyhow::bail!(
                "Telegram bot_token not configured for instance: {}",
                instance.name
            );
        }

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
                "Telegram message sent successfully to chat {} [instance: {}]",
                chat_id, instance.name
            ))
        } else {
            Err(anyhow::anyhow!(
                "Failed to send Telegram message: {}",
                response.body
            ))
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("chat_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: chat_id"))?;
        parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        Ok(())
    }
}
