//! HTTP upload driver
//!
//! This driver provides functionality to upload a file to an HTTP server
//! using multipart/form-data.
use crate::common::http::{UploadConfig, http_upload};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for uploading files via HTTP
#[derive(Debug)]
pub struct HttpUploadDriver;
#[async_trait::async_trait]
impl Driver for HttpUploadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "http_upload"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Upload a file to an HTTP server using multipart/form-data"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to upload a file to a server via HTTP multipart form"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "Upload endpoint URL".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://example.com/upload".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "file_path".to_string(),
                param_type: "string".to_string(),
                description: "Local file path to upload".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/file.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "field_name".to_string(),
                param_type: "string".to_string(),
                description: "Form field name for the file (default: file)".to_string(),
                required: false,
                default: Some(Value::String("file".to_string())),
                example: Some(Value::String("document".to_string())),
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
                description: "Upload timeout in seconds (default: 300)".to_string(),
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
            "action": "http_upload",
            "parameters": {
                "url": "https://example.com/upload",
                "file_path": "/tmp/file.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Uploaded /tmp/file.txt to https://example.com/upload (status: 200)".to_string();
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
        debug!("Executing http_upload driver");
        let url = parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("url"))?;
        let file_path = parameters.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("file_path"))?;
        let field_name = parameters.get("field_name").and_then(|v| v.as_str()).unwrap_or("file").to_string();
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
        info!("HTTP upload: url={}, file={}, field={}, timeout={}s", url, file_path, field_name, timeout);
        let config = UploadConfig { url: url.to_string(), file_path: file_path.to_string(), field_name, headers, timeout_secs: Some(timeout) };
        let result = http_upload(&config).await.map_err(|e| DriverError::execution(format!("Upload failed: {}", e)))?;
        info!("HTTP upload completed: {}", result);
        return Ok(result);
    }
}
