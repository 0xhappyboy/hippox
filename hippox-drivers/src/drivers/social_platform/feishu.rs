//! Feishu (Lark) message driver
//!
//! This driver provides functionality to send messages via Feishu bot.
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
/// Helper function to get an array parameter
fn get_param_array(params: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    params.get(name).and_then(|v| v.as_array()).cloned().unwrap_or_default()
}
/// Driver for sending Feishu messages
#[derive(Debug)]
pub struct SendFeishuDriver;
#[async_trait::async_trait]
impl Driver for SendFeishuDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "send_feishu"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a message via Feishu (Lark) bot"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to send a Feishu message"
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::SocialPlatform
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "webhook".to_string(),
                param_type: "string".to_string(),
                description: "Feishu webhook URL".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://open.feishu.cn/open-apis/bot/v2/hook/xxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "secret".to_string(),
                param_type: "string".to_string(),
                description: "Secret for signature (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("your_secret".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Message text to send".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Hello from Hippo!".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "msg_type".to_string(),
                param_type: "string".to_string(),
                description: "Message type: 'text', 'post', or 'image'".to_string(),
                required: false,
                default: Some(Value::String("text".to_string())),
                example: Some(Value::String("post".to_string())),
                enum_values: Some(vec!["text".to_string(), "post".to_string(), "image".to_string()]),
            },
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Title for post messages".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Notification".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "content".to_string(),
                param_type: "array".to_string(),
                description: "Rich content for post messages".to_string(),
                required: false,
                default: None,
                example: Some(json!([[{"tag": "text", "text": "Hello"}]])),
                enum_values: None,
            },
            DriverParameter {
                name: "image_key".to_string(),
                param_type: "string".to_string(),
                description: "Image key for image messages".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("img_v2_xxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "at_mobiles".to_string(),
                param_type: "array".to_string(),
                description: "User IDs to @ mention".to_string(),
                required: false,
                default: None,
                example: Some(json!(["ou_xxx"])),
                enum_values: None,
            },
            DriverParameter {
                name: "at_all".to_string(),
                param_type: "boolean".to_string(),
                description: "@ everyone".to_string(),
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
            "action": "send_feishu",
            "parameters": {
                "webhook": "https://open.feishu.cn/xxx",
                "text": "Hello"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Feishu message sent successfully".to_string()
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing send_feishu driver");
        let webhook = get_param_string(parameters, "webhook")?;
        let text = parameters.get("text").and_then(|v| v.as_str());
        let msg_type = parameters.get("msg_type").and_then(|v| v.as_str()).unwrap_or("text");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let content = parameters.get("content");
        let image_key = parameters.get("image_key").and_then(|v| v.as_str());
        let at_mobiles = get_param_array(parameters, "at_mobiles");
        let at_all = get_param_bool(parameters, "at_all", false);
        let secret = parameters.get("secret").and_then(|v| v.as_str());
        info!("Feishu send: webhook={}, msg_type={}, text_len={:?}", webhook, msg_type, text.map(|s| s.len()));
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut body = serde_json::Map::new();
        if let Some(secret_val) = secret {
            body.insert("timestamp".to_string(), json!(timestamp.to_string()));
            let sign_str = format!("{}\n{}", timestamp, secret_val);
            let sign = format!("{:x}", md5::compute(sign_str.as_bytes()));
            body.insert("sign".to_string(), json!(sign));
        }
        match msg_type {
            "post" => {
                let post_title = title.unwrap_or("Notification");
                let post_content = if let Some(c) = content {
                    c.clone()
                } else if let Some(t) = text {
                    json!([[{"tag": "text", "text": t}]])
                } else {
                    json!([[{"tag": "text", "text": ""}]])
                };
                body.insert("msg_type".to_string(), json!("post"));
                body.insert("content".to_string(), json!({ "post": { "zh_cn": { "title": post_title, "content": post_content } } }));
            }
            "image" => {
                let img_key = image_key.ok_or_else(|| DriverError::execution("Missing 'image_key' for image message"))?;
                body.insert("msg_type".to_string(), json!("image"));
                body.insert("content".to_string(), json!({ "image_key": img_key }));
            }
            _ => {
                let msg_text = text.unwrap_or("");
                body.insert("msg_type".to_string(), json!("text"));
                body.insert("content".to_string(), json!({ "text": msg_text }));
            }
        }
        let at_mobiles_strs: Vec<String> = at_mobiles.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
        if !at_mobiles_strs.is_empty() || at_all {
            let mut at = serde_json::Map::new();
            if !at_mobiles_strs.is_empty() {
                at.insert("atMobiles".to_string(), json!(at_mobiles_strs));
            }
            if at_all {
                at.insert("isAtAll".to_string(), json!(true));
            }
            body.insert("at".to_string(), Value::Object(at));
        }
        let http_config = RequestConfig {
            url: webhook,
            method: "POST".to_string(),
            headers: Some([("Content-Type".to_string(), "application/json".to_string())].into()),
            body: Some(serde_json::to_string(&body).map_err(|e| DriverError::execution(format!("Failed to serialize body: {}", e)))?),
            timeout_secs: Some(30),
        };
        let response = execute(&http_config).await?;
        if response.is_success {
            if let Ok(resp_json) = serde_json::from_str::<Value>(&response.body) {
                if let Some(code) = resp_json.get("code").and_then(|v| v.as_i64()) {
                    if code == 0 {
                        info!("Feishu message sent successfully");
                        return Ok("Feishu message sent successfully".to_string());
                    } else {
                        let msg = resp_json.get("msg").and_then(|v| v.as_str()).unwrap_or("unknown error");
                        return Err(DriverError::execution(format!("Feishu API error: {} - {}", code, msg)));
                    }
                }
            }
            info!("Feishu message sent successfully");
            Ok("Feishu message sent successfully".to_string())
        } else {
            Err(DriverError::execution(format!("Failed to send Feishu message: {}", response.body)))
        }
    }
}
