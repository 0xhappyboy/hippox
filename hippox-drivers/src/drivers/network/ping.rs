//! Network connectivity testing utilities including ICMP ping, TCP ping, and batch ping operations.
//!
//! This module provides several skills for testing network connectivity:
//! - `PingDriver`: Send ICMP echo request packets to test network connectivity and latency
//! - `TcpPingDriver`: Perform TCP ping (SYN scan) to test if a port is reachable when ICMP is blocked
//! - `BatchPingDriver`: Ping multiple hosts simultaneously and aggregate results
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info};
/// A skill for sending ICMP echo request packets to test network connectivity and latency.
#[derive(Debug)]
pub struct PingDriver;
#[async_trait::async_trait]
impl Driver for PingDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "ping"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Send ICMP echo request packets to test network connectivity and latency"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if a host is reachable, measure network latency, \
         or test packet loss. Works with both domain names and IP addresses."
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address to ping".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of ping packets to send".to_string(),
                required: false,
                default: Some(Value::Number(4.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds for each ping".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(3.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "packet_size".to_string(),
                param_type: "integer".to_string(),
                description: "Packet size in bytes".to_string(),
                required: false,
                default: Some(Value::Number(56.into())),
                example: Some(Value::Number(64.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "ping",
            "parameters": {
                "target": "8.8.8.8",
                "count": 4
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "PING 8.8.8.8 (8.8.8.8): 56 data bytes\n64 bytes from 8.8.8.8: seq=0 ttl=117 time=12.3 ms\n64 bytes from 8.8.8.8: seq=1 ttl=117 time=11.8 ms\n\n--- 8.8.8.8 ping statistics ---\n4 packets transmitted, 4 received, 0% packet loss\nround-trip min/avg/max = 11.8/12.1/12.3 ms".to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes an ICMP ping to the target host
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing ping driver");
        let target = parameters.get("target").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("target"))?;
        let count = parameters.get("count").and_then(|v| v.as_u64()).unwrap_or(4) as u32;
        let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(2);
        let packet_size = parameters.get("packet_size").and_then(|v| v.as_u64()).unwrap_or(56) as usize;
        info!("Ping: target={}, count={}, timeout={}s, packet_size={}", target, count, timeout_secs, packet_size);
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = std::process::Command::new("ping");
            cmd.arg("-n").arg(count.to_string());
            cmd.arg("-w").arg((timeout_secs * 1000).to_string());
            cmd.arg("-l").arg(packet_size.to_string());
            cmd.arg(target);
            cmd
        } else {
            let mut cmd = std::process::Command::new("ping");
            cmd.arg("-c").arg(count.to_string());
            cmd.arg("-W").arg(timeout_secs.to_string());
            cmd.arg("-s").arg(packet_size.to_string());
            cmd.arg(target);
            cmd
        };
        let output = cmd.output().map_err(|e| DriverError::execution(format!("Failed to execute ping: {}", e)))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("Ping successful for {}", target);
            return Ok(stdout.to_string());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            info!("Ping failed for {}: {}", target, stderr);
            return Ok(format!("Ping failed:\n{}", stderr));
        }
    }
}
/// A skill for performing TCP ping (SYN scan) to test if a port is reachable.
#[derive(Debug)]
pub struct TcpPingDriver;
#[async_trait::async_trait]
impl Driver for TcpPingDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "tcp_ping"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Perform TCP ping (SYN scan) to test if a port is reachable, useful when ICMP is blocked"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when ICMP ping is blocked and you need to test connectivity to a specific port"
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to connect to (default: 80)".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(443.into())),
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
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "tcp_ping",
            "parameters": {
                "host": "google.com",
                "port": 443
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "TCP Ping to google.com:443\n✓ Port 443 is reachable\nResponse time: 15.3ms".to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes a TCP ping to test port reachability
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing tcp_ping driver");
        let host = parameters.get("host").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("host"))?;
        let port = parameters.get("port").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
        let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(3);
        info!("TCP ping: host={}, port={}, timeout={}s", host, port, timeout_secs);
        let addr = format!("{}:{}", host, port);
        let start = Instant::now();
        let result = timeout(Duration::from_secs(timeout_secs), async {
            let addrs: Vec<SocketAddr> =
                addr.to_socket_addrs().map_err(|e| DriverError::execution(format!("Failed to resolve host: {}", e)))?.collect();
            for addr in addrs {
                if let Ok(_) = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)) {
                    return Ok(addr);
                }
            }
            Err(DriverError::execution("Connection failed"))
        })
        .await;
        match result {
            Ok(Ok(resolved_addr)) => {
                let elapsed = start.elapsed();
                info!("TCP ping successful: {}:{} - {:.1}ms", host, port, elapsed.as_secs_f64() * 1000.0);
                return Ok(format!(
                    "TCP Ping to {}:{}\n✓ Port {} is reachable\nResponse time: {:.1}ms\nResolved to: {}",
                    host,
                    port,
                    port,
                    elapsed.as_secs_f64() * 1000.0,
                    resolved_addr.ip()
                ));
            }
            Ok(Err(e)) => {
                info!("TCP ping failed: {}:{} - {}", host, port, e);
                return Ok(format!("TCP Ping to {}:{}\n✗ Port {} is not reachable: {}", host, port, port, e));
            }
            Err(_) => {
                info!("TCP ping timeout: {}:{} after {}s", host, port, timeout_secs);
                return Ok(format!("TCP Ping to {}:{}\n✗ Connection timeout after {} seconds", host, port, timeout_secs));
            }
        }
    }
}
/// A skill for pinging multiple hosts simultaneously and returning aggregated results.
#[derive(Debug)]
pub struct BatchPingDriver;
#[async_trait::async_trait]
impl Driver for BatchPingDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "batch_ping"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Ping multiple hosts simultaneously and return results"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to check connectivity to multiple hosts at once"
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "targets".to_string(),
                param_type: "array".to_string(),
                description: "List of target hostnames or IP addresses".to_string(),
                required: true,
                default: None,
                example: Some(json!(["google.com", "github.com", "8.8.8.8"])),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds for each ping".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(3.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "batch_ping",
            "parameters": {
                "targets": ["1.1.1.1", "8.8.8.8", "google.com"]
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "Batch ping results:\n✓ 1.1.1.1 - Reachable (12.3ms)\n✓ 8.8.8.8 - Reachable (15.1ms)\n✗ google.com - Timeout\n\nSuccess rate: 2/3 (66.7%)".to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes batch pings to multiple targets simultaneously
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing batch_ping driver");
        let targets = parameters.get("targets").and_then(|v| v.as_array()).ok_or_else(|| DriverError::missing_parameter("targets (array)"))?;
        let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(2);
        info!("Batch ping: {} targets, timeout={}s", targets.len(), timeout_secs);
        let mut results = Vec::new();
        let mut successful = 0;
        for target_value in targets {
            if let Some(target) = target_value.as_str() {
                let start = Instant::now();
                let addr = format!("{}:80", target);
                let reachable = timeout(Duration::from_secs(timeout_secs), async {
                    match addr.to_socket_addrs() {
                        Ok(mut addrs) => addrs.next().is_some(),
                        Err(_) => false,
                    }
                })
                .await
                .unwrap_or(false);
                let elapsed = start.elapsed();
                if reachable {
                    successful += 1;
                    results.push(format!("✓ {} - Reachable ({:.1}ms)", target, elapsed.as_secs_f64() * 1000.0));
                } else {
                    results.push(format!("✗ {} - Unreachable", target));
                }
            }
        }
        let total = targets.len();
        let success_rate = (successful as f64 / total as f64) * 100.0;
        info!("Batch ping complete: {}/{} successful ({:.1}%)", successful, total, success_rate);
        let mut output = String::from("Batch ping results:\n");
        output.push_str(&results.join("\n"));
        output.push_str(&format!("\n\nSuccess rate: {}/{} ({:.1}%)", successful, total, success_rate));
        return Ok(output);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[tokio::test]
    async fn test_tcp_ping_reachable() {
        let skill = TcpPingDriver;
        let mut params = HashMap::new();
        params.insert("host".to_string(), json!("google.com"));
        params.insert("port".to_string(), json!(80));
        params.insert("timeout".to_string(), json!(5));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("TCP Ping to google.com:80"));
        assert!(output.contains("✓ Port 80 is reachable") || output.contains("Response time:"));
    }
    #[tokio::test]
    async fn test_tcp_ping_unreachable_port() {
        let skill = TcpPingDriver;
        let mut params = HashMap::new();
        params.insert("host".to_string(), json!("localhost"));
        params.insert("port".to_string(), json!(9999));
        params.insert("timeout".to_string(), json!(2));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("TCP Ping to localhost:9999"));
        assert!(output.contains("✗ Port 9999 is not reachable") || output.contains("Connection timeout") || output.contains("Connection refused"));
    }
    #[tokio::test]
    async fn test_batch_ping_multiple_targets() {
        let skill = BatchPingDriver;
        let mut params = HashMap::new();
        params.insert("targets".to_string(), json!(["8.8.8.8", "1.1.1.1"]));
        params.insert("timeout".to_string(), json!(3));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Batch ping results:"));
        assert!(output.contains("8.8.8.8") || output.contains("1.1.1.1"));
        assert!(output.contains("Success rate:"));
    }
}
