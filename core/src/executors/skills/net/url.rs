use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    execute, parse_config,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct ReadUrlSkill;

#[async_trait::async_trait]
impl Skill for ReadUrlSkill {
    fn name(&self) -> &str {
        "read_url"
    }

    fn description(&self) -> &str {
        "Fetch and read content from a URL"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to fetch a webpage, API response, or any content from a URL"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "The URL to fetch content from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "method".to_string(),
                param_type: "string".to_string(),
                description: "HTTP method (GET, POST, PUT, DELETE)".to_string(),
                required: false,
                default: Some(Value::String("GET".to_string())),
                example: Some(Value::String("GET".to_string())),
                enum_values: Some(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                ]),
            },
            SkillParameter {
                name: "headers".to_string(),
                param_type: "object".to_string(),
                description: "HTTP headers as key-value pairs".to_string(),
                required: false,
                default: None,
                example: Some(json!({
                    "User-Agent": "Mozilla/5.0",
                    "Accept": "application/json"
                })),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds (default 30)".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_size".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum bytes to read (default 1MB)".to_string(),
                required: false,
                default: Some(Value::Number(1048576.into())),
                example: Some(Value::Number(102400.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "raw".to_string(),
                param_type: "boolean".to_string(),
                description: "Return raw content without formatting (default false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "read_url",
            "parameters": {
                "url": "https://api.github.com/repos/rust-lang/rust"
            }
        })
    }

    fn example_output(&self) -> String {
        "HTTP 200:\n{\"full_name\": \"rust-lang/rust\", ...}".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let raw = parameters
            .get("raw")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_size = parameters
            .get("max_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024 * 1024) as usize;
        let config = parse_config(parameters)?;
        let response = execute(&config).await?;
        if raw {
            if response.body.len() > max_size {
                Ok(format!(
                    "{}{}",
                    &response.body[..max_size],
                    "\n\n[Content truncated due to size limit]"
                ))
            } else {
                Ok(response.body)
            }
        } else {
            Ok(response.to_formatted_string())
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;
        Ok(())
    }
}
