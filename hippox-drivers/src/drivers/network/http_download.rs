//! HTTP download driver
//!
//! This driver provides functionality to download a file from an HTTP URL
//! and save it to local disk.
use crate::common::http::{DownloadConfig, http_download};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for downloading files via HTTP
#[derive(Debug)]
pub struct HttpDownloadDriver;
#[async_trait::async_trait]
impl Driver for HttpDownloadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "http_download"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Download a file from an HTTP URL and save it to local disk"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to download a file from a URL to the local filesystem"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "http_download",
            "parameters": {
                "url": "https://example.com/file.zip",
                "output_path": "/tmp/file.zip"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Downloaded 1048576 bytes to /tmp/file.zip".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing http_download driver");
        let url = parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("url"))?;
        let output_path = parameters.get("output_path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("output_path"))?;
        let timeout = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(300);
        let headers = parameters.get("headers").and_then(|v| v.as_object()).map(|obj| {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
            map
        });
        info!("HTTP download: url={}, output={}, timeout={}s", url, output_path, timeout);
        let config = DownloadConfig { url: url.to_string(), output_path: output_path.to_string(), headers, timeout_secs: Some(timeout) };
        let result = http_download(&config).await.map_err(|e| DriverError::execution(format!("Download failed: {}", e)))?;
        info!("HTTP download completed: {}", result);
        return Ok(result);
    }
}
