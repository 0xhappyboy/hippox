use crate::common::http::{DownloadConfig, http_download};
use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverCategory, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpDownloadDriver;

#[async_trait::async_trait]
impl Driver for HttpDownloadDriver {
    fn name(&self) -> &str {
        "http_download"
    }

    fn description(&self) -> &str {
        "Download a file from an HTTP URL and save it to local disk"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to download a file from a URL to the local filesystem"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "URL of the file to download".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://example.com/file.zip".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "output_path".to_string(),
                param_type: "string".to_string(),
                description: "Local path to save the downloaded file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/file.zip".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "headers".to_string(),
                param_type: "object".to_string(),
                description: "HTTP headers as key-value pairs".to_string(),
                required: false,
                default: None,
                example: Some(json!({"Authorization": "Bearer token"})),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Download timeout in seconds (default: 300)".to_string(),
                required: false,
                default: Some(Value::Number(300.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "http_download",
            "parameters": {
                "url": "https://example.com/file.zip",
                "output_path": "/tmp/file.zip"
            }
        })
    }

    fn example_output(&self) -> String {
        "Downloaded 1048576 bytes to /tmp/file.zip".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting HTTP download".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(5), None);
        }

        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;

        let output_path = parameters
            .get("output_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output_path' parameter"))?;

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
            cb.on_log(task_id.clone(), driver_index, Some(format!("URL: {}", url)));
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Output: {}", output_path)),
            );
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Timeout: {}s", timeout)),
            );
            if let Some(h) = &headers {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("Headers: {:?}", h)),
                );
            }
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }

        let config = DownloadConfig {
            url: url.to_string(),
            output_path: output_path.to_string(),
            headers,
            timeout_secs: Some(timeout),
        };

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Downloading...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let result = http_download(&config).await?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Download completed".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("http_download".to_string()),
                Some(result.clone()),
            );
        }
        Ok(result)
    }
}
