//! SSH execution driver
//!
//! This driver provides functionality to execute a command on a remote host via SSH.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use ssh2::Session;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};
/// Driver for executing commands via SSH
#[derive(Debug)]
pub struct SshExecDriver;
#[async_trait::async_trait]
impl Driver for SshExecDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "ssh_exec";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute a command on a remote host via SSH";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to run commands on a remote server via SSH. Requires authentication (password or key).";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Remote hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("192.168.1.100".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "SSH port (default: 22)".to_string(),
                required: false,
                default: Some(Value::Number(22.into())),
                example: Some(Value::Number(2222.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "SSH username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("root".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "SSH password (optional if key provided)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("secret123".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "key_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to SSH private key (optional if password provided)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/home/user/.ssh/id_rsa".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to execute on remote host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ls -la /var/log".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection and execution timeout in seconds (default: 30)".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "ssh_exec",
            "parameters": {
                "host": "192.168.1.100",
                "username": "root",
                "password": "secret123",
                "command": "uptime"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Command executed successfully (exit code: 0)\nstdout: 10:30:00 up 5 days, 2 users, load average: 0.5\nstderr: ".to_string();
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
        debug!("Executing ssh_exec driver");
        // Extract all parameters into owned values before spawn_blocking
        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'host' parameter");
                return DriverError::missing_parameter("host");
            })?
            .to_string();
        let port = parameters.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
        let username = parameters
            .get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'username' parameter");
                return DriverError::missing_parameter("username");
            })?
            .to_string();
        // Convert to owned String to avoid lifetime issues
        let password = parameters.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
        let key_path = parameters.get("key_path").and_then(|v| v.as_str()).map(|s| s.to_string());
        let command = parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'command' parameter");
                return DriverError::missing_parameter("command");
            })?
            .to_string();
        let timeout_secs = parameters.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
        info!("SSH exec: host={}, port={}, username={}, timeout={}s", host, port, username, timeout_secs);
        if password.is_none() && key_path.is_none() {
            debug!("Either password or key_path must be provided");
            return Err(DriverError::execution("Either password or key_path must be provided"));
        }
        // Use tokio::task::spawn_blocking for blocking SSH operations
        let result = tokio::task::spawn_blocking(move || {
            let timeout_dur = Duration::from_secs(timeout_secs);
            // Connect with timeout
            let addr = format!("{}:{}", host, port);
            let stream = TcpStream::connect_timeout(&addr.parse().map_err(|e| format!("Invalid address: {}", e))?, timeout_dur)
                .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
            let mut session = Session::new().map_err(|e| format!("Failed to create SSH session: {}", e))?;
            session.set_tcp_stream(stream);
            session.handshake().map_err(|e| format!("SSH handshake failed: {}", e))?;
            // Authenticate
            if let Some(pass) = password.as_deref() {
                session.userauth_password(&username, pass).map_err(|e| format!("Password authentication failed: {}", e))?;
            } else if let Some(key_path_str) = key_path.as_deref() {
                let key_path = Path::new(key_path_str);
                if !key_path.exists() {
                    return Err(format!("Private key file not found: {}", key_path_str));
                }
                session.userauth_pubkey_file(&username, None, key_path, None).map_err(|e| format!("Public key authentication failed: {}", e))?;
            } else {
                return Err("No authentication method provided".to_string());
            }
            if !session.authenticated() {
                return Err("Authentication failed".to_string());
            }
            // Execute command
            let mut channel = session.channel_session().map_err(|e| format!("Failed to open channel: {}", e))?;
            channel.exec(&command).map_err(|e| format!("Failed to execute command: {}", e))?;
            // Read stdout
            let mut stdout = String::new();
            let mut stdout_buf = [0u8; 4096];
            loop {
                match channel.read(&mut stdout_buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        stdout.push_str(&String::from_utf8_lossy(&stdout_buf[..n]));
                    }
                    Err(_) => break,
                }
            }
            // Read stderr
            let mut stderr = String::new();
            let mut stderr_buf = [0u8; 4096];
            let mut stderr_channel = channel.stderr();
            loop {
                match stderr_channel.read(&mut stderr_buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        stderr.push_str(&String::from_utf8_lossy(&stderr_buf[..n]));
                    }
                }
            }
            let exit_code = channel.exit_status().map_err(|e| format!("Failed to get exit status: {}", e))?;
            channel.close().map_err(|e| format!("Failed to close channel: {}", e))?;
            session.disconnect(None, "Goodbye", None).map_err(|e| format!("Failed to disconnect: {}", e))?;
            return Ok::<_, String>((exit_code, stdout, stderr));
        })
        .await
        .map_err(|e| {
            debug!("Task panicked: {}", e);
            return DriverError::execution(format!("Task panicked: {}", e));
        })?
        .map_err(|e| {
            debug!("SSH execution failed: {}", e);
            return DriverError::execution(e);
        })?;
        let (exit_code, stdout, stderr) = result;
        info!("SSH exec complete: exit_code={}, stdout_len={}, stderr_len={}", exit_code, stdout.len(), stderr.len());
        return Ok(format!(
            "Command executed successfully (exit code: {})\nstdout: {}\nstderr: {}",
            exit_code,
            if stdout.is_empty() { "(empty)" } else { &stdout },
            if stderr.is_empty() { "(empty)" } else { &stderr }
        ));
    }
}
