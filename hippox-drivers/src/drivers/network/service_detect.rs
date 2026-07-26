//! Service detection driver
//!
//! This driver provides functionality to detect services and versions running on open ports using banner grabbing.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    common::net::{get_probe_for_port, identify_service, parse_ports, resolve_host, tcp_connect},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};
/// Driver for detecting services
#[derive(Debug)]
pub struct ServiceDetectDriver;
#[async_trait::async_trait]
impl Driver for ServiceDetectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "service_detect";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Detect services and versions running on open ports using banner grabbing";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to identify services, versions, and software running on open ports";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_detect",
            "parameters": {
                "target": "google.com",
                "ports": "80,443"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Service Detection Results:\n\nPort 80: HTTP (nginx/1.18.0) [Confidence: 95%]\nPort 443: HTTPS (nginx/1.18.0) [Confidence: 90%]"
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
        debug!("Executing service_detect driver");
        let target = get_param_string(parameters, "target")?;
        let ports_spec = get_param_string(parameters, "ports")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 5);
        let banner_size = get_param_u64(parameters, "banner_size", 4096) as usize;
        info!("Service detection: target={}, ports={}, timeout={}s", target, ports_spec, timeout_secs);
        let ip = resolve_host(&target).map_err(|e| {
            let err_msg = format!("Failed to resolve host: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        let ports = parse_ports(&ports_spec).map_err(|e| {
            let err_msg = format!("Failed to parse ports: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        info!("Detecting services on {} ports", ports.len());
        let mut results = Vec::new();
        for port in ports {
            let result = detect_service(ip, port, timeout_secs, banner_size).await;
            results.push(result);
        }
        let mut output = format!("Service Detection Results for {}:\n", target);
        let results_size = results.len();
        for (port, service, version, confidence) in results {
            output.push_str(&format!("\nPort {}: {} ", port, service));
            if let Some(v) = version {
                output.push_str(&format!("({}) ", v));
            }
            output.push_str(&format!("[Confidence: {}%]", confidence));
        }
        info!("Service detection complete: {} services detected", results_size);
        return Ok(output);
    }
}
/// Detects service on a specific port
async fn detect_service(ip: std::net::IpAddr, port: u16, timeout_secs: u64, banner_size: usize) -> (u16, String, Option<String>, u8) {
    debug!("Detecting service on port {}:{}", ip, port);
    let timeout_dur = Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, async {
        // Convert DriverError to String
        let mut stream = match tcp_connect(ip, port, timeout_secs).await {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!("Failed to connect: {}", e);
                warn!("{}", err_msg);
                return Err::<_, String>(err_msg);
            }
        };
        let probe = get_probe_for_port(port);
        if let Some(data) = probe {
            let _ = stream.write_all(data).await;
        }
        let mut buffer = vec![0u8; banner_size];
        let read_timeout = Duration::from_secs(3);
        // Handle timeout and read errors separately
        let n = match tokio::time::timeout(read_timeout, stream.read(&mut buffer)).await {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => {
                let err_msg = format!("Failed to read banner: {}", e);
                warn!("{}", err_msg);
                return Err::<_, String>(err_msg);
            }
            Err(_) => {
                let err_msg = format!("Banner read timeout on port {}", port);
                warn!("{}", err_msg);
                return Err::<_, String>(err_msg);
            }
        };
        let banner = String::from_utf8_lossy(&buffer[..n]).to_string();
        info!("Banner received on port {}: {} bytes", port, n);
        let (service, version, confidence) = identify_service(port, &banner);
        return Ok::<_, String>((port, service, version, confidence));
    })
    .await
    {
        Ok(Ok(result)) => {
            info!("Service detected on port {}: {:?}", port, result);
            return result;
        }
        Ok(Err(e)) => {
            warn!("Error detecting service on port {}: {}", port, e);
            return (port, "Closed".to_string(), None, 0);
        }
        Err(_) => {
            warn!("Timeout detecting service on port {}", port);
            return (port, "Closed".to_string(), None, 0);
        }
    }
}
/// Gets a string parameter from the parameters map
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    return params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| {
        let err_msg = format!("Missing parameter: {}", name);
        warn!("{}", err_msg);
        return DriverError::missing_parameter(name);
    });
}
/// Gets a u64 parameter from the parameters map with a default value
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    let value = params.get(name).and_then(|v| v.as_u64()).unwrap_or(default);
    debug!("Parameter {}: {}", name, value);
    return value;
}
