//! Port scanning skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    common::net::{get_service_name, parse_ports, resolve_host},
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[derive(Debug)]
pub struct PortScanSkill;

#[async_trait::async_trait]
impl Skill for PortScanSkill {
    fn name(&self) -> &str {
        "port_scan"
    }

    fn description(&self) -> &str {
        "Scan ports on a target host to discover open ports and services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find open ports, check service availability, or perform network reconnaissance"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
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
                example: Some(Value::String("22,80,443".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(3.into())),
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "port_scan",
            "parameters": {
                "target": "localhost",
                "ports": "1-1000"
            }
        })
    }

    fn example_output(&self) -> String {
        "Scanning localhost (127.0.0.1)\nPort 22: Open - SSH\nPort 80: Open - HTTP\nPort 443: Open - HTTPS\nTotal open ports: 3\nScan completed in 2.5 seconds".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let target = get_param_string(parameters, "target")?;
        let ports_spec = parameters
            .get("ports")
            .and_then(|v| v.as_str())
            .unwrap_or("1-1024");
        let timeout_secs = get_param_u64(parameters, "timeout", 2);
        let concurrency = get_param_u64(parameters, "concurrency", 100) as usize;

        let ip = resolve_host(&target)?;
        let ports = parse_ports(ports_spec)?;
        let total_ports = ports.len();
        let start_time = std::time::Instant::now();

        let semaphore = Arc::new(Semaphore::new(concurrency));
        let mut tasks = vec![];

        for port in ports {
            let permit = semaphore.clone().acquire_owned().await?;
            let target_ip = ip;
            let timeout_dur = Duration::from_secs(timeout_secs);

            tasks.push(tokio::spawn(async move {
                let is_open = scan_port(target_ip, port, timeout_dur).await;
                drop(permit);
                (port, is_open)
            }));
        }

        let mut open_ports = Vec::new();
        for task in tasks {
            if let Ok((port, true)) = task.await {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        let duration = start_time.elapsed();

        let mut result = format!("Scanning {} ({})\n", target, ip);
        result.push_str(&format!("Total ports scanned: {}\n", total_ports));

        if !open_ports.is_empty() {
            result.push_str(&format!("\nOpen ports: {}\n", open_ports.len()));
            for port in &open_ports {
                result.push_str(&format!(
                    "  Port {}: Open - {}\n",
                    port,
                    get_service_name(*port)
                ));
            }
        } else {
            result.push_str("\nNo open ports found\n");
        }

        result.push_str(&format!(
            "\nScan completed in {:.2} seconds",
            duration.as_secs_f64()
        ));
        Ok(result)
    }
}

async fn scan_port(ip: std::net::IpAddr, port: u16, timeout_dur: Duration) -> bool {
    let addr = std::net::SocketAddr::new(ip, port);
    match timeout(timeout_dur, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
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
