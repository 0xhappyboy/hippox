use crate::common::http::{http_upload, UploadConfig};
use crate::types::{Skill, SkillParameter};
use crate::{SkillCallback, SkillCategory, SkillContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpUploadSkill;

#[async_trait::async_trait]
impl Skill for HttpUploadSkill {
    fn name(&self) -> &str {
        "http_upload"
    }

    fn description(&self) -> &str {
        "Upload a file to an HTTP server using multipart/form-data"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to upload a file to a server via HTTP multipart form"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "Upload endpoint URL".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://example.com/upload".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "file_path".to_string(),
                param_type: "string".to_string(),
                description: "Local file path to upload".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/file.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "field_name".to_string(),
                param_type: "string".to_string(),
                description: "Form field name for the file (default: file)".to_string(),
                required: false,
                default: Some(Value::String("file".to_string())),
                example: Some(Value::String("document".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "headers".to_string(),
                param_type: "object".to_string(),
                description: "HTTP headers as key-value pairs".to_string(),
                required: false,
                default: None,
                example: Some(json!({"Authorization": "Bearer token"})),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Upload timeout in seconds (default: 300)".to_string(),
                required: false,
                default: Some(Value::Number(300.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "http_upload",
            "parameters": {
                "url": "https://example.com/upload",
                "file_path": "/tmp/file.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "Uploaded /tmp/file.txt to https://example.com/upload (status: 200)".to_string()
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
            cb.on_log(task_id.clone(), skill_index, Some("Starting HTTP upload".to_string()));
            cb.on_progress(task_id.clone(), skill_index, Some(5), None);
        }
        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
        let file_path = parameters
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'file_path' parameter"))?;
        let field_name = parameters
            .get("field_name")
            .and_then(|v| v.as_str())
            .unwrap_or("file")
            .to_string();
        let timeout = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(300);
        let headers = parameters
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        map.insert(k.clone(), s.to_string());
                    }
                }
                map
            });
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some(format!("URL: {}", url)));
            cb.on_log(task_id.clone(), skill_index, Some(format!("File: {}", file_path)));
            cb.on_log(task_id.clone(), skill_index, Some(format!("Field: {}", field_name)));
            cb.on_log(task_id.clone(), skill_index, Some(format!("Timeout: {}s", timeout)));
            if let Some(h) = &headers {
                cb.on_log(task_id.clone(), skill_index, Some(format!("Headers: {:?}", h)));
            }
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        let config = UploadConfig {
            url: url.to_string(),
            file_path: file_path.to_string(),
            field_name,
            headers,
            timeout_secs: Some(timeout),
        };
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some("Uploading...".to_string()));
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        let result = http_upload(&config).await?;
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some("Upload completed".to_string()));
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(task_id.clone(), skill_index, Some("http_upload".to_string()), Some(result.clone()));
        }
        Ok(result)
    }
}