use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use crate::{RequestConfig, SkillCategory, execute};

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

fn get_param_array(params: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    params
        .get(name)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

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

    fn category(&self) -> SkillCategory {
        SkillCategory::SocialPlatform
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "access_token".to_string(),
                param_type: "string".to_string(),
                description: "DingTalk access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("your_access_token".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "secret".to_string(),
                param_type: "string".to_string(),
                description: "Secret for signature (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("your_secret".to_string())),
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
                description: "Title for markdown messages".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Notification".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "at_mobiles".to_string(),
                param_type: "array".to_string(),
                description: "Array of phone numbers to @ mention".to_string(),
                required: false,
                default: None,
                example: Some(json!(["13800000000"])),
                enum_values: None,
            },
            SkillParameter {
                name: "at_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to @ everyone".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "send_dingding", "parameters": { "access_token": "your_token", "text": "Hello from Hippo!" } })
    }

    fn example_output(&self) -> String {
        "DingDing message sent successfully".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let access_token = get_param_string(parameters, "access_token")?;
        let text = get_param_string(parameters, "text")?;
        let msg_type = parameters
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let at_mobiles = get_param_array(parameters, "at_mobiles");
        let at_all = get_param_bool(parameters, "at_all", false);
        let secret = parameters.get("secret").and_then(|v| v.as_str());

        // Build webhook URL with optional signature
        let webhook = if let Some(secret_val) = secret {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let sign_str = format!("{}\n{}", timestamp, secret_val);
            let sign = format!("{:x}", md5::compute(sign_str.as_bytes()));
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

        let mut body = serde_json::Map::new();
        if msg_type == "markdown" {
            let markdown_title = title
                .ok_or_else(|| anyhow::anyhow!("Missing 'title' parameter for markdown message"))?;
            body.insert("msgtype".to_string(), json!("markdown"));
            body.insert(
                "markdown".to_string(),
                json!({ "title": markdown_title, "text": text }),
            );
        } else {
            body.insert("msgtype".to_string(), json!("text"));
            body.insert("text".to_string(), json!({ "content": text }));
        }

        let at_mobiles_strs: Vec<String> = at_mobiles
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        let mut at = serde_json::Map::new();
        if !at_mobiles_strs.is_empty() {
            at.insert("atMobiles".to_string(), json!(at_mobiles_strs));
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
                        return Ok("DingDing message sent successfully".to_string());
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
            Ok("DingDing message sent successfully".to_string())
        } else {
            Err(anyhow::anyhow!(
                "Failed to send DingDing message: {}",
                response.body
            ))
        }
    }
}
