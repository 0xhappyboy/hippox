//! Service detection skill

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    common::net::{get_probe_for_port, identify_service, parse_ports, resolve_host, tcp_connect},
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct ServiceDetectDriver;

#[async_trait::async_trait]
impl Driver for ServiceDetectDriver {
    fn name(&self) -> &str {
        "service_detect"
    }

    fn description(&self) -> &str {
        "Detect services and versions running on open ports using banner grabbing"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to identify services, versions, and software running on open ports"
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
                description: "Ports to detect (comma-separated or range)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("22,80,443".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "banner_size".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum banner bytes to read".to_string(),
                required: false,
                default: Some(Value::Number(4096.into())),
                example: Some(Value::Number(8192.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_detect",
            "parameters": {
                "target": "google.com",
                "ports": "80,443"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service Detection Results:\n\nPort 80: HTTP (nginx/1.18.0) [Confidence: 95%]\nPort 443: HTTPS (nginx/1.18.0) [Confidence: 90%]".to_string()
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
        let ports_spec = get_param_string(parameters, "ports")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 5);
        let banner_size = get_param_u64(parameters, "banner_size", 4096) as usize;
        let ip = resolve_host(&target)?;
        let ports = parse_ports(&ports_spec)?;
        let mut results = Vec::new();
        for port in ports {
            let result = detect_service(ip, port, timeout_secs, banner_size).await;
            results.push(result);
        }
        let mut output = format!("Service Detection Results for {}:\n", target);
        for (port, service, version, confidence) in results {
            output.push_str(&format!("\nPort {}: {} ", port, service));
            if let Some(v) = version {
                output.push_str(&format!("({}) ", v));
            }
            output.push_str(&format!("[Confidence: {}%]", confidence));
        }
        Ok(output)
    }
}

async fn detect_service(
    ip: std::net::IpAddr,
    port: u16,
    timeout_secs: u64,
    banner_size: usize,
) -> (u16, String, Option<String>, u8) {
    let timeout_dur = Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, async {
        let mut stream = tcp_connect(ip, port, timeout_secs).await?;
        let probe = get_probe_for_port(port);
        if let Some(data) = probe {
            let _ = stream.write_all(data).await;
        }
        let mut buffer = vec![0u8; banner_size];
        let n = tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer)).await??;
        let banner = String::from_utf8_lossy(&buffer[..n]).to_string();
        let (service, version, confidence) = identify_service(port, &banner);
        Ok::<_, anyhow::Error>((port, service, version, confidence))
    })
    .await
    {
        Ok(Ok(result)) => result,
        _ => (port, "Closed".to_string(), None, 0),
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
