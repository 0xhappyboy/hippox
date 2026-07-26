use crate::DriverError;
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
use crate::result::DriverResult;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{debug, info, warn};
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
        return Self { url: String::new(), method: "GET".to_string(), headers: None, body: None, timeout_secs: Some(30) };
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
    /// Format response as formatted string
    pub fn to_formatted_string(&self) -> String {
        if self.is_success {
            if let Ok(json) = serde_json::from_str::<Value>(&self.body) {
                return format!("HTTP {}:\n{}", self.status, serde_json::to_string_pretty(&json).unwrap_or(self.body.clone()));
            } else {
                return format!("HTTP {}:\n{}", self.status, self.body);
            }
        } else {
            return format!("HTTP Error {}: {}", self.status, self.body);
        }
    }
}
/// Execute HTTP request
pub async fn execute(config: &RequestConfig) -> DriverResult<Response> {
    debug!("Executing HTTP request: {} {}", config.method, config.url);
    let timeout = std::time::Duration::from_secs(config.timeout_secs.unwrap_or(30));
    let client = match Client::builder().timeout(timeout).build() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to build HTTP client: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let method = config.method.to_uppercase();
    let mut request_builder = match method.as_str() {
        "GET" => client.get(&config.url),
        "POST" => client.post(&config.url),
        "PUT" => client.put(&config.url),
        "DELETE" => client.delete(&config.url),
        "PATCH" => client.patch(&config.url),
        _ => {
            let err_msg = format!("Unsupported HTTP method: {}", method);
            warn!("{}", err_msg);
            return Err(DriverError::validation("method", err_msg));
        }
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
        debug!("Added {} headers to request", headers.len());
    }
    if let Some(body) = &config.body {
        request_builder = request_builder.body(body.clone());
        debug!("Added body to request: {} bytes", body.len());
    }
    let response = match request_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("HTTP request failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let status = response.status().as_u16();
    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            let err_msg = format!("Failed to read response body: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let is_success = status >= 200 && status < 300;
    info!("HTTP request completed: {} {} -> status={}", config.method, config.url, status);
    return Ok(Response { status, body, is_success });
}
/// Parse parameters from Skill parameters into RequestConfig
pub fn parse_config(parameters: &HashMap<String, Value>) -> DriverResult<RequestConfig> {
    debug!("Parsing HTTP config from parameters");
    let url = parameters
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            let err_msg = "Missing 'url' parameter".to_string();
            warn!("{}", err_msg);
            return DriverError::missing_parameter("url");
        })?
        .to_string();
    let method = parameters.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
    let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64());
    let headers = parameters.get("headers").and_then(|v| v.as_object()).map(|obj| {
        let mut map = HashMap::new();
        for (k, v) in obj {
            if let Some(val_str) = v.as_str() {
                map.insert(k.clone(), val_str.to_string());
            }
        }
        return map;
    });
    let body = parameters.get("body").map(|v| if v.is_string() { v.as_str().unwrap_or("").to_string() } else { v.to_string() });
    info!("Parsed config: url={}, method={}", url, method);
    return Ok(RequestConfig { url, method, headers, body, timeout_secs });
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
pub async fn http_download(config: &DownloadConfig) -> DriverResult<String> {
    debug!("Downloading file from: {} to {}", config.url, config.output_path);
    let timeout = std::time::Duration::from_secs(config.timeout_secs.unwrap_or(300));
    let client = match Client::builder().timeout(timeout).build() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to build HTTP client: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
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
    let response = match request_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Download request failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let path = Path::new(&config.output_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            match std::fs::create_dir_all(parent) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to create output directory: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
    }
    let mut file = match File::create(&config.output_path) {
        Ok(f) => f,
        Err(e) => {
            let err_msg = format!("Failed to create output file: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    let mut downloaded: u64 = 0;
    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to download chunk: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::execution(err_msg));
            }
        };
        match file.write_all(&chunk) {
            Ok(_) => {}
            Err(e) => {
                let err_msg = format!("Failed to write chunk to file: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
        downloaded += chunk.len() as u64;
    }
    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            let err_msg = format!("Failed to get file size: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    info!("Downloaded {} bytes to {}", file_size, config.output_path);
    return Ok(format!("Downloaded {} bytes to {}", file_size, config.output_path));
}
/// Upload a file via multipart/form-data
pub async fn http_upload(config: &UploadConfig) -> DriverResult<String> {
    debug!("Uploading file: {} to {}", config.file_path, config.url);
    use reqwest::multipart;
    let timeout = std::time::Duration::from_secs(config.timeout_secs.unwrap_or(300));
    let client = match Client::builder().timeout(timeout).build() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to build HTTP client: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let file_path = Path::new(&config.file_path);
    if !file_path.exists() {
        let err_msg = format!("File not found: {}", config.file_path);
        warn!("{}", err_msg);
        return Err(DriverError::validation("file_path", err_msg));
    }
    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let file_content = match std::fs::read(&config.file_path) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to read file: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
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
    let response = match request_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Upload request failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let status = response.status().as_u16();
    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            let err_msg = format!("Failed to read upload response: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    if status >= 200 && status < 300 {
        info!("Uploaded {} to {} (status: {})", config.file_path, config.url, status);
        return Ok(format!("Uploaded {} to {} (status: {})", config.file_path, config.url, status));
    } else {
        let err_msg = format!("Upload failed (status: {}): {}", status, body);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
// =========================== Web Hook ===========================
/// Send webhook (JSON POST)
pub async fn send_webhook(url: &str, payload: &Value, headers: Option<HashMap<String, String>>) -> DriverResult<String> {
    debug!("Sending webhook to: {}", url);
    let client = match Client::builder().timeout(std::time::Duration::from_secs(30)).build() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to build HTTP client: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
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
    let response = match request_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Webhook request failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let status = response.status().as_u16();
    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            let err_msg = format!("Failed to read webhook response: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    if status >= 200 && status < 300 {
        info!("Webhook sent successfully (status: {})", status);
        return Ok(format!("Webhook sent successfully (status: {})", status));
    } else {
        let err_msg = format!("Webhook failed (status: {}): {}", status, body);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
