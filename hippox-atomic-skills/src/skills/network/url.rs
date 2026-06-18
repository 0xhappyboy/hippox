use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory, execute, parse_config,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

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

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting URL fetch".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(5), None);
        }
        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Fetching URL: {}", url)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }
        let method = parameters
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("HTTP Method: {}", method)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(15), None);
        }
        let timeout = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Timeout: {} seconds", timeout)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        let max_size = parameters
            .get("max_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024 * 1024) as usize;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Max size: {} bytes", max_size)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(25), None);
        }
        let raw = parameters
            .get("raw")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Raw output: {}", raw)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let headers = parameters
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>()
            });
        if let Some(cb) = cb {
            if let Some(h) = &headers {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Headers: {:?}", h)),
                );
            } else {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some("No custom headers".to_string()),
                );
            }
            cb.on_progress(task_id.clone(), skill_index, Some(35), None);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Sending HTTP request...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }
        let config = parse_config(parameters)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Waiting for response...".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(65), None);
        }
        let response = execute(&config).await?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Response received (status: {}, size: {} bytes)",
                    response.status,
                    response.body.len()
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(80), None);
        }
        let result = if raw {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some("Processing raw content...".to_string()),
                );
                cb.on_progress(task_id.clone(), skill_index, Some(90), None);
            }
            if response.body.len() > max_size {
                let truncated = format!(
                    "{}{}",
                    &response.body[..max_size],
                    "\n\n[Content truncated due to size limit]"
                );
                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        skill_index,
                        Some(format!(
                            "Content truncated: {} of {} bytes shown",
                            max_size,
                            response.body.len()
                        )),
                    );
                }
                truncated
            } else {
                response.body
            }
        } else {
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some("Formatting response...".to_string()),
                );
                cb.on_progress(task_id.clone(), skill_index, Some(90), None);
            }
            response.to_formatted_string()
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Final result size: {} characters", result.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("read_url".to_string()),
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
