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
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

/// HTTP download configuration
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub url: String,
    pub output_path: String,
    pub headers: Option<HashMap<String, String>>,
    pub timeout_secs: Option<u64>,
}
/// HTTP upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub url: String,
    pub file_path: String,
    pub field_name: String,
    pub headers: Option<HashMap<String, String>>,
    pub timeout_secs: Option<u64>,
}

/// Download a file from HTTP URL
pub async fn http_download(config: &DownloadConfig) -> Result<String> {
    let timeout = std::time::Duration::from_secs(config.timeout_secs.unwrap_or(300));
    let client = Client::builder().timeout(timeout).build()?;

    let mut request_builder = client.get(&config.url);
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
    let response = request_builder.send().await?;
    let path = Path::new(&config.output_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut file = File::create(&config.output_path)?;
    let mut downloaded: u64 = 0;

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
    }

    let file_size = file.metadata()?.len();
    Ok(format!(
        "Downloaded {} bytes to {}",
        file_size, config.output_path
    ))
}

/// Upload a file via multipart/form-data
pub async fn http_upload(config: &UploadConfig) -> Result<String> {
    use reqwest::multipart;

    let timeout = std::time::Duration::from_secs(config.timeout_secs.unwrap_or(300));
    let client = Client::builder().timeout(timeout).build()?;

    let file_path = Path::new(&config.file_path);
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", config.file_path);
    }

    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let file_content = std::fs::read(&config.file_path)?;
    let part = multipart::Part::bytes(file_content).file_name(file_name);

    let form = multipart::Form::new().part(config.field_name.clone(), part);

    let mut request_builder = client.post(&config.url).multipart(form);
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

    let response = request_builder.send().await?;
    let status = response.status().as_u16();
    let body = response.text().await?;

    if status >= 200 && status < 300 {
        Ok(format!(
            "Uploaded {} to {} (status: {})",
            config.file_path, config.url, status
        ))
    } else {
        anyhow::bail!("Upload failed (status: {}): {}", status, body)
    }
}

// =========================== Web Hook ===========================

/// Send webhook (JSON POST)
pub async fn send_webhook(
    url: &str,
    payload: &Value,
    headers: Option<HashMap<String, String>>,
) -> Result<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut request_builder = client.post(url).json(payload);
    if let Some(headers) = &headers {
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

    let response = request_builder.send().await?;
    let status = response.status().as_u16();
    let body = response.text().await?;

    if status >= 200 && status < 300 {
        Ok(format!("Webhook sent successfully (status: {})", status))
    } else {
        anyhow::bail!("Webhook failed (status: {}): {}", status, body)
    }
}
