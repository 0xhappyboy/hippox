//! WeCom (Enterprise WeChat) message driver
//!
//! This driver provides functionality to send messages via WeCom robot.
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
/// Driver for sending WeCom messages
#[derive(Debug)]
pub struct SendWecomDriver;
#[async_trait::async_trait]
impl Driver for SendWecomDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "send_wecom";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Send a message via WeCom (Enterprise WeChat) robot";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to send a WeCom message";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::SocialPlatform;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "webhook".to_string(),
                param_type: "string".to_string(),
                description: "WeCom webhook URL".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=xxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Webhook key (alternative to webhook URL)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("your-key".to_string())),
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
                description: "Message type: 'text', 'markdown', 'image', 'news'".to_string(),
                required: false,
                default: Some(Value::String("text".to_string())),
                example: Some(Value::String("markdown".to_string())),
                enum_values: Some(vec!["text".to_string(), "markdown".to_string(), "image".to_string(), "news".to_string()]),
            },
            DriverParameter {
                name: "mentioned_list".to_string(),
                param_type: "array".to_string(),
                description: "User IDs to @ mention".to_string(),
                required: false,
                default: None,
                example: Some(json!(["zhangsan"])),
                enum_values: None,
            },
            DriverParameter {
                name: "mentioned_mobile_list".to_string(),
                param_type: "array".to_string(),
                description: "Mobile numbers to @ mention".to_string(),
                required: false,
                default: None,
                example: Some(json!(["13800000000"])),
                enum_values: None,
            },
            DriverParameter {
                name: "image_base64".to_string(),
                param_type: "string".to_string(),
                description: "Base64 encoded image".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/9j/4AAQSkZJRg...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "image_md5".to_string(),
                param_type: "string".to_string(),
                description: "MD5 hash of the image".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("a1b2c3d4...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "news_articles".to_string(),
                param_type: "array".to_string(),
                description: "Articles for news message".to_string(),
                required: false,
                default: None,
                example: Some(json!([{"title": "News", "url": "https://example.com"}])),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "send_wecom",
            "parameters": {
                "key": "your-key",
                "text": "Hello"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "WeCom message sent successfully".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing send_wecom driver");
        let text = get_param_string(parameters, "text")?;
        let msg_type = parameters.get("msg_type").and_then(|v| v.as_str()).unwrap_or("text");
        let mentioned_list = get_param_array(parameters, "mentioned_list");
        let mentioned_mobile_list = get_param_array(parameters, "mentioned_mobile_list");
        // Build webhook URL
        let webhook: String;
        if let Some(webhook_url) = parameters.get("webhook").and_then(|v| v.as_str()) {
            webhook = webhook_url.to_string();
        } else if let Some(key) = parameters.get("key").and_then(|v| v.as_str()) {
            webhook = format!("https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key={}", key);
        } else {
            debug!("Missing webhook or key parameter");
            return Err(DriverError::execution("Missing webhook or key parameter"));
        }
        info!("WeCom send: webhook={}, msg_type={}, text_len={}", webhook, msg_type, text.len());
        let mut body = serde_json::Map::new();
        match msg_type {
            "markdown" => {
                body.insert("msgtype".to_string(), json!("markdown"));
                body.insert(
                    "markdown".to_string(),
                    json!({
                        "content": text
                    }),
                );
            }
            "image" => {
                let image_base64 = parameters.get("image_base64").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing image_base64 parameter");
                    return DriverError::execution("Missing image_base64");
                })?;
                let image_md5 = parameters.get("image_md5").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing image_md5 parameter");
                    return DriverError::execution("Missing image_md5");
                })?;
                body.insert("msgtype".to_string(), json!("image"));
                body.insert(
                    "image".to_string(),
                    json!({
                        "base64": image_base64,
                        "md5": image_md5
                    }),
                );
            }
            "news" => {
                let articles = parameters.get("news_articles").and_then(|v| v.as_array()).ok_or_else(|| {
                    debug!("Missing news_articles parameter");
                    return DriverError::execution("Missing news_articles");
                })?;
                body.insert("msgtype".to_string(), json!("news"));
                body.insert(
                    "news".to_string(),
                    json!({
                        "articles": articles
                    }),
                );
            }
            _ => {
                body.insert("msgtype".to_string(), json!("text"));
                let mut text_content = json!({
                    "content": text
                });
                if !mentioned_list.is_empty() {
                    let list: Vec<String> = mentioned_list.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
                    text_content["mentioned_list"] = json!(list);
                }
                if !mentioned_mobile_list.is_empty() {
                    let list: Vec<String> = mentioned_mobile_list.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
                    text_content["mentioned_mobile_list"] = json!(list);
                }
                body.insert("text".to_string(), text_content);
            }
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
                        info!("WeCom message sent successfully");
                        return Ok("WeCom message sent successfully".to_string());
                    } else {
                        let errmsg = resp_json.get("errmsg").and_then(|v| v.as_str()).unwrap_or("unknown error");
                        debug!("WeCom API error: {} - {}", errcode, errmsg);
                        return Err(DriverError::execution(format!("WeCom API error: {} - {}", errcode, errmsg)));
                    }
                }
            }
            info!("WeCom message sent successfully");
            return Ok("WeCom message sent successfully".to_string());
        } else {
            debug!("Failed to send WeCom message: {}", response.body);
            return Err(DriverError::execution(format!("Failed to send WeCom message: {}", response.body)));
        }
    }
}
