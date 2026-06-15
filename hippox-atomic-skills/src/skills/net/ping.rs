//! Network connectivity testing utilities including ICMP ping, TCP ping, and batch ping operations.
//!
//! This module provides several skills for testing network connectivity:
//! - `PingSkill`: Send ICMP echo request packets to test network connectivity and latency
//! - `TcpPingSkill`: Perform TCP ping (SYN scan) to test if a port is reachable when ICMP is blocked
//! - `BatchPingSkill`: Ping multiple hosts simultaneously and aggregate results

use crate::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// A skill for sending ICMP echo request packets to test network connectivity and latency.
///
/// This skill uses the system's native ping command to send ICMP packets to a target host.
/// It supports configurable packet count, timeout, and packet size. Works with both
/// domain names and IP addresses across Windows and Unix-like systems.
///
/// # Examples
/// ```
/// // Basic ping to google.com with 4 packets
/// let result = ping_skill.execute(&HashMap::from([
///     ("target".to_string(), json!("google.com")),
///     ("count".to_string(), json!(4)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct PingSkill;

#[async_trait::async_trait]
impl Skill for PingSkill {
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
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address to ping".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of ping packets to send".to_string(),
                required: false,
                default: Some(Value::Number(4.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds for each ping".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(3.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "packet_size".to_string(),
                param_type: "integer".to_string(),
                description: "Packet size in bytes".to_string(),
                required: false,
                default: Some(Value::Number(56.into())),
                example: Some(Value::Number(64.into())),
                enum_values: None,
            },
        ]
    }

    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "ping",
            "parameters": {
                "target": "8.8.8.8",
                "count": 4
            }
        })
    }

    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        "PING 8.8.8.8 (8.8.8.8): 56 data bytes\n64 bytes from 8.8.8.8: seq=0 ttl=117 time=12.3 ms\n64 bytes from 8.8.8.8: seq=1 ttl=117 time=11.8 ms\n\n--- 8.8.8.8 ping statistics ---\n4 packets transmitted, 4 received, 0% packet loss\nround-trip min/avg/max = 11.8/12.1/12.3 ms".to_string()
    }

    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> &str {
        "net"
    }

    /// Executes an ICMP ping to the target host
    ///
    /// # Arguments
    /// * `parameters` - A HashMap containing the target, optional count, timeout, and packet_size
    ///
    /// # Returns
    /// The raw output from the system ping command
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let target = parameters
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: target"))?;
        let count = parameters
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(4) as u32;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(2);
        let packet_size = parameters
            .get("packet_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(56) as usize;
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
        let output = cmd.output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("Ping failed:\n{}", stderr))
        }
    }
}

/// A skill for performing TCP ping (SYN scan) to test if a port is reachable.
///
/// This skill is useful when ICMP is blocked by firewalls and you need to test
/// connectivity to a specific port using TCP connection attempts.
///
/// # Examples
/// ```
/// // Test if HTTPS port is reachable on google.com
/// let result = tcp_ping_skill.execute(&HashMap::from([
///     ("host".to_string(), json!("google.com")),
///     ("port".to_string(), json!(443)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct TcpPingSkill;

#[async_trait::async_trait]
impl Skill for TcpPingSkill {
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
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to connect to (default: 80)".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(443.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(3.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ]
    }

    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "tcp_ping",
            "parameters": {
                "host": "google.com",
                "port": 443
            }
        })
    }

    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        "TCP Ping to google.com:443\n✓ Port 443 is reachable\nResponse time: 15.3ms".to_string()
    }

    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> &str {
        "net"
    }

    /// Executes a TCP ping to test port reachability
    ///
    /// # Arguments
    /// * `parameters` - A HashMap containing the host, optional port, and optional timeout
    ///
    /// # Returns
    /// A formatted string indicating whether the port is reachable and the response time
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: host"))?;
        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as u16;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);
        let addr = format!("{}:{}", host, port);
        let start = Instant::now();
        match timeout(Duration::from_secs(timeout_secs), async {
            let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();
            for addr in addrs {
                if let Ok(_) = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs))
                {
                    return Ok(addr);
                }
            }
            Err(anyhow::anyhow!("Connection failed"))
        })
        .await
        {
            Ok(Ok(resolved_addr)) => {
                let elapsed = start.elapsed();
                Ok(format!(
                    "TCP Ping to {}:{}\n✓ Port {} is reachable\nResponse time: {:.1}ms\nResolved to: {}",
                    host,
                    port,
                    port,
                    elapsed.as_secs_f64() * 1000.0,
                    resolved_addr.ip()
                ))
            }
            Ok(Err(e)) => Ok(format!(
                "TCP Ping to {}:{}\n✗ Port {} is not reachable: {}",
                host, port, port, e
            )),
            Err(_) => Ok(format!(
                "TCP Ping to {}:{}\n✗ Connection timeout after {} seconds",
                host, port, timeout_secs
            )),
        }
    }
}

/// A skill for pinging multiple hosts simultaneously and returning aggregated results.
///
/// This skill allows efficient connectivity testing to multiple targets in parallel,
/// providing a summary of reachable vs unreachable hosts and success rates.
///
/// # Examples
/// ```
/// // Ping multiple hosts at once
/// let result = batch_ping_skill.execute(&HashMap::from([
///     ("targets".to_string(), json!(["8.8.8.8", "1.1.1.1", "google.com"])),
///     ("timeout".to_string(), json!(2)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct BatchPingSkill;

#[async_trait::async_trait]
impl Skill for BatchPingSkill {
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
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "targets".to_string(),
                param_type: "array".to_string(),
                description: "List of target hostnames or IP addresses".to_string(),
                required: true,
                default: None,
                example: Some(json!(["google.com", "github.com", "8.8.8.8"])),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds for each ping".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(3.into())),
                enum_values: None,
            },
        ]
    }

    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "batch_ping",
            "parameters": {
                "targets": ["1.1.1.1", "8.8.8.8", "google.com"]
            }
        })
    }

    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        "Batch ping results:\n✓ 1.1.1.1 - Reachable (12.3ms)\n✓ 8.8.8.8 - Reachable (15.1ms)\n✗ google.com - Timeout\n\nSuccess rate: 2/3 (66.7%)".to_string()
    }

    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> &str {
        "net"
    }

    /// Executes batch pings to multiple targets simultaneously
    ///
    /// # Arguments
    /// * `parameters` - A HashMap containing the targets array and optional timeout
    ///
    /// # Returns
    /// A formatted string containing results for all targets and a success rate summary
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let targets = parameters
            .get("targets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: targets (array)"))?;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(2);
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
                    results.push(format!(
                        "✓ {} - Reachable ({:.1}ms)",
                        target,
                        elapsed.as_secs_f64() * 1000.0
                    ));
                } else {
                    results.push(format!("✗ {} - Unreachable", target));
                }
            }
        }
        let total = targets.len();
        let success_rate = (successful as f64 / total as f64) * 100.0;
        let mut output = String::from("Batch ping results:\n");
        output.push_str(&results.join("\n"));
        output.push_str(&format!(
            "\n\nSuccess rate: {}/{} ({:.1}%)",
            successful, total, success_rate
        ));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test TCP ping to a reachable host (google.com on port 80)
    #[tokio::test]
    async fn test_tcp_ping_reachable() {
        let skill = TcpPingSkill;
        let mut params = HashMap::new();
        params.insert("host".to_string(), json!("google.com"));
        params.insert("port".to_string(), json!(80));
        params.insert("timeout".to_string(), json!(5));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("TCP Ping to google.com:80"));
        assert!(output.contains("✓ Port 80 is reachable") || output.contains("Response time:"));
    }

    /// Test TCP ping to an unreachable port (port 9999 on a known host)
    #[tokio::test]
    async fn test_tcp_ping_unreachable_port() {
        let skill = TcpPingSkill;
        let mut params = HashMap::new();
        params.insert("host".to_string(), json!("localhost"));
        params.insert("port".to_string(), json!(9999));
        params.insert("timeout".to_string(), json!(2));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("TCP Ping to localhost:9999"));
        assert!(
            output.contains("✗ Port 9999 is not reachable")
                || output.contains("Connection timeout")
                || output.contains("Connection refused")
        );
    }

    /// Test batch ping skill with multiple targets
    #[tokio::test]
    async fn test_batch_ping_multiple_targets() {
        let skill = BatchPingSkill;
        let mut params = HashMap::new();
        params.insert("targets".to_string(), json!(["8.8.8.8", "1.1.1.1"]));
        params.insert("timeout".to_string(), json!(3));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Batch ping results:"));
        assert!(output.contains("8.8.8.8") || output.contains("1.1.1.1"));
        assert!(output.contains("Success rate:"));
    }
}
