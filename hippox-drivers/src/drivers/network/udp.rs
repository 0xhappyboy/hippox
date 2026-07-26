//! UDP operations driver
//!
//! This module provides drivers for UDP operations including send, receive, and broadcast.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, info};
/// Gets a string parameter from the parameters map
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name))
}
/// Gets a u64 parameter from the parameters map with a default value
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
/// UDP Send Driver
#[derive(Debug)]
pub struct UdpSendDriver;
#[async_trait::async_trait]
impl Driver for UdpSendDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "udp_send"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send data over UDP"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to send UDP datagram to a server"
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
                example: Some(Value::Number(9999.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, UDP Server!".to_string())),
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
                description: "Send timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "udp_send",
            "parameters": {
                "host": "127.0.0.1",
                "port": 9999,
                "data": "Hello"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully sent 5 bytes to 127.0.0.1:9999".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing udp_send driver");
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 0) as u16;
        let data_str = get_param_string(parameters, "data")?;
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf8");
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        info!("UDP send: host={}, port={}, size={}", host, port, data_str.len());
        let data = match encoding {
            "hex" => hex::decode(data_str).map_err(|e| DriverError::execution(format!("Failed to decode hex: {}", e)))?,
            "base64" => STANDARD.decode(data_str).map_err(|e| DriverError::execution(format!("Failed to decode base64: {}", e)))?,
            _ => data_str.as_bytes().to_vec(),
        };
        let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| DriverError::execution(format!("Failed to bind socket: {}", e)))?;
        let addr = format!("{}:{}", host, port);
        let bytes_sent = timeout(std::time::Duration::from_secs(timeout_secs), socket.send_to(&data, &addr))
            .await
            .map_err(|_| DriverError::execution(format!("Send timeout after {}s", timeout_secs)))?
            .map_err(|e| DriverError::execution(format!("Failed to send: {}", e)))?;
        info!("UDP send successful: {} bytes", bytes_sent);
        return Ok(format!("Successfully sent {} bytes to {}:{}", bytes_sent, host, port));
    }
}
/// UDP Receive Driver
#[derive(Debug)]
pub struct UdpReceiveDriver;
#[async_trait::async_trait]
impl Driver for UdpReceiveDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "udp_receive"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Receive UDP datagram"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to listen for UDP packets"
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
                description: "Port to bind and listen on".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(9999.into())),
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
                description: "Receive timeout in seconds".to_string(),
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
                description: "Optional response to send back to sender".to_string(),
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
            "action": "udp_receive",
            "parameters": {
                "port": 9999,
                "timeout": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Received 11 bytes from 127.0.0.1:54321:\nHello, UDP!".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing udp_receive driver");
        let port = get_param_u64(parameters, "port", 0) as u16;
        let bind_address = parameters.get("bind_address").and_then(|v| v.as_str()).unwrap_or("0.0.0.0");
        let buffer_size = get_param_u64(parameters, "buffer_size", 4096) as usize;
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf8");
        let send_response = parameters.get("send_response").and_then(|v| v.as_str());
        let addr = format!("{}:{}", bind_address, port);
        info!("UDP receive listening on {}", addr);
        let socket = UdpSocket::bind(&addr).await.map_err(|e| DriverError::execution(format!("Failed to bind: {}", e)))?;
        let mut buffer = vec![0u8; buffer_size];
        let receive_result = timeout(std::time::Duration::from_secs(timeout_secs), socket.recv_from(&mut buffer))
            .await
            .map_err(|_| DriverError::execution(format!("Receive timeout after {}s", timeout_secs)))?
            .map_err(|e| DriverError::execution(format!("Failed to receive: {}", e)))?;
        let (size, src_addr) = receive_result;
        let received_data = &buffer[..size];
        info!("Received {} bytes from {}", size, src_addr);
        let output = match encoding {
            "hex" => hex::encode(received_data),
            "base64" => STANDARD.encode(received_data),
            _ => String::from_utf8_lossy(received_data).to_string(),
        };
        let mut result = format!("Received {} bytes from {}:\n{}", size, src_addr, output);
        if let Some(response) = send_response {
            socket.send_to(response.as_bytes(), src_addr).await.map_err(|e| DriverError::execution(format!("Failed to send response: {}", e)))?;
            result.push_str(&format!("\nResponse sent: {}", response));
            info!("Response sent: {}", response);
        }
        return Ok(result);
    }
}
/// UDP Broadcast Driver
#[derive(Debug)]
pub struct UdpBroadcastDriver;
#[async_trait::async_trait]
impl Driver for UdpBroadcastDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "udp_broadcast"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send UDP broadcast message"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to send a broadcast message to all hosts on the network"
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
                description: "Target port number".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(9999.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to broadcast".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("DISCOVER".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Data encoding (utf8, hex, base64)".to_string(),
                required: false,
                default: Some(Value::String("utf8".to_string())),
                example: Some(Value::String("utf8".to_string())),
                enum_values: Some(vec!["utf8".to_string(), "hex".to_string(), "base64".to_string()]),
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Send timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "udp_broadcast",
            "parameters": {
                "port": 9999,
                "data": "DISCOVER"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully broadcasted 7 bytes to port 9999".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing udp_broadcast driver");
        let port = get_param_u64(parameters, "port", 0) as u16;
        let data_str = get_param_string(parameters, "data")?;
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf8");
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        info!("UDP broadcast: port={}, size={}", port, data_str.len());
        let data = match encoding {
            "hex" => hex::decode(data_str).map_err(|e| DriverError::execution(format!("Failed to decode hex: {}", e)))?,
            "base64" => STANDARD.decode(data_str).map_err(|e| DriverError::execution(format!("Failed to decode base64: {}", e)))?,
            _ => data_str.as_bytes().to_vec(),
        };
        let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| DriverError::execution(format!("Failed to bind socket: {}", e)))?;
        socket.set_broadcast(true).map_err(|e| DriverError::execution(format!("Failed to set broadcast: {}", e)))?;
        let broadcast_addr = format!("255.255.255.255:{}", port);
        let bytes_sent = timeout(std::time::Duration::from_secs(timeout_secs), socket.send_to(&data, &broadcast_addr))
            .await
            .map_err(|_| DriverError::execution(format!("Send timeout after {}s", timeout_secs)))?
            .map_err(|e| DriverError::execution(format!("Failed to send: {}", e)))?;
        info!("UDP broadcast successful: {} bytes", bytes_sent);
        return Ok(format!("Successfully broadcasted {} bytes to port {}", bytes_sent, port));
    }
}
