//! TCP send driver
//!
//! This driver provides functionality to connect to a TCP server, send data once,
//! optionally read response once, then close.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info};
/// Helper function to get a string parameter
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name))
}
/// Helper function to get a u64 parameter with default
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
/// Helper function to get a bool parameter with default
fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    params.get(name).and_then(|v| v.as_bool()).unwrap_or(default)
}
/// TCP Send Driver - single-shot sender
#[derive(Debug)]
pub struct TcpSendDriver;
#[async_trait::async_trait]
impl Driver for TcpSendDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "tcp_send"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Connect to a TCP server, send data ONCE, optionally read response ONCE, then close."
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Single-shot TCP sender. Connects, sends data, optionally waits for one response, then closes. For multiple exchanges, call this skill repeatedly."
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Target port number".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(8080.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, Server!".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Data encoding (utf8, hex, base64)".to_string(),
                required: false,
                default: Some(Value::String("utf8".to_string())),
                example: Some(Value::String("hex".to_string())),
                enum_values: Some(vec!["utf8".to_string(), "hex".to_string(), "base64".to_string()]),
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection and send timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "Optional delimiter to append (\\n, \\r\\n)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("\\n".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "wait_response".to_string(),
                param_type: "boolean".to_string(),
                description: "Wait for server response after sending".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "response_timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout for waiting response in seconds".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "response_buffer".to_string(),
                param_type: "integer".to_string(),
                description: "Buffer size for response".to_string(),
                required: false,
                default: Some(Value::Number(4096.into())),
                example: Some(Value::Number(8192.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "tcp_send",
            "parameters": {
                "host": "127.0.0.1",
                "port": 8080,
                "data": "Hello",
                "wait_response": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully sent 5 bytes to 127.0.0.1:8080\nResponse: ACK".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing tcp_send driver");
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 0) as u16;
        let data_str = get_param_string(parameters, "data")?;
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf8");
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        let delimiter = parameters.get("delimiter").and_then(|v| v.as_str()).unwrap_or("");
        let wait_response = get_param_bool(parameters, "wait_response", false);
        let response_timeout = get_param_u64(parameters, "response_timeout", 10);
        let response_buffer = get_param_u64(parameters, "response_buffer", 4096) as usize;
        info!("TCP send: host={}, port={}, data_size={}, wait_response={}", host, port, data_str.len(), wait_response);
        // Decode data based on encoding
        let data = match encoding {
            "hex" => hex::decode(data_str).map_err(|e| DriverError::execution(format!("Failed to decode hex: {}", e)))?,
            "base64" => STANDARD.decode(data_str).map_err(|e| DriverError::execution(format!("Failed to decode base64: {}", e)))?,
            _ => data_str.as_bytes().to_vec(),
        };
        // Handle delimiter
        let delimiter_bytes = match delimiter {
            "\\n" => "\n".as_bytes(),
            "\\r\\n" => "\r\n".as_bytes(),
            "\\r" => "\r".as_bytes(),
            _ => delimiter.as_bytes(),
        };
        let final_data = if !delimiter_bytes.is_empty() { [data.as_slice(), delimiter_bytes].concat() } else { data };
        // Connect to target
        let addr = format!("{}:{}", host, port);
        let timeout_dur = Duration::from_secs(timeout_secs);
        let connection = timeout(timeout_dur, TcpStream::connect(&addr))
            .await
            .map_err(|_| DriverError::execution(format!("Connection timeout after {}s", timeout_secs)))?
            .map_err(|e| DriverError::execution(format!("Failed to connect: {}", e)))?;
        let mut stream = connection;
        // Send data
        let bytes_sent = timeout(timeout_dur, async {
            stream.write_all(&final_data).await?;
            Ok::<_, std::io::Error>(final_data.len())
        })
        .await
        .map_err(|_| DriverError::execution(format!("Send timeout after {}s", timeout_secs)))?
        .map_err(|e| DriverError::execution(format!("Failed to send data: {}", e)))?;
        info!("Sent {} bytes to {}:{}", bytes_sent, host, port);
        let mut result = format!("Successfully sent {} bytes to {}:{}", bytes_sent, host, port);
        // Wait for response if requested
        if wait_response {
            debug!("Waiting for response from {}:{}", host, port);
            let mut buffer = vec![0u8; response_buffer];
            let read_result = timeout(Duration::from_secs(response_timeout), stream.read(&mut buffer))
                .await
                .map_err(|_| DriverError::execution(format!("Response timeout after {}s", response_timeout)))?
                .map_err(|e| DriverError::execution(format!("Failed to read response: {}", e)))?;
            let response = String::from_utf8_lossy(&buffer[..read_result]);
            info!("Received {} bytes response", read_result);
            result.push_str(&format!("\nResponse: {}", response));
        }
        return Ok(result);
    }
}
