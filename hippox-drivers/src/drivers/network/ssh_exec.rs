use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverCategory, DriverContext, ssh_exec};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SshExecDriver;

#[async_trait::async_trait]
impl Driver for SshExecDriver {
    fn name(&self) -> &str {
        "ssh_exec"
    }

    fn description(&self) -> &str {
        "Execute a command on a remote host via SSH"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to run commands on a remote server via SSH. Requires authentication (password or key)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
                description: "Connection and execution timeout in seconds (default: 30)"
                    .to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "ssh_exec",
            "parameters": {
                "host": "192.168.1.100",
                "username": "root",
                "password": "secret123",
                "command": "uptime"
            }
        })
    }

    fn example_output(&self) -> String {
        "Command executed successfully (exit code: 0)\nstdout: 10:30:00 up 5 days, 2 users, load average: 0.5\nstderr: ".to_string()
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
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting SSH execution".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(5), None);
        }
        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'host' parameter"))?;
        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(22) as u16;
        let username = parameters
            .get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'username' parameter"))?;
        let password = parameters.get("password").and_then(|v| v.as_str());
        let key_path = parameters.get("key_path").and_then(|v| v.as_str());
        let command = parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Host: {}:{}", host, port)),
            );
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Username: {}", username)),
            );
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Command: {}", command)),
            );
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Timeout: {}s", timeout_secs)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        if password.is_none() && key_path.is_none() {
            anyhow::bail!("Either password or key_path must be provided");
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Connecting to remote host...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let result = ssh_exec(
            host,
            port,
            username,
            password,
            key_path,
            command,
            timeout_secs,
        )
        .await?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Exit code: {}", result.exit_code)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        let output = format!(
            "Command executed successfully (exit code: {})\nstdout: {}\nstderr: {}",
            result.exit_code,
            if result.stdout.is_empty() {
                "(empty)"
            } else {
                &result.stdout
            },
            if result.stderr.is_empty() {
                "(empty)"
            } else {
                &result.stderr
            }
        );
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Execution completed".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("ssh_exec".to_string()),
                Some(output.clone()),
            );
        }
        Ok(output)
    }
}
