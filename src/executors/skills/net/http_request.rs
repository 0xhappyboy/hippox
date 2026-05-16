use crate::executors::{
    skills::common::Http,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequestSkill;

#[async_trait::async_trait]
impl Skill for HttpRequestSkill {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Send HTTP requests to web APIs"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to fetch data from an API, call a web service, or interact with HTTP endpoints"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "The complete URL to send the HTTP request to".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "https://api.github.com/repos/rust-lang/rust".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "method".to_string(),
                param_type: "string".to_string(),
                description: "HTTP method (GET, POST, PUT, DELETE, PATCH)".to_string(),
                required: false,
                default: Some(Value::String("GET".to_string())),
                example: Some(Value::String("POST".to_string())),
                enum_values: Some(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "PATCH".to_string(),
                ]),
            },
            SkillParameter {
                name: "headers".to_string(),
                param_type: "object".to_string(),
                description: "HTTP headers as key-value pairs".to_string(),
                required: false,
                default: None,
                example: Some(
                    json!({"Authorization": "Bearer token", "Content-Type": "application/json"}),
                ),
                enum_values: None,
            },
            SkillParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Request body (for POST, PUT, PATCH)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(r#"{"name": "test"}"#.to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "http_request",
            "parameters": {
                "url": "https://api.github.com/repos/rust-lang/rust",
                "method": "GET"
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
        let config = Http::parse_config(parameters)?;
        let response = Http::execute(&config).await?;
        Ok(response.to_formatted_string())
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;
        Ok(())
    }
}
