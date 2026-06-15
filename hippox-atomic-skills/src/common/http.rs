/// HTTP common module
///
/// This module provides a reusable HTTP client that can be used by other skills.
///
/// # Examples
///
/// ## Parse from skill parameters
///
/// ```rust
/// use crate::executors::utils::Http;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// let mut params = HashMap::new();
/// params.insert("url".to_string(), json!("https://api.example.com/data"));
/// params.insert("method".to_string(), json!("GET"));
///
/// let config = Http::parse_config(&params)?;
/// let response = Http::execute(&config).await?;
/// println!("{}", response.to_formatted_string());
/// ```
///
/// ## Build config manually
///
/// ```rust
/// use crate::executors::utils::Http;
///
/// let config = Http::RequestConfig {
///     url: "https://api.weather.com/v1/current".to_string(),
///     method: "POST".to_string(),
///     headers: Some([
///         ("Authorization".to_string(), "Bearer token".to_string()),
///     ].into()),
///     body: Some(r#"{"city": "Beijing"}"#.to_string()),
///     timeout_secs: Some(10),
/// };
///
/// let response = Http::execute(&config).await?;
/// ```
use anyhow::Result;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "GET".to_string(),
            headers: None,
            body: None,
            timeout_secs: Some(30),
        }
    }
}

/// HTTP response result
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
    pub is_success: bool,
}

impl Response {
    pub fn to_formatted_string(&self) -> String {
        if self.is_success {
            if let Ok(json) = serde_json::from_str::<Value>(&self.body) {
                format!(
                    "HTTP {}:\n{}",
                    self.status,
                    serde_json::to_string_pretty(&json).unwrap_or(self.body.clone())
                )
            } else {
                format!("HTTP {}:\n{}", self.status, self.body)
            }
        } else {
            format!("HTTP Error {}: {}", self.status, self.body)
        }
    }
}

/// Execute HTTP request
pub async fn execute(config: &RequestConfig) -> Result<Response> {
    let timeout = std::time::Duration::from_secs(config.timeout_secs.unwrap_or(30));
    let client = Client::builder().timeout(timeout).build()?;
    let method = config.method.to_uppercase();
    let mut request_builder = match method.as_str() {
        "GET" => client.get(&config.url),
        "POST" => client.post(&config.url),
        "PUT" => client.put(&config.url),
        "DELETE" => client.delete(&config.url),
        "PATCH" => client.patch(&config.url),
        _ => anyhow::bail!("Unsupported HTTP method: {}", method),
    };
    if let Some(headers) = &config.headers {
        let mut header_map = HeaderMap::new();
        for (key, value) in headers {
            if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    header_map.insert(header_name, header_value);
                }
            }
        }
        request_builder = request_builder.headers(header_map);
    }
    if let Some(body) = &config.body {
        request_builder = request_builder.body(body.clone());
    }
    let response = request_builder.send().await?;
    let status = response.status().as_u16();
    let body = response.text().await?;
    let is_success = status >= 200 && status < 300;
    Ok(Response {
        status,
        body,
        is_success,
    })
}

/// Parse parameters from Skill parameters into RequestConfig
pub fn parse_config(parameters: &HashMap<String, Value>) -> Result<RequestConfig> {
    let url = parameters
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?
        .to_string();
    let method = parameters
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_string();
    let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64());
    let headers = parameters
        .get("headers")
        .and_then(|v| v.as_object())
        .map(|obj| {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let Some(val_str) = v.as_str() {
                    map.insert(k.clone(), val_str.to_string());
                }
            }
            map
        });
    let body = parameters.get("body").map(|v| {
        if v.is_string() {
            v.as_str().unwrap_or("").to_string()
        } else {
            v.to_string()
        }
    });
    Ok(RequestConfig {
        url,
        method,
        headers,
        body,
        timeout_secs,
    })
}
