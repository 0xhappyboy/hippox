use crate::SkillCallback;
use crate::SkillContext;
use crate::get_param_bool;
use crate::get_param_string;
use crate::get_param_u64;
use crate::{Skill, SkillCategory, SkillParameter};
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// TCP Send Skill
#[derive(Debug)]
pub struct TcpSendSkill;

#[async_trait::async_trait]
impl Skill for TcpSendSkill {
    fn name(&self) -> &str {
        "tcp_send"
    }

    fn description(&self) -> &str {
        "Connect to a TCP server, send data ONCE, optionally read response ONCE, then close."
    }

    fn usage_hint(&self) -> &str {
        "Single-shot TCP sender. Connects, sends data, optionally waits for one response, then closes. For multiple exchanges, call this skill repeatedly."
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
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
                example: Some(Value::Number(8080.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, Server!".to_string())),
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
                description: "Connection and send timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "Optional delimiter to append (\\n, \\r\\n)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("\\n".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "wait_response".to_string(),
                param_type: "boolean".to_string(),
                description: "Wait for server response after sending".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "response_timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout for waiting response in seconds".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "response_buffer".to_string(),
                param_type: "integer".to_string(),
                description: "Buffer size for response".to_string(),
                required: false,
                default: Some(Value::Number(4096.into())),
                example: Some(Value::Number(8192.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "tcp_send", "parameters": { "host": "127.0.0.1", "port": 8080, "data": "Hello", "wait_response": true } })
    }

    fn example_output(&self) -> String {
        "Successfully sent 5 bytes to 127.0.0.1:8080\nResponse: ACK".to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 0) as u16;
        let data_str = get_param_string(parameters, "data")?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        let delimiter = parameters
            .get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let wait_response = get_param_bool(parameters, "wait_response", false);
        let response_timeout = get_param_u64(parameters, "response_timeout", 10);
        let response_buffer = get_param_u64(parameters, "response_buffer", 4096) as usize;
        // Decode data
        let data = match encoding {
            "hex" => hex::decode(data_str)?,
            "base64" => STANDARD.decode(data_str)?,
            _ => data_str.as_bytes().to_vec(),
        };
        // Handle delimiter
        let delimiter_bytes = match delimiter {
            "\\n" => "\n".as_bytes(),
            "\\r\\n" => "\r\n".as_bytes(),
            "\\r" => "\r".as_bytes(),
            _ => delimiter.as_bytes(),
        };
        let final_data = if !delimiter_bytes.is_empty() {
            [data.as_slice(), delimiter_bytes].concat()
        } else {
            data
        };
        // Connect and send
        let addr = format!("{}:{}", host, port);
        let connection = timeout(
            std::time::Duration::from_secs(timeout_secs),
            TcpStream::connect(&addr),
        )
        .await??;
        let mut stream = connection;
        let bytes_sent = timeout(std::time::Duration::from_secs(timeout_secs), async {
            stream.write_all(&final_data).await?;
            Ok::<_, anyhow::Error>(final_data.len())
        })
        .await??;
        let mut result = format!(
            "Successfully sent {} bytes to {}:{}",
            bytes_sent, host, port
        );
        // Wait for response if requested
        if wait_response {
            let mut buffer = vec![0u8; response_buffer];
            let read_result = timeout(
                std::time::Duration::from_secs(response_timeout),
                stream.read(&mut buffer),
            )
            .await??;
            let response = String::from_utf8_lossy(&buffer[..read_result]);
            result.push_str(&format!("\nResponse: {}", response));
        }
        Ok(result)
    }
}
