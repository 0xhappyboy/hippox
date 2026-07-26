//! HTTP request driver
//!
//! This driver provides functionality to send HTTP requests to web APIs.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, RequestConfig, execute,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
/// Parse configuration from parameters
pub fn parse_config(parameters: &HashMap<String, Value>) -> DriverResult<RequestConfig> {
    debug!("Parsing HTTP request configuration");
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
            return map;
        })
        .unwrap_or_default();
    let body = parameters.get("body").and_then(|v| v.as_str()).map(|s| s.to_string());
    info!("Parsed config: url={}, method={}, timeout={}s", url, method, timeout);
    return Ok(RequestConfig { url, method, headers: Some(headers), body, timeout_secs: Some(timeout) });
}
/// Driver for sending HTTP requests
#[derive(Debug)]
pub struct HttpRequestDriver;
#[async_trait::async_trait]
impl Driver for HttpRequestDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "http_request";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Send HTTP requests to web APIs";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to fetch data from an API, call a web service, or interact with HTTP endpoints";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "The complete URL to send the HTTP request to".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://api.github.com/repos/rust-lang/rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "method".to_string(),
                param_type: "string".to_string(),
                description: "HTTP method (GET, POST, PUT, DELETE, PATCH)".to_string(),
                required: false,
                default: Some(Value::String("GET".to_string())),
                example: Some(Value::String("POST".to_string())),
                enum_values: Some(vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string(), "PATCH".to_string()]),
            },
            DriverParameter {
                name: "headers".to_string(),
                param_type: "object".to_string(),
                description: "HTTP headers as key-value pairs".to_string(),
                required: false,
                default: None,
                example: Some(json!({
                    "Authorization": "Bearer token",
                    "Content-Type": "application/json"
                })),
                enum_values: None,
            },
            DriverParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Request body (for POST, PUT, PATCH)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(r#"{"name": "test"}"#.to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "http_request",
            "parameters": {
                "url": "https://api.github.com/repos/rust-lang/rust",
                "method": "GET"
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
        debug!("Executing http_request driver");
        let start_time = Instant::now();
        let config = parse_config(parameters)?;
        info!("HTTP request: url={}, method={}, timeout={}s", config.url, config.method, config.timeout_secs.unwrap_or(30));
        let response = execute(&config).await.map_err(|e| {
            debug!("HTTP request failed: {}", e);
            return DriverError::execution(format!("HTTP request failed: {}", e));
        })?;
        let duration = start_time.elapsed();
        info!("HTTP request completed: status={}, duration={}ms", response.status, duration.as_millis());
        let result = response.to_formatted_string();
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating http_request parameters");
        parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'url' parameter");
            return DriverError::missing_parameter("url");
        })?;
        info!("http_request validation passed");
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_parse_config() {
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));
        params.insert("method".to_string(), json!("POST"));
        params.insert("timeout".to_string(), json!(60));
        let config = parse_config(&params).unwrap();
        assert_eq!(config.url, "https://example.com");
        assert_eq!(config.method, "POST");
        assert_eq!(config.timeout_secs, Some(60));
    }
    #[test]
    fn test_parse_config_with_headers() {
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));
        let headers = json!({
            "Authorization": "Bearer token",
            "Content-Type": "application/json"
        });
        params.insert("headers".to_string(), headers);
        let config = parse_config(&params).unwrap();
        let headers = config.headers.unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
}
