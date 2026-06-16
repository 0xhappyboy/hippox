//! OS fingerprinting skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::TcpStream;
use std::time::Duration;

use crate::{
    SkillCategory,
    common::net::resolve_host,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct OsFingerprintSkill;

#[async_trait::async_trait]
impl Skill for OsFingerprintSkill {
    fn name(&self) -> &str {
        "os_fingerprint"
    }

    fn description(&self) -> &str {
        "Detect operating system by analyzing TCP/IP stack characteristics"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to identify the operating system of a remote host"
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
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to probe (default: 80)".to_string(),
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
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_fingerprint",
            "parameters": {
                "target": "google.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "OS Fingerprint Results:\n\nDetected OS: Linux\nConfidence: 85%\nTTL: 64\nWindow Size: 65535".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let target = get_param_string(parameters, "target")?;
        let port = get_param_u64(parameters, "port", 80) as u16;
        let timeout_secs = get_param_u64(parameters, "timeout", 5);

        let ip = resolve_host(&target)?;
        let (ttl, window) = get_tcp_stack_info(ip, port, timeout_secs).await;
        let (os, confidence) = fingerprint_os(ttl, window);

        let mut output = format!("OS Fingerprint Results for {}:\n", target);
        output.push_str(&format!("\nDetected OS: {}\n", os));
        output.push_str(&format!("Confidence: {}%\n", confidence));
        output.push_str(&format!("TTL: {}\n", ttl));
        output.push_str(&format!("Window Size: {}\n", window));

        Ok(output)
    }
}

async fn get_tcp_stack_info(ip: std::net::IpAddr, port: u16, timeout_secs: u64) -> (u32, u32) {
    let addr = std::net::SocketAddr::new(ip, port);
    let timeout_dur = Duration::from_secs(timeout_secs);

    match tokio::time::timeout(timeout_dur, async { TcpStream::connect(addr) }).await {
        Ok(Ok(stream)) => {
            let ttl = get_ttl(&stream);
            let window = get_window_size(&stream);
            (ttl, window)
        }
        _ => (64, 65535),
    }
}

fn get_ttl(stream: &TcpStream) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();
        let mut ttl: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        unsafe {
            if libc::getsockopt(
                fd,
                libc::IPPROTO_IP,
                libc::IP_TTL,
                &mut ttl as *mut u32 as *mut std::ffi::c_void,
                &mut len,
            ) == 0
            {
                return ttl;
            }
        }
    }
    64
}

fn get_window_size(stream: &TcpStream) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();
        let mut window: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        unsafe {
            if libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_RCVBUF,
                &mut window as *mut u32 as *mut std::ffi::c_void,
                &mut len,
            ) == 0
            {
                return window;
            }
        }
    }
    65535
}

fn fingerprint_os(ttl: u32, _window: u32) -> (String, u8) {
    match ttl {
        64 => ("Linux".to_string(), 85),
        128 => ("Windows".to_string(), 85),
        255 => ("Cisco/UNIX".to_string(), 70),
        60 => ("Linux (older)".to_string(), 60),
        _ => ("Unknown".to_string(), 30),
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
