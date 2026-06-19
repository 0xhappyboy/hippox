//! Firewall detection Driver

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    common::net::{parse_ports, resolve_host},
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[derive(Debug)]
pub struct FirewallCheckDriver;

#[async_trait::async_trait]
impl Driver for FirewallCheckDriver {
    fn name(&self) -> &str {
        "firewall_check"
    }

    fn description(&self) -> &str {
        "Check firewall behavior by analyzing response patterns to different port scans"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to detect firewall presence and behavior based on port responses"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("scanme.nmap.org".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "ports".to_string(),
                param_type: "string".to_string(),
                description: "Ports to test (default: common ports)".to_string(),
                required: false,
                default: Some(Value::String("22,80,443,3389,8080".to_string())),
                example: Some(Value::String("1-1024".to_string())),
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "firewall_check",
            "parameters": {
                "target": "example.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "Firewall Check Results for example.com:\n\nFirewall Detected: Yes\nBehavior: Filtered - Most ports show no response\nOpen Ports: 80, 443\nFiltered Ports: 22, 3389, 8080".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let target = get_param_string(parameters, "target")?;
        let ports_spec = parameters
            .get("ports")
            .and_then(|v| v.as_str())
            .unwrap_or("22,80,443,3389,8080");
        let timeout_secs = get_param_u64(parameters, "timeout", 3);

        let ip = resolve_host(&target)?;
        let ports = parse_ports(ports_spec)?;

        let semaphore = Arc::new(Semaphore::new(50));
        let mut tasks = vec![];

        for port in ports {
            let permit = semaphore.clone().acquire_owned().await?;
            let target_ip = ip;
            let timeout_dur = Duration::from_secs(timeout_secs);

            tasks.push(tokio::spawn(async move {
                let (tcp_result, udp_result) = check_port(target_ip, port, timeout_dur).await;
                drop(permit);
                (port, target_ip, tcp_result, udp_result)
            }));
        }

        let mut open_ports = Vec::new();
        let mut filtered_ports = Vec::new();
        let mut closed_ports = Vec::new();

        for task in tasks {
            if let Ok((port, target_ip, tcp, udp)) = task.await {
                match (tcp, udp) {
                    (true, _) => open_ports.push(port),
                    (false, true) => open_ports.push(port),
                    (false, false) => {
                        if port_is_filtered(target_ip, port, timeout_secs).await {
                            filtered_ports.push(port);
                        } else {
                            closed_ports.push(port);
                        }
                    }
                }
            }
        }

        let firewall_detected = !filtered_ports.is_empty() && open_ports.is_empty();

        let mut output = format!("Firewall Check Results for {}:\n", target);
        output.push_str(&format!(
            "\nFirewall Detected: {}\n",
            if firewall_detected { "Yes" } else { "No" }
        ));

        if !open_ports.is_empty() {
            output.push_str(&format!(
                "Open Ports: {}\n",
                open_ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !filtered_ports.is_empty() {
            output.push_str(&format!(
                "Filtered Ports: {}\n",
                filtered_ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !closed_ports.is_empty() && !firewall_detected {
            output.push_str(&format!(
                "Closed Ports: {}\n",
                closed_ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(output)
    }
}

async fn check_port(ip: std::net::IpAddr, port: u16, timeout_dur: Duration) -> (bool, bool) {
    let tcp_open = check_tcp(ip, port, timeout_dur).await;
    let udp_open = check_udp(ip, port, timeout_dur).await;
    (tcp_open, udp_open)
}

async fn check_tcp(ip: std::net::IpAddr, port: u16, timeout_dur: Duration) -> bool {
    let addr = std::net::SocketAddr::new(ip, port);
    match timeout(timeout_dur, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

async fn check_udp(ip: std::net::IpAddr, port: u16, timeout_dur: Duration) -> bool {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(_) => return false,
    };
    let addr = std::net::SocketAddr::new(ip, port);
    let mut buf = [0u8; 1];
    match timeout(timeout_dur, async {
        let _ = socket.send_to(&[], &addr).await;
        socket.recv_from(&mut buf).await
    })
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

async fn port_is_filtered(ip: std::net::IpAddr, port: u16, timeout_secs: u64) -> bool {
    let timeout_dur = Duration::from_secs(timeout_secs);
    let addr = std::net::SocketAddr::new(ip, port);
    match timeout(timeout_dur, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => false,
        Ok(Err(e)) => e.kind() != std::io::ErrorKind::ConnectionRefused,
        Err(_) => true,
    }
}

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
