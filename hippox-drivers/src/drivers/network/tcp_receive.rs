//! TCP receive driver
//!
//! This driver provides functionality to accept a TCP connection and read data once.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, get_param_u64,
    types::{Driver, DriverParameter},
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use tracing::{debug, info};
/// TCP Receive Driver - single-shot receiver
#[derive(Debug)]
pub struct TcpReceiveDriver;
#[async_trait::async_trait]
impl Driver for TcpReceiveDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "tcp_receive"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Accept ONE TCP connection, read ONCE up to buffer_size bytes, return data, then close. Excess data is truncated."
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Single-shot receiver. Reads once, max buffer_size bytes (default 4096). For larger data, increase buffer_size or call repeatedly."
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to listen on".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(8888.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "bind_address".to_string(),
                param_type: "string".to_string(),
                description: "Address to bind (default: 0.0.0.0)".to_string(),
                required: false,
                default: Some(Value::String("0.0.0.0".to_string())),
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "buffer_size".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum bytes to receive".to_string(),
                required: false,
                default: Some(Value::Number(4096.into())),
                example: Some(Value::Number(8192.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Wait timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Output encoding (utf8, hex, base64)".to_string(),
                required: false,
                default: Some(Value::String("utf8".to_string())),
                example: Some(Value::String("hex".to_string())),
                enum_values: Some(vec!["utf8".to_string(), "hex".to_string(), "base64".to_string()]),
            },
            DriverParameter {
                name: "send_response".to_string(),
                param_type: "string".to_string(),
                description: "Optional response to send back to client".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ACK".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "tcp_receive",
            "parameters": {
                "port": 8888,
                "timeout": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Received 42 bytes from 127.0.0.1:54321:\nHello, TCP Server!\nResponse sent: OK".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing tcp_receive driver");
        let port = get_param_u64(parameters, "port", 0) as u16;
        let bind_address = parameters.get("bind_address").and_then(|v| v.as_str()).unwrap_or("0.0.0.0");
        let buffer_size = get_param_u64(parameters, "buffer_size", 4096) as usize;
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf8");
        let send_response = parameters.get("send_response").and_then(|v| v.as_str());
        let addr = format!("{}:{}", bind_address, port);
        info!("TCP receive listening on {}", addr);
        let listener = TcpListener::bind(&addr).await.map_err(|e| DriverError::execution(format!("Failed to bind: {}", e)))?;
        let accept_result = timeout(std::time::Duration::from_secs(timeout_secs), listener.accept())
            .await
            .map_err(|_| DriverError::execution(format!("Timeout waiting for connection after {}s", timeout_secs)))?
            .map_err(|e| DriverError::execution(format!("Failed to accept connection: {}", e)))?;
        let (mut stream, client_addr) = accept_result;
        info!("TCP connection accepted from {}", client_addr);
        let mut buffer = vec![0u8; buffer_size];
        let read_result = timeout(std::time::Duration::from_secs(timeout_secs), stream.read(&mut buffer))
            .await
            .map_err(|_| DriverError::execution("Timeout reading data"))?
            .map_err(|e| DriverError::execution(format!("Failed to read data: {}", e)))?;
        let received_data = &buffer[..read_result];
        info!("Received {} bytes from {}", read_result, client_addr);
        let output = match encoding {
            "hex" => hex::encode(received_data),
            "base64" => STANDARD.encode(received_data),
            _ => String::from_utf8_lossy(received_data).to_string(),
        };
        let mut result = format!("Received {} bytes from {}:\n{}", read_result, client_addr, output);
        if let Some(response) = send_response {
            stream.write_all(response.as_bytes()).await.map_err(|e| DriverError::execution(format!("Failed to send response: {}", e)))?;
            result.push_str(&format!("\nResponse sent: {}", response));
            info!("Response sent: {}", response);
        }
        return Ok(result);
    }
}
