use crate::config::{get_tcp_instance, list_tcp_instances};
use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
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
        "Send data over TCP connection"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to send raw data over TCP socket to a server"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_tcp_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "TCP instance ID (use 'list_tcp_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("tcp_server1".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address (overrides instance config)"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Target port number (overrides instance config)".to_string(),
                required: false,
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
        json!({
            "action": "tcp_send",
            "parameters": {
                "instance_id": "tcp_server1",
                "data": "Hello, TCP Server!",
                "wait_response": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully sent 18 bytes to localhost:9999\nResponse: Server ACK [instance: tcp_server1]"
            .to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());

        let instance = if let Some(id) = instance_id {
            get_tcp_instance(id).ok_or_else(|| anyhow::anyhow!("TCP instance not found: {}", id))?
        } else {
            let instances = list_tcp_instances();
            instances.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!("No TCP instance configured. Please add a TCP instance first.")
            })?
        };

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());
        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(instance.port.into()) as u16;
        let data_str = parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data"))?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(instance.timeout);
        let delimiter = parameters
            .get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let wait_response = parameters
            .get("wait_response")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let response_timeout = parameters
            .get("response_timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);
        let response_buffer = parameters
            .get("response_buffer")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096) as usize;

        let data = match encoding {
            "hex" => hex::decode(data_str)?,
            "base64" => STANDARD.decode(data_str)?,
            _ => data_str.as_bytes().to_vec(),
        };

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
            "Successfully sent {} bytes to {}:{} [instance: {}]",
            bytes_sent, host, port, instance.name
        );

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

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data"))?;
        Ok(())
    }
}

/// TCP Receive Skill
#[derive(Debug)]
pub struct TcpReceiveSkill;

#[async_trait::async_trait]
impl Skill for TcpReceiveSkill {
    fn name(&self) -> &str {
        "tcp_receive"
    }

    fn description(&self) -> &str {
        "Receive data from TCP connection (start a simple TCP server)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to listen on a TCP port and receive data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_tcp_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "TCP instance ID (use 'list_tcp_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("tcp_server1".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port to listen on (overrides instance config)".to_string(),
                required: false,
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
        json!({
            "action": "tcp_receive",
            "parameters": {
                "instance_id": "tcp_server1",
                "timeout": 10,
                "send_response": "OK"
            }
        })
    }

    fn example_output(&self) -> String {
        "Received 42 bytes from 127.0.0.1:54321:\nHello, TCP Server!\nResponse sent: OK [instance: tcp_server1]".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());

        let instance = if let Some(id) = instance_id {
            get_tcp_instance(id).ok_or_else(|| anyhow::anyhow!("TCP instance not found: {}", id))?
        } else {
            let instances = list_tcp_instances();
            instances.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!("No TCP instance configured. Please add a TCP instance first.")
            })?
        };

        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(instance.port.into()) as u16;
        let bind_address = parameters
            .get("bind_address")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0.0");
        let buffer_size = parameters
            .get("buffer_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096) as usize;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(instance.timeout);
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        let send_response = parameters.get("send_response").and_then(|v| v.as_str());

        let addr = format!("{}:{}", bind_address, port);
        let listener = TcpListener::bind(&addr).await?;
        let accept_result = timeout(
            std::time::Duration::from_secs(timeout_secs),
            listener.accept(),
        )
        .await??;
        let (mut stream, client_addr) = accept_result;

        let mut buffer = vec![0u8; buffer_size];
        let read_result = timeout(
            std::time::Duration::from_secs(timeout_secs),
            stream.read(&mut buffer),
        )
        .await??;
        let received_data = &buffer[..read_result];

        let output = match encoding {
            "hex" => hex::encode(received_data),
            "base64" => base64::encode(received_data),
            _ => String::from_utf8_lossy(received_data).to_string(),
        };

        let mut result = format!(
            "Received {} bytes from {}:\n{} [instance: {}]",
            read_result, client_addr, output, instance.name
        );

        if let Some(response) = send_response {
            stream.write_all(response.as_bytes()).await?;
            result.push_str(&format!("\nResponse sent: {}", response));
        }

        Ok(result)
    }

    fn validate(&self, _parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}
