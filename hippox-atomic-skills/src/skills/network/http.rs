use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory, execute, parse_config,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Instant;

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

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let start_time = Instant::now();
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name.clone());
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting HTTP request".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }
        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some(format!("URL: {}", url)));
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        let method = parameters
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Method: {}", method)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let headers = parameters
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                let mut map = std::collections::HashMap::new();
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        map.insert(k.clone(), s.to_string());
                    }
                }
                map
            });
        if let Some(cb) = cb {
            if let Some(h) = &headers {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Headers: {:?}", h)),
                );
            }
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        let body = parameters
            .get("body")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(cb) = cb {
            if let Some(b) = &body {
                let truncated = if b.len() > 200 {
                    format!("{}...", &b[..200])
                } else {
                    b.clone()
                };
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Body: {}", truncated)),
                );
            }
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }
        let timeout = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Timeout: {}s", timeout)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Sending request...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
        }
        let config = parse_config(parameters)?;
        let response = execute(&config).await?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Response status: {}", response.status)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(90), None);
        }
        let result = response.to_formatted_string();
        if let Some(cb) = cb {
            let duration = start_time.elapsed().as_millis() as u64;
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Completed in {}ms", duration)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                step_name,
                Some(result.clone()),
            );
        }
        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;
        Ok(())
    }
}
