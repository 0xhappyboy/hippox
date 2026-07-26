//! URL reading driver
//!
//! This driver provides functionality to fetch and read content from a URL.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, RequestConfig, execute,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading URLs
#[derive(Debug)]
pub struct ReadUrlDriver;
#[async_trait::async_trait]
impl Driver for ReadUrlDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "read_url";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Fetch and read content from a URL";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to fetch a webpage, API response, or any content from a URL";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "The URL to fetch content from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "method".to_string(),
                param_type: "string".to_string(),
                description: "HTTP method (GET, POST, PUT, DELETE)".to_string(),
                required: false,
                default: Some(Value::String("GET".to_string())),
                example: Some(Value::String("GET".to_string())),
                enum_values: Some(vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()]),
            },
            DriverParameter {
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
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds (default 30)".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "max_size".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum bytes to read (default 1MB)".to_string(),
                required: false,
                default: Some(Value::Number(1048576.into())),
                example: Some(Value::Number(102400.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "raw".to_string(),
                param_type: "boolean".to_string(),
                description: "Return raw content without formatting (default false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "read_url",
            "parameters": {
                "url": "https://api.github.com/repos/rust-lang/rust"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "HTTP 200:\n{\"full_name\": \"rust-lang/rust\", ...}".to_string();
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
        debug!("Executing read_url driver");
        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'url' parameter");
                return DriverError::missing_parameter("url");
            })?
            .to_string();
        let method = parameters.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
        let timeout = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
        let max_size = parameters.get("max_size").and_then(|v| v.as_u64()).unwrap_or(1024 * 1024) as usize;
        let raw = parameters.get("raw").and_then(|v| v.as_bool()).unwrap_or(false);
        let headers = parameters.get("headers").and_then(|v| v.as_object()).map(|obj| {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
            return map;
        });
        info!("Read URL: url={}, method={}, timeout={}s, max_size={}, raw={}", url, method, timeout, max_size, raw);
        let config = RequestConfig { url, method, headers, body: None, timeout_secs: Some(timeout) };
        let response = execute(&config).await.map_err(|e| {
            debug!("Failed to fetch URL: {}", e);
            return DriverError::execution(format!("Failed to fetch URL: {}", e));
        })?;
        info!("Read URL complete: status={}, body_size={}", response.status, response.body.len());
        let result = if raw {
            if response.body.len() > max_size {
                let truncated = format!("{}{}", &response.body[..max_size], "\n\n[Content truncated due to size limit]");
                info!("Content truncated: {} of {} bytes shown", max_size, response.body.len());
                truncated
            } else {
                response.body
            }
        } else {
            response.to_formatted_string()
        };
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating read_url parameters");
        parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'url' parameter");
            return DriverError::missing_parameter("url");
        })?;
        info!("read_url validation passed");
        return Ok(());
    }
}
