//! Universal TCP server skill - A flexible TCP server that provides basic communication infrastructure
//!
//! This skill starts a TCP server that listens on a specified port and handles client connections.
//! It provides the fundamental communication layer while allowing LLMs to define the actual
//! protocol behavior through parameters. The server can be configured with custom responses,
//! prefixes, suffixes, and delimiters to simulate various protocols.
//!
//! This is a generic server infrastructure skill that gives LLMs full control over the
//! communication protocol. LLMs can define the data structure, response format, and
//! protocol behavior using the provided parameters.
//!
//! # Examples
//!
//! Simple TCP server with custom response:
//! ```json
//! {
//!     "action": "tcp_universal_server",
//!     "parameters": {
//!         "port": 8080,
//!         "response": "HTTP/1.1 200 OK\r\n\r\nHello World"
//!     }
//! }
//! ```
//!
//! Protocol simulation with prefix/suffix:
//! ```json
//! {
//!     "action": "tcp_universal_server",
//!     "parameters": {
//!         "port": 9999,
//!         "response_prefix": "DATA: ",
//!         "response_suffix": " [END]",
//!         "delimiter": "\\n"
//!     }
//! }
//! ```

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Debug)]
pub struct TcpUniversalServerSkill;

#[async_trait::async_trait]
impl Skill for TcpUniversalServerSkill {
    fn name(&self) -> &str {
        "tcp_universal_server"
    }

    fn description(&self) -> &str {
        "Start a universal TCP server that listens on a port and provides basic communication infrastructure. You can define the protocol behavior through parameters."
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to start a TCP server to receive and respond to client connections. This is a universal server infrastructure that you can configure to implement any protocol by specifying response format, prefixes, suffixes, and delimiters. The server runs continuously until cancelled."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Port number to listen on (1-65535)".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(8080.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "bind_address".to_string(),
                param_type: "string".to_string(),
                description: "Address to bind to (default: 0.0.0.0 for all interfaces, use 127.0.0.1 for localhost only)".to_string(),
                required: false,
                default: Some(Value::String("0.0.0.0".to_string())),
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "response".to_string(),
                param_type: "string".to_string(),
                description: "Fixed response to send back to client for every request. You can use this to implement any protocol response (HTTP, SMTP, custom binary, etc.)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("+OK\r\n".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "response_prefix".to_string(),
                param_type: "string".to_string(),
                description: "Prefix to prepend to the response. Useful for wrapping client data with protocol headers.".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "response_suffix".to_string(),
                param_type: "string".to_string(),
                description: "Suffix to append to the response. Useful for adding protocol trailers or terminators.".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("\r\n\r\n".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "Message delimiter to append: \\n (newline), \\r\\n (CRLF), \\0 (null), or none".to_string(),
                required: false,
                default: Some(Value::String("\\n".to_string())),
                example: Some(Value::String("\\r\\n".to_string())),
                enum_values: Some(vec![
                    "\\n".to_string(),
                    "\\r\\n".to_string(),
                    "\\0".to_string(),
                    "none".to_string(),
                ]),
            },
            SkillParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Output encoding for the response: utf8 (plain text), hex (hexadecimal), base64".to_string(),
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
                name: "buffer_size".to_string(),
                param_type: "integer".to_string(),
                description: "Buffer size in bytes for reading data from clients (default: 4096)".to_string(),
                required: false,
                default: Some(Value::Number(4096.into())),
                example: Some(Value::Number(8192.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_connections".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of concurrent client connections (default: 100)".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "log_requests".to_string(),
                param_type: "boolean".to_string(),
                description: "Log each client request (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "tcp_universal_server",
            "parameters": {
                "port": 8080,
                "response": "OK",
                "delimiter": "\\n"
            }
        })
    }

    fn example_output(&self) -> String {
        "Universal TCP server started on 0.0.0.0:8080\nServer is running and accepting connections. Use cancel_task() to stop the server.".to_string()
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
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting universal TCP server".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(5), None);
        }

        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: port"))?
            as u16;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Port: {}", port)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        let bind_address = parameters
            .get("bind_address")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0.0");

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Bind address: {}", bind_address)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(15), None);
        }
        let response = parameters
            .get("response")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(cb) = cb {
            if let Some(r) = &response {
                let preview = if r.len() > 50 {
                    format!("{}...", &r[..50])
                } else {
                    r.clone()
                };
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Response: {}", preview)),
                );
            } else {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some("Response: echo mode (no fixed response)".to_string()),
                );
            }
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }
        let response_prefix = parameters
            .get("response_prefix")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(cb) = cb {
            if let Some(p) = &response_prefix {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Response prefix: {}", p)),
                );
            }
            cb.on_progress(task_id.clone(), skill_index, Some(25), None);
        }
        let response_suffix = parameters
            .get("response_suffix")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(cb) = cb {
            if let Some(s) = &response_suffix {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Response suffix: {}", s)),
                );
            }
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let delimiter = parameters
            .get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or("\\n");
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Delimiter: {}", delimiter)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(35), None);
        }
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf8");
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Encoding: {}", encoding)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }
        let buffer_size = parameters
            .get("buffer_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096) as usize;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Buffer size: {} bytes", buffer_size)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(45), None);
        }
        let max_connections = parameters
            .get("max_connections")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Max connections: {}", max_connections)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }
        let log_requests = parameters
            .get("log_requests")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Log requests: {}", log_requests)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(55), None);
        }
        let addr = format!("{}:{}", bind_address, port);
        let listener = TcpListener::bind(&addr).await?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Universal TCP server started on {}", addr)),
            );
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(
                    "Server is running and accepting connections. Use cancel_task() to stop it."
                        .to_string(),
                ),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        let connection_counter = Arc::new(Semaphore::new(max_connections));
        loop {
            if let Some(cb) = cb {
                if let Some(context) = context {
                    if let Some(task_id_str) = context.task_id() {
                        if let Some(updater) = crate::tasks::get_state_updater(task_id_str).await {
                            if updater.is_cancelled().await {
                                cb.on_log(
                                    task_id.clone(),
                                    skill_index,
                                    Some(
                                        "Received cancellation signal, shutting down..."
                                            .to_string(),
                                    ),
                                );
                                break;
                            }
                        }
                    }
                }
            }
            let permit = connection_counter.clone().acquire_owned().await?;
            let (mut stream, client_addr) = listener.accept().await?;
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Client connected from {}", client_addr)),
                );
            }
            let response_clone = response.clone();
            let response_prefix_clone = response_prefix.clone();
            let response_suffix_clone = response_suffix.clone();
            let delimiter_clone = delimiter.to_string();
            let encoding_clone = encoding.to_string();
            let buffer_size_clone = buffer_size;
            let cb_clone = cb;
            let task_id_clone = task_id.clone();
            let skill_index_clone = skill_index;
            let log_requests_clone = log_requests;
            tokio::spawn(async move {
                let mut buf = vec![0u8; buffer_size_clone];
                let _permit = permit;
                loop {
                    match stream.read(&mut buf).await {
                        Ok(0) => {
                            if let Some(cb) = cb_clone {
                                cb.on_log(
                                    task_id_clone.clone(),
                                    skill_index_clone,
                                    Some(format!("Client {} disconnected", client_addr)),
                                );
                            }
                            break;
                        }
                        Ok(n) => {
                            let received = &buf[..n];
                            if log_requests_clone {
                                if let Some(cb) = cb_clone {
                                    let preview = if n > 100 {
                                        format!("{}...", String::from_utf8_lossy(&received[..100]))
                                    } else {
                                        String::from_utf8_lossy(received).to_string()
                                    };
                                    cb.on_log(
                                        task_id_clone.clone(),
                                        skill_index_clone,
                                        Some(format!(
                                            "Received {} bytes from {}: {}",
                                            n, client_addr, preview
                                        )),
                                    );
                                }
                            }
                            let response_data = build_response(
                                received,
                                &response_clone,
                                &response_prefix_clone,
                                &response_suffix_clone,
                                &delimiter_clone,
                                &encoding_clone,
                            );
                            if let Some(cb) = cb_clone {
                                cb.on_log(
                                    task_id_clone.clone(),
                                    skill_index_clone,
                                    Some(format!(
                                        "Sending {} bytes response to {}",
                                        response_data.len(),
                                        client_addr
                                    )),
                                );
                            }
                            if let Err(e) = stream.write_all(&response_data).await {
                                if let Some(cb) = cb_clone {
                                    cb.on_log(
                                        task_id_clone.clone(),
                                        skill_index_clone,
                                        Some(format!("Failed to send response: {}", e)),
                                    );
                                }
                                break;
                            }
                        }
                        Err(e) => {
                            if let Some(cb) = cb_clone {
                                cb.on_log(
                                    task_id_clone.clone(),
                                    skill_index_clone,
                                    Some(format!("Connection error from {}: {}", client_addr, e)),
                                );
                            }
                            break;
                        }
                    }
                }
            });
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Server stopped".to_string()),
            );
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("tcp_universal_server".to_string()),
                Some("Server stopped".to_string()),
            );
        }
        Ok("Server stopped".to_string())
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: port"))?;
        Ok(())
    }
}

fn build_response(
    data: &[u8],
    response: &Option<String>,
    prefix: &Option<String>,
    suffix: &Option<String>,
    delimiter: &str,
    encoding: &str,
) -> Vec<u8> {
    let text = match encoding {
        "hex" => hex::encode(data),
        "base64" => STANDARD.encode(data),
        _ => String::from_utf8_lossy(data).to_string(),
    };
    let mut result = if let Some(fixed) = response {
        fixed.clone()
    } else {
        let text_trimmed = text.trim_end_matches('\n').trim_end_matches('\r');
        format!(
            "{}{}{}",
            prefix.clone().unwrap_or_default(),
            text_trimmed,
            suffix.clone().unwrap_or_default()
        )
    };

    match delimiter {
        "\\n" => result.push('\n'),
        "\\r\\n" => {
            result.push('\r');
            result.push('\n');
        }
        "\\0" => result.push('\0'),
        "none" => {}
        _ => {}
    }
    result.into_bytes()
}
