//! Port scanning and testing utilities for network discovery, service identification, and connectivity testing.
//!
//! This module provides several skills for port-related operations:
//! - `PortScanSkill`: Scan ports on a target host to discover open ports and services
//! - `PortLookupSkill`: Look up information about a specific port number
//! - `PortTestSkill`: Test connectivity to a specific port on a target host

use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

/// A skill for scanning ports on a target host to discover open ports and services.
///
/// This skill performs concurrent TCP port scanning with configurable port ranges,
/// timeout, and concurrency limits. It supports single ports, port ranges (e.g., 1-1024),
/// and comma-separated port lists. Results include service name identification for
/// well-known ports.
///
/// # Examples
/// ```
/// // Scan common web ports on a target
/// let result = port_scan_skill.execute(&HashMap::from([
///     ("target".to_string(), json!("scanme.nmap.org")),
///     ("ports".to_string(), json!("22,80,443,8080")),
///     ("timeout".to_string(), json!(2)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct PortScanSkill;

#[async_trait::async_trait]
impl Skill for PortScanSkill {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "port_scan"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Scan ports on a target host to discover open ports and services"
    }

    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to find open ports, check service availability, \
         or perform network reconnaissance on a specific host. Supports single port, port ranges, \
         comma-separated ports, and concurrent scanning."
    }

    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address to scan".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("scanme.nmap.org".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "ports".to_string(),
                param_type: "string".to_string(),
                description: "Ports to scan (e.g., '80', '1-1024', '22,80,443')".to_string(),
                required: false,
                default: Some(Value::String("1-1024".to_string())),
                example: Some(Value::String("22,80,443,8080".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds for each port".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "concurrency".to_string(),
                param_type: "integer".to_string(),
                description: "Number of concurrent connection attempts".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "show_closed".to_string(),
                param_type: "boolean".to_string(),
                description: "Show closed ports in results".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ]
    }

    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "port_scan",
            "parameters": {
                "target": "localhost",
                "ports": "1-1000",
                "timeout": 1
            }
        })
    }

    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        "Scanning localhost (127.0.0.1)\nPort 22: Open - SSH\nPort 80: Open - HTTP\nPort 443: Open - HTTPS\nTotal open ports: 3\nScan completed in 2.5 seconds".to_string()
    }

    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> &str {
        "net"
    }

    /// Executes a port scan on the target host
    ///
    /// # Arguments
    /// * `parameters` - A HashMap containing target, optional ports spec, timeout, concurrency, and show_closed
    ///
    /// # Returns
    /// A formatted string containing scan results including open ports and service names
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let target = parameters
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: target"))?;
        let ports_spec = parameters
            .get("ports")
            .and_then(|v| v.as_str())
            .unwrap_or("1-1024");
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        let concurrency = parameters
            .get("concurrency")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;
        let show_closed = parameters
            .get("show_closed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let addr = format!("{}:0", target);
        let resolved = match addr.to_socket_addrs() {
            Ok(mut addrs) => addrs.next().map(|s| s.ip()),
            Err(_) => None,
        };
        let ip = match resolved {
            Some(ip) => ip,
            None => return Ok(format!("Failed to resolve target: {}", target)),
        };
        let ports = parse_ports(ports_spec)?;
        let total_ports = ports.len();
        let start_time = std::time::Instant::now();
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let mut tasks = vec![];
        for port in ports {
            let permit = semaphore.clone().acquire_owned().await?;
            let target_ip = ip;
            let timeout_dur = Duration::from_secs(timeout_secs);
            let task = tokio::spawn(async move {
                let result = scan_port(target_ip, port, timeout_dur).await;
                drop(permit);
                (port, result)
            });
            tasks.push(task);
        }
        let mut open_ports = Vec::new();
        let mut closed_ports = Vec::new();
        for task in tasks {
            match task.await {
                Ok((port, is_open)) => {
                    if is_open {
                        open_ports.push(port);
                    } else if show_closed {
                        closed_ports.push(port);
                    }
                }
                Err(e) => eprintln!("Task failed: {}", e),
            }
        }
        open_ports.sort();
        closed_ports.sort();
        let duration = start_time.elapsed();
        let mut result = format!("Scanning {} ({})\n", target, ip);
        result.push_str(&format!("Total ports scanned: {}\n", total_ports));
        if !open_ports.is_empty() {
            result.push_str(&format!("\nOpen ports: {}\n", open_ports.len()));
            for port in &open_ports {
                let service = get_service_name(*port);
                result.push_str(&format!("  Port {}: Open - {}\n", port, service));
            }
        } else {
            result.push_str("\nNo open ports found\n");
        }
        if show_closed && !closed_ports.is_empty() {
            result.push_str(&format!("\nClosed ports: {}\n", closed_ports.len()));
            for port in closed_ports.iter().take(20) {
                result.push_str(&format!("  Port {}\n", port));
            }
            if closed_ports.len() > 20 {
                result.push_str(&format!("  ... and {} more\n", closed_ports.len() - 20));
            }
        }
        result.push_str(&format!(
            "\nScan completed in {:.2} seconds",
            duration.as_secs_f64()
        ));
        Ok(result)
    }
}

/// Returns the common service name associated with a given port number
///
/// # Arguments
/// * `port` - The port number to look up
///
/// # Returns
/// A string slice containing the service name
fn get_service_name(port: u16) -> &'static str {
    match port {
        20 => "FTP-data",
        21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 => "SMTP",
        53 => "DNS",
        80 => "HTTP",
        110 => "POP3",
        111 => "RPC",
        135 => "RPC",
        139 => "NetBIOS",
        143 => "IMAP",
        443 => "HTTPS",
        445 => "SMB",
        993 => "IMAPS",
        995 => "POP3S",
        1433 => "MSSQL",
        1521 => "Oracle",
        3306 => "MySQL",
        3389 => "RDP",
        5432 => "PostgreSQL",
        6379 => "Redis",
        8080 => "HTTP-Alt",
        8443 => "HTTPS-Alt",
        27017 => "MongoDB",
        _ => "Unknown",
    }
}

/// Attempts to connect to a port on a target IP address
///
/// # Arguments
/// * `ip` - The target IP address
/// * `port` - The port number to connect to
/// * `timeout_dur` - The duration to wait before timing out
///
/// # Returns
/// `true` if the connection was successful, `false` otherwise
async fn scan_port(ip: IpAddr, port: u16, timeout_dur: Duration) -> bool {
    let addr = SocketAddr::new(ip, port);
    match timeout(timeout_dur, TcpStream::connect_addr(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

/// Parses a port specification string into a vector of port numbers
///
/// Supports formats like:
/// - Single port: "80"
/// - Port range: "1-1024"
/// - Comma-separated: "22,80,443"
/// - Mixed: "22,80-90,443"
///
/// # Arguments
/// * `ports_spec` - The port specification string
///
/// # Returns
/// A vector of sorted, unique port numbers
fn parse_ports(ports_spec: &str) -> Result<Vec<u16>> {
    let mut ports = Vec::new();
    for part in ports_spec.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                let start = range[0].parse::<u16>()?;
                let end = range[1].parse::<u16>()?;
                for port in start..=end {
                    ports.push(port);
                }
            }
        } else {
            let port = part.parse::<u16>()?;
            ports.push(port);
        }
    }
    ports.sort();
    ports.dedup();
    Ok(ports)
}

/// A skill for looking up information about a specific port number.
///
/// Provides details about common services, protocols, and descriptions for well-known ports.
///
/// # Examples
/// ```
/// // Look up information about port 22 (SSH)
/// let result = port_lookup_skill.execute(&HashMap::from([
///     ("port".to_string(), json!(22)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct PortLookupSkill;

#[async_trait::async_trait]
impl Skill for PortLookupSkill {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "port_lookup"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Look up information about a specific port number, including common services and vulnerabilities"
    }

    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to know what service runs on a specific port"
    }

    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "port".to_string(),
            param_type: "integer".to_string(),
            description: "Port number to look up".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(22.into())),
            enum_values: None,
        }]
    }

    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "port_lookup",
            "parameters": {
                "port": 443
            }
        })
    }

    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        "Port 443: HTTPS - HTTP over TLS/SSL\nCommon services: HTTPS, SSL/TLS encrypted web traffic\nDefault protocol: TCP".to_string()
    }

    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> &str {
        "net"
    }

    /// Looks up information about a specific port
    ///
    /// # Arguments
    /// * `parameters` - A HashMap containing the port number
    ///
    /// # Returns
    /// A formatted string containing service information and description
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: port"))?
            as u16;
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
        Ok(format!(
            "Port {}: {} - {}\nProtocol: {}\nCommon services: {}\n",
            port, service, description, protocol, service
        ))
    }
}

/// A skill for testing connectivity to a specific port on a target host.
///
/// Performs a TCP connection attempt to verify if a specific port is open and accepting connections.
/// Useful for quick connectivity checks and service availability testing.
///
/// # Examples
/// ```
/// // Test HTTPS connectivity to google.com
/// let result = port_test_skill.execute(&HashMap::from([
///     ("host".to_string(), json!("google.com")),
///     ("port".to_string(), json!(443)),
///     ("timeout".to_string(), json!(3)),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct PortTestSkill;

#[async_trait::async_trait]
impl Skill for PortTestSkill {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "port_test"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Test connectivity to a specific port on a target host"
    }

    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if a specific port is reachable on a host"
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
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port number to test".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(80.into())),
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
            "action": "port_test",
            "parameters": {
                "host": "example.com",
                "port": 443
            }
        })
    }

    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        "Testing connection to example.com:443...\nConnection successful! Port 443 is open and accepting connections\nResponse time: 45ms".to_string()
    }

    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> &str {
        "net"
    }

    /// Tests connectivity to a specific port on a target host
    ///
    /// # Arguments
    /// * `parameters` - A HashMap containing host, port, and optional timeout
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
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: port"))?
            as u16;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);
        let addr = format!("{}:{}", host, port);
        let start = std::time::Instant::now();
        match timeout(Duration::from_secs(timeout_secs), async {
            let addrs: Vec<SocketAddr> = match addr.to_socket_addrs() {
                Ok(addrs) => addrs.collect(),
                Err(e) => return Err(anyhow::anyhow!("Failed to resolve host: {}", e)),
            };
            for addr in addrs {
                if let Ok(_) = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs))
                {
                    return Ok(addr);
                }
            }
            Err(anyhow::anyhow!("Connection refused or timeout"))
        })
        .await
        {
            Ok(Ok(resolved_addr)) => {
                let elapsed = start.elapsed();
                let service = get_service_name(port);
                Ok(format!(
                    "Testing connection to {}:{}...\n✓ Connection successful! Port {} ({}) is open and accepting connections\nResponse time: {}ms\nResolved to: {}",
                    host,
                    port,
                    port,
                    service,
                    elapsed.as_millis(),
                    resolved_addr.ip()
                ))
            }
            Ok(Err(e)) => Ok(format!(
                "Testing connection to {}:{}...\n✗ Connection failed: {}\nPort {} is likely closed or filtered",
                host, port, e, port
            )),
            Err(_) => Ok(format!(
                "Testing connection to {}:{}...\n✗ Connection timeout after {} seconds\nPort {} is unreachable or filtered",
                host, port, timeout_secs, port
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test parsing of port specifications
    #[test]
    fn test_parse_ports() {
        let ports = parse_ports("80").unwrap();
        assert_eq!(ports, vec![80]);
        let ports = parse_ports("1-5").unwrap();
        assert_eq!(ports, vec![1, 2, 3, 4, 5]);
        let ports = parse_ports("22,80,443").unwrap();
        assert_eq!(ports, vec![22, 80, 443]);
        let ports = parse_ports("22,80-82,443").unwrap();
        assert_eq!(ports, vec![22, 80, 81, 82, 443]);
        let ports = parse_ports("80,80,443,443").unwrap();
        assert_eq!(ports, vec![80, 443]);
    }

    /// Test port lookup for well-known ports
    #[tokio::test]
    async fn test_port_lookup_well_known() {
        let skill = PortLookupSkill;
        let mut params = HashMap::new();
        params.insert("port".to_string(), json!(22));
        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("Port 22: SSH"));
        assert!(result.contains("Secure Shell"));
        let mut params = HashMap::new();
        params.insert("port".to_string(), json!(80));
        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("Port 80: HTTP"));
        assert!(result.contains("Hypertext Transfer Protocol"));
        let mut params = HashMap::new();
        params.insert("port".to_string(), json!(443));
        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("Port 443: HTTPS"));
    }

    /// Test port connectivity to a known open port (google.com:80)
    #[tokio::test]
    async fn test_port_test_open_port() {
        let skill = PortTestSkill;
        let mut params = HashMap::new();
        params.insert("host".to_string(), json!("google.com"));
        params.insert("port".to_string(), json!(80));
        params.insert("timeout".to_string(), json!(5));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Testing connection to google.com:80"));
        assert!(output.contains("✓ Connection successful"));
    }
}
