use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::{Skill, SkillParameter};

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

/// UDP Send Skill
#[derive(Debug)]
pub struct UdpSendSkill;

#[async_trait::async_trait]
impl Skill for UdpSendSkill {
    fn name(&self) -> &str {
        "udp_send"
    }
    fn description(&self) -> &str {
        "Send data over UDP"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to send UDP datagram to a server"
    }
    fn category(&self) -> &str {
        "net"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Target port number".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(9999.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, UDP Server!".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Data encoding (utf8, hex, base64)".to_string(),
                required: false,
                default: Some(Value::String("utf8".to_string())),
                example: Some(Value::String("hex".to_string())),
                enum_values: Some(vec![
                    "utf8".to_string(),
                    "hex".to_string(),
                    "base64".to_string(),
                ]),
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Send timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "udp_send", "parameters": { "host": "127.0.0.1", "port": 9999, "data": "Hello" } })
    }

    fn example_output(&self) -> String {
        "Successfully sent 5 bytes to 127.0.0.1:9999".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 0) as u16;
        let data_str = get_param_string(parameters, "data")?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        let timeout_secs = get_param_u64(parameters, "timeout", 30);

        let data = match encoding {
            "hex" => hex::decode(data_str)?,
            "base64" => STANDARD.decode(data_str)?,
            _ => data_str.as_bytes().to_vec(),
        };

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let addr = format!("{}:{}", host, port);
        let bytes_sent = timeout(
            std::time::Duration::from_secs(timeout_secs),
            socket.send_to(&data, &addr),
        )
        .await??;

        Ok(format!(
            "Successfully sent {} bytes to {}:{}",
            bytes_sent, host, port
        ))
    }
}

/// UDP Receive Skill
#[derive(Debug)]
pub struct UdpReceiveSkill;

#[async_trait::async_trait]
impl Skill for UdpReceiveSkill {
    fn name(&self) -> &str {
        "udp_receive"
    }
    fn description(&self) -> &str {
        "Receive UDP datagram"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to listen for UDP packets"
    }
    fn category(&self) -> &str {
        "net"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to bind and listen on".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(9999.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "bind_address".to_string(),
                param_type: "string".to_string(),
                description: "Address to bind (default: 0.0.0.0)".to_string(),
                required: false,
                default: Some(Value::String("0.0.0.0".to_string())),
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "buffer_size".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum bytes to receive".to_string(),
                required: false,
                default: Some(Value::Number(4096.into())),
                example: Some(Value::Number(8192.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Receive timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Output encoding (utf8, hex, base64)".to_string(),
                required: false,
                default: Some(Value::String("utf8".to_string())),
                example: Some(Value::String("hex".to_string())),
                enum_values: Some(vec![
                    "utf8".to_string(),
                    "hex".to_string(),
                    "base64".to_string(),
                ]),
            },
            SkillParameter {
                name: "send_response".to_string(),
                param_type: "string".to_string(),
                description: "Optional response to send back to sender".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ACK".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "udp_receive", "parameters": { "port": 9999, "timeout": 10 } })
    }

    fn example_output(&self) -> String {
        "Received 11 bytes from 127.0.0.1:54321:\nHello, UDP!".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let port = get_param_u64(parameters, "port", 0) as u16;
        let bind_address = parameters
            .get("bind_address")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0.0");
        let buffer_size = get_param_u64(parameters, "buffer_size", 4096) as usize;
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        let send_response = parameters.get("send_response").and_then(|v| v.as_str());

        let addr = format!("{}:{}", bind_address, port);
        let socket = UdpSocket::bind(&addr).await?;
        let mut buffer = vec![0u8; buffer_size];

        let receive_result = timeout(
            std::time::Duration::from_secs(timeout_secs),
            socket.recv_from(&mut buffer),
        )
        .await??;
        let (size, src_addr) = receive_result;
        let received_data = &buffer[..size];

        let output = match encoding {
            "hex" => hex::encode(received_data),
            "base64" => STANDARD.encode(received_data),
            _ => String::from_utf8_lossy(received_data).to_string(),
        };

        let mut result = format!("Received {} bytes from {}:\n{}", size, src_addr, output);

        if let Some(response) = send_response {
            socket.send_to(response.as_bytes(), src_addr).await?;
            result.push_str(&format!("\nResponse sent: {}", response));
        }

        Ok(result)
    }
}

/// UDP Broadcast Skill
#[derive(Debug)]
pub struct UdpBroadcastSkill;

#[async_trait::async_trait]
impl Skill for UdpBroadcastSkill {
    fn name(&self) -> &str {
        "udp_broadcast"
    }
    fn description(&self) -> &str {
        "Send UDP broadcast message"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to send a broadcast message to all hosts on the network"
    }
    fn category(&self) -> &str {
        "net"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Target port number".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(9999.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to broadcast".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("DISCOVER".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Data encoding (utf8, hex, base64)".to_string(),
                required: false,
                default: Some(Value::String("utf8".to_string())),
                example: Some(Value::String("utf8".to_string())),
                enum_values: Some(vec![
                    "utf8".to_string(),
                    "hex".to_string(),
                    "base64".to_string(),
                ]),
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Send timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "udp_broadcast", "parameters": { "port": 9999, "data": "DISCOVER" } })
    }

    fn example_output(&self) -> String {
        "Successfully broadcasted 7 bytes to port 9999".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let port = get_param_u64(parameters, "port", 0) as u16;
        let data_str = get_param_string(parameters, "data")?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        let timeout_secs = get_param_u64(parameters, "timeout", 30);

        let data = match encoding {
            "hex" => hex::decode(data_str)?,
            "base64" => STANDARD.decode(data_str)?,
            _ => data_str.as_bytes().to_vec(),
        };

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        let broadcast_addr = format!("255.255.255.255:{}", port);
        let bytes_sent = timeout(
            std::time::Duration::from_secs(timeout_secs),
            socket.send_to(&data, &broadcast_addr),
        )
        .await??;

        Ok(format!(
            "Successfully broadcasted {} bytes to port {}",
            bytes_sent, port
        ))
    }
}
