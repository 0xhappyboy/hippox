use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    config::{get_dingtalk_instance, list_dingtalk_instances},
    executors::{
        RequestConfig, execute,
        types::{Skill, SkillParameter},
    },
};

#[derive(Debug)]
pub struct SendDingDingSkill;

#[async_trait::async_trait]
impl Skill for SendDingDingSkill {
    fn name(&self) -> &str {
        "send_dingding"
    }

    fn description(&self) -> &str {
        "Send a message via DingDing robot"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to send a DingDing message, notify via DingDing, or send a message to a DingDing group"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_dingtalk_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "DingTalk instance ID (use 'list_dingtalk_instances' to see available instances)".to_string(),
                required: false,
                default: if instance_ids.is_empty() { None } else { Some(Value::String(instance_ids[0].clone())) },
                example: Some(Value::String("dingtalk_prod".to_string())),
                enum_values: if instance_ids.is_empty() { None } else { Some(instance_ids) },
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
                name: "at_mobiles".to_string(),
                param_type: "array".to_string(),
                description: "Array of phone numbers to @ mention".to_string(),
                required: false,
                default: None,
                example: Some(json!(["13800000000", "13900000000"])),
                enum_values: None,
            },
            SkillParameter {
                name: "at_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to @ everyone in the group".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "msg_type".to_string(),
                param_type: "string".to_string(),
                description: "Message type: 'text' or 'markdown'".to_string(),
                required: false,
                default: Some(Value::String("text".to_string())),
                example: Some(Value::String("markdown".to_string())),
                enum_values: Some(vec!["text".to_string(), "markdown".to_string()]),
            },
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Title for markdown messages (required if msg_type is 'markdown')"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Notification".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "access_token".to_string(),
                param_type: "string".to_string(),
                description: "Access token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("your_access_token".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "secret".to_string(),
                param_type: "string".to_string(),
                description: "Secret for signature (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("your_secret".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "send_dingding",
            "parameters": {
                "instance_id": "dingtalk_prod",
                "text": "Hello from Hippo!"
            }
        })
    }

    fn example_output(&self) -> String {
        "DingDing message sent successfully [instance: dingtalk_prod]".to_string()
    }

    fn category(&self) -> &str {
        "messaging"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'text' parameter"))?;

        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());

        // Get instance configuration
        let instance = if let Some(id) = instance_id {
            get_dingtalk_instance(id)
                .ok_or_else(|| anyhow::anyhow!("DingTalk instance not found: {}", id))?
        } else {
            let instances = list_dingtalk_instances();
            instances.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!(
                    "No DingTalk instance configured. Please add a DingTalk instance first."
                )
            })?
        };

        // Parameter priority: parameter > instance config
        let access_token = parameters
            .get("access_token")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.access_token.as_str());

        if access_token.is_empty() {
            anyhow::bail!(
                "DingTalk access_token not configured for instance: {}",
                instance.name
            );
        }

        // Build webhook URL with optional signature
        let webhook = if let Some(secret) = parameters.get("secret").and_then(|v| v.as_str()) {
            // Use provided secret
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let sign_str = format!("{}\n{}", timestamp, secret);
            let sign = format!("{:?}", md5::compute(sign_str.as_bytes()));
            format!(
                "https://oapi.dingtalk.com/robot/send?access_token={}&timestamp={}&sign={}",
                access_token, timestamp, sign
            )
        } else if let Some(secret) = &instance.secret {
            // Use instance secret
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let sign_str = format!("{}\n{}", timestamp, secret);
            let sign = format!("{:?}", md5::compute(sign_str.as_bytes()));
            format!(
                "https://oapi.dingtalk.com/robot/send?access_token={}&timestamp={}&sign={}",
                access_token, timestamp, sign
            )
        } else {
            format!(
                "https://oapi.dingtalk.com/robot/send?access_token={}",
                access_token
            )
        };

        let msg_type = parameters
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let at_mobiles = parameters
            .get("at_mobiles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let at_all = parameters
            .get("at_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let title = parameters.get("title").and_then(|v| v.as_str());

        let mut body = serde_json::Map::new();
        if msg_type == "markdown" {
            let markdown_title = title
                .ok_or_else(|| anyhow::anyhow!("Missing 'title' parameter for markdown message"))?;
            body.insert("msgtype".to_string(), json!("markdown"));
            body.insert(
                "markdown".to_string(),
                json!({
                    "title": markdown_title,
                    "text": text
                }),
            );
        } else {
            body.insert("msgtype".to_string(), json!("text"));
            body.insert(
                "text".to_string(),
                json!({
                    "content": text
                }),
            );
        }

        let mut at = serde_json::Map::new();
        if !at_mobiles.is_empty() {
            at.insert("atMobiles".to_string(), json!(at_mobiles));
        }
        if at_all {
            at.insert("isAtAll".to_string(), json!(true));
        }
        if !at.is_empty() {
            body.insert("at".to_string(), Value::Object(at));
        }

        let http_config = RequestConfig {
            url: webhook,
            method: "POST".to_string(),
            headers: Some([("Content-Type".to_string(), "application/json".to_string())].into()),
            body: Some(serde_json::to_string(&body)?),
            timeout_secs: Some(30),
        };

        let response = execute(&http_config).await?;
        if response.is_success {
            if let Ok(resp_json) = serde_json::from_str::<Value>(&response.body) {
                if let Some(errcode) = resp_json.get("errcode").and_then(|v| v.as_i64()) {
                    if errcode == 0 {
                        return Ok(format!(
                            "DingDing message sent successfully [instance: {}]",
                            instance.name
                        ));
                    } else {
                        let errmsg = resp_json
                            .get("errmsg")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown error");
                        return Err(anyhow::anyhow!(
                            "DingDing API error: {} - {}",
                            errcode,
                            errmsg
                        ));
                    }
                }
            }
            Ok(format!(
                "DingDing message sent successfully [instance: {}]",
                instance.name
            ))
        } else {
            Err(anyhow::anyhow!(
                "Failed to send DingDing message: {}",
                response.body
            ))
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let msg_type = parameters
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        if msg_type == "markdown" {
            parameters
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!("Missing required parameter: title for markdown message")
                })?;
        }
        Ok(())
    }
}
