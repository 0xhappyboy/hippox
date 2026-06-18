use crate::SkillCallback;
use crate::SkillContext;
use crate::get_param_u64;
use crate::{Skill, SkillCategory, SkillParameter};
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;

/// TCP Receive Skill
#[derive(Debug)]
pub struct TcpReceiveSkill;

#[async_trait::async_trait]
impl Skill for TcpReceiveSkill {
    fn name(&self) -> &str {
        "tcp_receive"
    }

    fn description(&self) -> &str {
        "Accept ONE TCP connection, read ONCE up to buffer_size bytes, return data, then close. Excess data is truncated."
    }

    fn usage_hint(&self) -> &str {
        "Single-shot receiver. Reads once, max buffer_size bytes (default 4096). For larger data, increase buffer_size or call repeatedly."
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to listen on".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(8888.into())),
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
                description: "Wait timeout in seconds".to_string(),
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
                description: "Optional response to send back to client".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ACK".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "tcp_receive", "parameters": { "port": 8888, "timeout": 10 } })
    }

    fn example_output(&self) -> String {
        "Received 42 bytes from 127.0.0.1:54321:\nHello, TCP Server!\nResponse sent: OK".to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
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
        let listener = TcpListener::bind(&addr).await?;
        // Accept connection with timeout
        let accept_result = timeout(
            std::time::Duration::from_secs(timeout_secs),
            listener.accept(),
        )
        .await??;
        let (mut stream, client_addr) = accept_result;
        // Read data
        let mut buffer = vec![0u8; buffer_size];
        let read_result = timeout(
            std::time::Duration::from_secs(timeout_secs),
            stream.read(&mut buffer),
        )
        .await??;
        let received_data = &buffer[..read_result];
        // Encode output
        let output = match encoding {
            "hex" => hex::encode(received_data),
            "base64" => STANDARD.encode(received_data),
            _ => String::from_utf8_lossy(received_data).to_string(),
        };
        let mut result = format!(
            "Received {} bytes from {}:\n{}",
            read_result, client_addr, output
        );
        // Send response if requested
        if let Some(response) = send_response {
            stream.write_all(response.as_bytes()).await?;
            result.push_str(&format!("\nResponse sent: {}", response));
        }
        Ok(result)
    }
}
