//! Port lookup and testing utilities
//!
//! This module provides skills for looking up port information and testing port connectivity.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    common::net::get_service_name,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info};
/// A skill for looking up information about a specific port number.
#[derive(Debug)]
pub struct PortLookupDriver;
#[async_trait::async_trait]
impl Driver for PortLookupDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "port_lookup"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Look up information about a specific port number, including common services"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to know what service runs on a specific port"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "port".to_string(),
            param_type: "integer".to_string(),
            description: "Port number to look up".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(22.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "port_lookup",
            "parameters": {
                "port": 443
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Port 443: HTTPS - HTTP over TLS/SSL\nCommon services: HTTPS, SSL/TLS encrypted web traffic\nDefault protocol: TCP".to_string();
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
        debug!("Executing port_lookup driver");
        let port = parameters.get("port").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("port"))? as u16;
        info!("Looking up port: {}", port);
        let service = get_service_name(port);
        let protocol = match port {
            53 | 123 | 161 | 162 | 514 => "UDP/TCP",
            _ => "TCP",
        };
        let description = match port {
            20 | 21 => "File Transfer Protocol",
            22 => "Secure Shell for secure remote administration",
            23 => "Telnet - insecure remote access",
            25 => "Simple Mail Transfer Protocol for email routing",
            53 => "Domain Name System for domain resolution",
            80 => "Hypertext Transfer Protocol for web traffic",
            110 => "Post Office Protocol v3 for email retrieval",
            143 => "Internet Message Access Protocol for email",
            443 => "HTTP over TLS/SSL for secure web traffic",
            445 => "Server Message Block for file sharing",
            3306 => "MySQL database",
            5432 => "PostgreSQL database",
            6379 => "Redis in-memory data store",
            8080 => "HTTP alternative port for web proxies and servers",
            _ => "Unknown service",
        };
        info!("Port lookup result: {} - {}", port, service);
        return Ok(format!("Port {}: {} - {}\nProtocol: {}\nCommon services: {}\n", port, service, description, protocol, service));
    }
}
/// A skill for testing connectivity to a specific port on a target host.
#[derive(Debug)]
pub struct PortTestDriver;
#[async_trait::async_trait]
impl Driver for PortTestDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "port_test"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Test connectivity to a specific port on a target host"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if a specific port is reachable on a host"
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
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port number to test".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(80.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(3.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "port_test",
            "parameters": {
                "host": "example.com",
                "port": 443
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Testing connection to example.com:443...\nConnection successful! Port 443 is open and accepting connections\nResponse time: 45ms"
            .to_string();
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
        debug!("Executing port_test driver");
        let host = parameters.get("host").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("host"))?;
        let port = parameters.get("port").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("port"))? as u16;
        let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(3);
        info!("Port test: host={}, port={}, timeout={}s", host, port, timeout_secs);
        let addr = format!("{}:{}", host, port);
        let start = std::time::Instant::now();
        let result = timeout(Duration::from_secs(timeout_secs), async {
            let addrs: Vec<SocketAddr> = match addr.to_socket_addrs() {
                Ok(addrs) => addrs.collect(),
                Err(e) => return Err(DriverError::execution(format!("Failed to resolve host: {}", e))),
            };
            for addr in addrs {
                if tokio::time::timeout(Duration::from_secs(timeout_secs), tokio::net::TcpStream::connect(&addr)).await.is_ok() {
                    return Ok(addr);
                }
            }
            Err(DriverError::execution("Connection refused or timeout"))
        })
        .await;
        match result {
            Ok(Ok(resolved_addr)) => {
                let elapsed = start.elapsed();
                let service = get_service_name(port);
                info!("Port test successful: {}:{} - {}ms", host, port, elapsed.as_millis());
                return Ok(format!(
                    "Testing connection to {}:{}...\n✓ Connection successful! Port {} ({}) is open and accepting connections\nResponse time: {}ms\nResolved to: {}",
                    host,
                    port,
                    port,
                    service,
                    elapsed.as_millis(),
                    resolved_addr.ip()
                ));
            }
            Ok(Err(e)) => {
                info!("Port test failed: {}:{} - {}", host, port, e);
                return Ok(format!(
                    "Testing connection to {}:{}...\n✗ Connection failed: {}\nPort {} is likely closed or filtered",
                    host, port, e, port
                ));
            }
            Err(_) => {
                info!("Port test timeout: {}:{} after {}s", host, port, timeout_secs);
                return Ok(format!(
                    "Testing connection to {}:{}...\n✗ Connection timeout after {} seconds\nPort {} is unreachable or filtered",
                    host, port, timeout_secs, port
                ));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[tokio::test]
    async fn test_port_lookup_well_known() {
        let skill = PortLookupDriver;
        let mut params = HashMap::new();
        params.insert("port".to_string(), json!(22));
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("Port 22: SSH"));
        assert!(result.contains("Secure Shell"));
    }
    #[tokio::test]
    async fn test_port_test_open_port() {
        let skill = PortTestDriver;
        let mut params = HashMap::new();
        params.insert("host".to_string(), json!("google.com"));
        params.insert("port".to_string(), json!(80));
        params.insert("timeout".to_string(), json!(5));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Testing connection to google.com:80"));
        assert!(output.contains("✓ Connection successful"));
    }
}
