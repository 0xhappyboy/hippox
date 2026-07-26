//! DingDing message driver
//!
//! This driver provides functionality to send messages via DingDing robot.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, RequestConfig, execute,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Helper function to get a string parameter
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    return params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name));
}
/// Helper function to get a bool parameter with default
fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    return params.get(name).and_then(|v| v.as_bool()).unwrap_or(default);
}
/// Helper function to get an array parameter
fn get_param_array(params: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    return params.get(name).and_then(|v| v.as_array()).cloned().unwrap_or_default();
}
/// Driver for sending DingDing messages
#[derive(Debug)]
pub struct SendDingDingDriver;
#[async_trait::async_trait]
impl Driver for SendDingDingDriver {
    fn name(&self) -> &str {
        return "send_dingding";
    }
    fn description(&self) -> &str {
        return "Send a message via DingDing robot";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to send a DingDing message, notify via DingDing, or send a message to a DingDing group";
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::SocialPlatform;
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "access_token".to_string(),
                param_type: "string".to_string(),
                description: "DingTalk access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("your_access_token".to_string())),
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
                required: true,
                default: None,
                example: Some(Value::String("Hello from Hippo!".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "msg_type".to_string(),
                param_type: "string".to_string(),
                description: "Message type: 'text' or 'markdown'".to_string(),
                required: false,
                default: Some(Value::String("text".to_string())),
                example: Some(Value::String("markdown".to_string())),
                enum_values: Some(vec!["text".to_string(), "markdown".to_string()]),
            },
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Title for markdown messages".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Notification".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "at_mobiles".to_string(),
                param_type: "array".to_string(),
                description: "Array of phone numbers to @ mention".to_string(),
                required: false,
                default: None,
                example: Some(json!(["13800000000"])),
                enum_values: None,
            },
            DriverParameter {
                name: "at_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to @ everyone".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "send_dingding",
            "parameters": {
                "access_token": "your_token",
                "text": "Hello from Hippo!"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "DingDing message sent successfully".to_string();
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing send_dingding driver");
        let access_token = get_param_string(parameters, "access_token")?;
        let text = get_param_string(parameters, "text")?;
        let msg_type = parameters.get("msg_type").and_then(|v| v.as_str()).unwrap_or("text");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let at_mobiles = get_param_array(parameters, "at_mobiles");
        let at_all = get_param_bool(parameters, "at_all", false);
        let secret = parameters.get("secret").and_then(|v| v.as_str());
        info!("DingDing send: access_token={}, msg_type={}, text_len={}", access_token, msg_type, text.len());
        // Build webhook URL with optional signature
        let webhook: String;
        if let Some(secret_val) = secret {
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
            let sign_str = format!("{}\n{}", timestamp, secret_val);
            let sign = format!("{:x}", md5::compute(sign_str.as_bytes()));
            webhook = format!("https://oapi.dingtalk.com/robot/send?access_token={}&timestamp={}&sign={}", access_token, timestamp, sign);
        } else {
            webhook = format!("https://oapi.dingtalk.com/robot/send?access_token={}", access_token);
        }
        let mut body = serde_json::Map::new();
        if msg_type == "markdown" {
            let markdown_title = title.ok_or_else(|| DriverError::execution("Missing 'title' parameter for markdown message"))?;
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
        let at_mobiles_strs: Vec<String> = at_mobiles.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
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
        let body_json = serde_json::to_string(&body).map_err(|e| DriverError::execution(format!("Failed to serialize body: {}", e)))?;
        let http_config = RequestConfig {
            url: webhook,
            method: "POST".to_string(),
            headers: Some([("Content-Type".to_string(), "application/json".to_string())].into_iter().collect()),
            body: Some(body_json),
            timeout_secs: Some(30),
        };
        let response = execute(&http_config).await?;
        if response.is_success {
            if let Ok(resp_json) = serde_json::from_str::<Value>(&response.body) {
                if let Some(errcode) = resp_json.get("errcode").and_then(|v| v.as_i64()) {
                    if errcode == 0 {
                        info!("DingDing message sent successfully");
                        return Ok("DingDing message sent successfully".to_string());
                    } else {
                        let errmsg = resp_json.get("errmsg").and_then(|v| v.as_str()).unwrap_or("unknown error");
                        debug!("DingDing API error: {} - {}", errcode, errmsg);
                        return Err(DriverError::execution(format!("DingDing API error: {} - {}", errcode, errmsg)));
                    }
                }
            }
            info!("DingDing message sent successfully");
            return Ok("DingDing message sent successfully".to_string());
        } else {
            debug!("Failed to send DingDing message: {}", response.body);
            return Err(DriverError::execution(format!("Failed to send DingDing message: {}", response.body)));
        }
    }
}
