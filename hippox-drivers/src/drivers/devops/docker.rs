//! Docker container orchestration utilities.
//!
//! This module provides drivers for Docker container operations including
//! listing containers, starting/stopping containers, viewing logs,
//! inspecting containers, and executing commands inside containers.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCategory, DriverError, DriverResult, ExecOptions, exec_async, exec_with_stdin_async};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Retrieves a string parameter from the parameters map
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
///
/// # Returns
/// * `DriverResult<String>` - Parameter value on success
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    return params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name));
}
/// Retrieves a boolean parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `bool` - Parameter value or default
fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    return params.get(name).and_then(|v| v.as_bool()).unwrap_or(default);
}
/// Retrieves a u64 parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `u64` - Parameter value or default
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    return params.get(name).and_then(|v| v.as_u64()).unwrap_or(default);
}
/// Builds Docker execution options
///
/// # Arguments
/// * `docker_host` - Docker host URL
/// * `timeout` - Command timeout in seconds
///
/// # Returns
/// * `ExecOptions` - Configured execution options
fn build_docker_opts(docker_host: Option<&str>, timeout: u64) -> ExecOptions {
    let mut opts = ExecOptions::new().with_timeout(timeout);
    if let Some(host) = docker_host {
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }
    }
    return opts;
}
/// Driver for listing Docker containers
#[derive(Debug)]
pub struct DockerPsDriver;
#[async_trait::async_trait]
impl Driver for DockerPsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docker_ps";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List Docker containers";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to see running containers, check container status, or find container IDs";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (e.g., unix:///var/run/docker.sock)".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "all".to_string(),
                param_type: "boolean".to_string(),
                description: "Show all containers (including stopped)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Filter output (e.g., 'status=exited', 'name=myapp')".to_string(),
                required: false,
                default: None,
                example: Some(json!("status=running")),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format: table, json, or quiet".to_string(),
                required: false,
                default: Some(json!("table")),
                example: Some(json!("json")),
                enum_values: Some(vec!["table".to_string(), "json".to_string(), "quiet".to_string()]),
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Command timeout in seconds".to_string(),
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
            "action": "docker_ps",
            "parameters": {
                "all": true,
                "format": "json"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "CONTAINER ID   IMAGE     COMMAND   STATUS          PORTS     NAMES\nabc123def456   nginx     \"nginx\"   Up 2 hours      80/tcp    web_nginx".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing docker_ps driver");
        // Extract required parameters
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let all = get_param_bool(parameters, "all", false);
        let filter = parameters.get("filter").and_then(|v| v.as_str());
        let format = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("table");
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build docker ps command arguments
        let mut args = vec!["ps"];
        if all {
            args.push("-a");
        }
        if let Some(f) = filter {
            args.push("--filter");
            args.push(f);
        }
        match format {
            "json" => {
                args.push("--format");
                args.push("json");
            }
            "quiet" => {
                args.push("-q");
            }
            _ => {}
        }
        debug!("Executing docker ps with args: {:?}", args);
        let opts = build_docker_opts(docker_host, timeout);
        let result = exec_async("docker", &args, Some(opts)).await.map_err(|e| DriverError::execution(format!("Docker ps failed: {}", e)))?;
        if !result.success {
            return Err(DriverError::execution(format!("Docker ps failed: {}", result.stderr)));
        }
        let output = if format == "json" {
            let containers: Vec<serde_json::Value> = result.stdout.lines().filter_map(|line| serde_json::from_str(line).ok()).collect();
            serde_json::to_string_pretty(&containers).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?
        } else {
            result.stdout
        };
        info!("Docker ps completed successfully");
        return Ok(output);
    }
}
/// Driver for starting or stopping Docker containers
#[derive(Debug)]
pub struct DockerStartStopDriver;
#[async_trait::async_trait]
impl Driver for DockerStartStopDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docker_start_stop";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Start, stop, restart, or pause Docker containers";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to control container lifecycle: start, stop, restart, pause, or unpause";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_container")),
                enum_values: None,
            },
            DriverParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action to perform: start, stop, restart, pause, unpause".to_string(),
                required: true,
                default: None,
                example: Some(json!("restart")),
                enum_values: Some(vec!["start".to_string(), "stop".to_string(), "restart".to_string(), "pause".to_string(), "unpause".to_string()]),
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds for stop (default: 10)".to_string(),
                required: false,
                default: Some(json!(10)),
                example: Some(json!(30)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "docker_start_stop",
            "parameters": {
                "container": "redis",
                "action": "restart"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Container 'redis' restarted successfully".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing docker_start_stop driver");
        // Extract required parameters
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let action = get_param_string(parameters, "action")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 10);
        // Validate action
        let docker_cmd = match action.as_str() {
            "start" => "start",
            "stop" => "stop",
            "restart" => "restart",
            "pause" => "pause",
            "unpause" => "unpause",
            _ => return Err(DriverError::validation("action", format!("Unknown action: {}", action))),
        };
        // Build docker command arguments
        let mut args: Vec<String> = vec![docker_cmd.to_string()];
        if action == "stop" {
            args.push("-t".to_string());
            args.push(timeout_secs.to_string());
        }
        args.push(container.clone());
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        debug!("Executing docker {} on container: {}", docker_cmd, container);
        let opts = build_docker_opts(docker_host, 60);
        let result = exec_async("docker", &args_ref, Some(opts))
            .await
            .map_err(|e| DriverError::execution(format!("Failed to {} container: {}", action, e)))?;
        if !result.success {
            return Err(DriverError::execution(format!("Failed to {} container: {}", action, result.stderr)));
        }
        info!("Container '{}' {}ed successfully", container, action);
        return Ok(format!("Container '{}' {}ed successfully", container, action));
    }
}
/// Driver for viewing Docker container logs
#[derive(Debug)]
pub struct DockerLogsDriver;
#[async_trait::async_trait]
impl Driver for DockerLogsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docker_logs";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "View logs from a Docker container";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to debug container issues, monitor output, or check error logs";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_app")),
                enum_values: None,
            },
            DriverParameter {
                name: "tail".to_string(),
                param_type: "integer".to_string(),
                description: "Number of lines to show from the end".to_string(),
                required: false,
                default: Some(json!(100)),
                example: Some(json!(50)),
                enum_values: None,
            },
            DriverParameter {
                name: "since".to_string(),
                param_type: "string".to_string(),
                description: "Show logs since timestamp (e.g., '2024-01-01T00:00:00Z' or '1h')".to_string(),
                required: false,
                default: None,
                example: Some(json!("1h")),
                enum_values: None,
            },
            DriverParameter {
                name: "follow".to_string(),
                param_type: "boolean".to_string(),
                description: "Follow log output (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "timestamps".to_string(),
                param_type: "boolean".to_string(),
                description: "Show timestamps (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "docker_logs",
            "parameters": {
                "container": "mysql",
                "tail": 20
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "2024-01-15T10:30:00Z [Note] [MY-010914] [Server] Shutdown complete\n2024-01-15T10:30:01Z [System] [MY-010116] [Server] /usr/sbin/mysqld: ready for connections".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing docker_logs driver");
        // Extract required parameters
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let tail = get_param_u64(parameters, "tail", 100);
        let since = parameters.get("since").and_then(|v| v.as_str());
        let follow = get_param_bool(parameters, "follow", false);
        let timestamps = get_param_bool(parameters, "timestamps", false);
        let tail_str = tail.to_string();
        // Build docker logs command arguments
        let mut args = vec!["logs", "--tail", &tail_str];
        if let Some(s) = since {
            args.push("--since");
            args.push(s);
        }
        if follow {
            args.push("--follow");
        }
        if timestamps {
            args.push("--timestamps");
        }
        args.push(&container);
        debug!("Fetching logs for container: {}", container);
        let opts = build_docker_opts(docker_host, 60);
        let result = exec_async("docker", &args, Some(opts)).await.map_err(|e| DriverError::execution(format!("Failed to get logs: {}", e)))?;
        if !result.success {
            return Err(DriverError::execution(format!("Failed to get logs: {}", result.stderr)));
        }
        let output = if result.stdout.is_empty() { "No logs available".to_string() } else { result.stdout };
        info!("Successfully retrieved logs for container: {}", container);
        return Ok(output);
    }
}
/// Driver for getting detailed information about a Docker container
#[derive(Debug)]
pub struct DockerInspectDriver;
#[async_trait::async_trait]
impl Driver for DockerInspectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docker_inspect";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get detailed JSON information about a Docker container";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need detailed container configuration, network settings, or mount information";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_container")),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Go template format for output".to_string(),
                required: false,
                default: None,
                example: Some(json!("{{.Name}} {{.State.Status}}")),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "docker_inspect",
            "parameters": {
                "container": "nginx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Detailed JSON container configuration".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing docker_inspect driver");
        // Extract required parameters
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let format = parameters.get("format").and_then(|v| v.as_str());
        // Build docker inspect command arguments
        let mut args = vec!["inspect"];
        if let Some(f) = format {
            args.push("--format");
            args.push(f);
        }
        args.push(&container);
        debug!("Inspecting container: {}", container);
        let opts = build_docker_opts(docker_host, 30);
        let result =
            exec_async("docker", &args, Some(opts)).await.map_err(|e| DriverError::execution(format!("Failed to inspect container: {}", e)))?;
        if !result.success {
            return Err(DriverError::execution(format!("Failed to inspect container: {}", result.stderr)));
        }
        info!("Successfully inspected container: {}", container);
        return Ok(result.stdout);
    }
}
/// Driver for executing commands in a running Docker container
#[derive(Debug)]
pub struct DockerExecDriver;
#[async_trait::async_trait]
impl Driver for DockerExecDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docker_exec";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute a command inside a running Docker container";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to run commands inside containers for debugging, maintenance, or automation";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_app")),
                enum_values: None,
            },
            DriverParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to execute".to_string(),
                required: true,
                default: None,
                example: Some(json!("ls -la")),
                enum_values: None,
            },
            DriverParameter {
                name: "interactive".to_string(),
                param_type: "boolean".to_string(),
                description: "Keep STDIN open (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "tty".to_string(),
                param_type: "boolean".to_string(),
                description: "Allocate a pseudo-TTY (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "workdir".to_string(),
                param_type: "string".to_string(),
                description: "Working directory inside the container".to_string(),
                required: false,
                default: None,
                example: Some(json!("/app")),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "docker_exec",
            "parameters": {
                "container": "mysql",
                "command": "mysql -e 'SHOW DATABASES'"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Database\ninformation_schema\nmysql\nperformance_schema\nsys".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing docker_exec driver");
        // Extract required parameters
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let command = get_param_string(parameters, "command")?;
        let interactive = get_param_bool(parameters, "interactive", false);
        let tty = get_param_bool(parameters, "tty", false);
        let workdir = parameters.get("workdir").and_then(|v| v.as_str());
        // Build docker exec command arguments
        let mut args = vec!["exec"];
        if interactive {
            args.push("-i");
        }
        if tty {
            args.push("-t");
        }
        if let Some(wd) = workdir {
            args.push("-w");
            args.push(wd);
        }
        args.push(&container);
        args.push("sh");
        args.push("-c");
        args.push(&command);
        debug!("Executing command in container {}: {}", container, command);
        let opts = build_docker_opts(docker_host, 30);
        let result = exec_async("docker", &args, Some(opts)).await.map_err(|e| DriverError::execution(format!("Command failed: {}", e)))?;
        if !result.success {
            return Err(DriverError::execution(format!("Command failed: {}", result.stderr)));
        }
        let output = if result.stdout.is_empty() { "Command executed successfully (no output)".to_string() } else { result.stdout };
        info!("Command executed successfully in container: {}", container);
        return Ok(output);
    }
}
