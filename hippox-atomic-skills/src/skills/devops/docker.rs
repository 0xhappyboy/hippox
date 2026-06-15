//! Docker container orchestration utilities.

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use crate::{ExecOptions, exec_async, exec_with_stdin_async};

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    params
        .get(name)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}

fn build_docker_opts(docker_host: Option<&str>, timeout: u64) -> ExecOptions {
    let mut opts = ExecOptions::new().with_timeout(timeout);
    if let Some(host) = docker_host {
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }
    }
    opts
}

/// A skill for listing Docker containers.
#[derive(Debug)]
pub struct DockerPsSkill;

#[async_trait::async_trait]
impl Skill for DockerPsSkill {
    fn name(&self) -> &str {
        "docker_ps"
    }
    fn description(&self) -> &str {
        "List Docker containers"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to see running containers, check container status, or find container IDs"
    }
    fn category(&self) -> &str {
        "devops"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (e.g., unix:///var/run/docker.sock)".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "all".to_string(),
                param_type: "boolean".to_string(),
                description: "Show all containers (including stopped)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Filter output (e.g., 'status=exited', 'name=myapp')".to_string(),
                required: false,
                default: None,
                example: Some(json!("status=running")),
                enum_values: None,
            },
            SkillParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format: table, json, or quiet".to_string(),
                required: false,
                default: Some(json!("table")),
                example: Some(json!("json")),
                enum_values: Some(vec![
                    "table".to_string(),
                    "json".to_string(),
                    "quiet".to_string(),
                ]),
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Command timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "docker_ps", "parameters": { "all": true, "format": "json" } })
    }

    fn example_output(&self) -> String {
        "CONTAINER ID   IMAGE     COMMAND   STATUS          PORTS     NAMES\nabc123def456   nginx     \"nginx\"   Up 2 hours      80/tcp    web_nginx".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let all = get_param_bool(parameters, "all", false);
        let filter = parameters.get("filter").and_then(|v| v.as_str());
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("table");
        let timeout = get_param_u64(parameters, "timeout", 30);

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

        let opts = build_docker_opts(docker_host, timeout);
        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!("Docker ps failed: {}", result.stderr));
        }

        let output = if format == "json" {
            let containers: Vec<serde_json::Value> = result
                .stdout
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            serde_json::to_string_pretty(&containers)?
        } else {
            result.stdout
        };
        Ok(output)
    }
}

/// A skill for starting or stopping Docker containers.
#[derive(Debug)]
pub struct DockerStartStopSkill;

#[async_trait::async_trait]
impl Skill for DockerStartStopSkill {
    fn name(&self) -> &str {
        "docker_start_stop"
    }
    fn description(&self) -> &str {
        "Start, stop, restart, or pause Docker containers"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to control container lifecycle: start, stop, restart, pause, or unpause"
    }
    fn category(&self) -> &str {
        "devops"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_container")),
                enum_values: None,
            },
            SkillParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action to perform: start, stop, restart, pause, unpause".to_string(),
                required: true,
                default: None,
                example: Some(json!("restart")),
                enum_values: Some(vec![
                    "start".to_string(),
                    "stop".to_string(),
                    "restart".to_string(),
                    "pause".to_string(),
                    "unpause".to_string(),
                ]),
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds for stop (default: 10)".to_string(),
                required: false,
                default: Some(json!(10)),
                example: Some(json!(30)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "docker_start_stop", "parameters": { "container": "redis", "action": "restart" } })
    }

    fn example_output(&self) -> String {
        "Container 'redis' restarted successfully".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let action = get_param_string(parameters, "action")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 10);

        let docker_cmd = match action.as_str() {
            "start" => "start",
            "stop" => "stop",
            "restart" => "restart",
            "pause" => "pause",
            "unpause" => "unpause",
            _ => return Err(anyhow::anyhow!("Unknown action: {}", action)),
        };

        let mut args: Vec<String> = vec![docker_cmd.to_string()];
        if action == "stop" {
            args.push("-t".to_string());
            args.push(timeout_secs.to_string());
        }
        args.push(container.clone());
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let opts = build_docker_opts(docker_host, 60);
        let result = exec_async("docker", &args_ref, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!(
                "Failed to {} container: {}",
                action,
                result.stderr
            ));
        }
        Ok(format!(
            "Container '{}' {}ed successfully",
            container, action
        ))
    }
}

/// A skill for viewing Docker container logs.
#[derive(Debug)]
pub struct DockerLogsSkill;

#[async_trait::async_trait]
impl Skill for DockerLogsSkill {
    fn name(&self) -> &str {
        "docker_logs"
    }
    fn description(&self) -> &str {
        "View logs from a Docker container"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to debug container issues, monitor output, or check error logs"
    }
    fn category(&self) -> &str {
        "devops"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_app")),
                enum_values: None,
            },
            SkillParameter {
                name: "tail".to_string(),
                param_type: "integer".to_string(),
                description: "Number of lines to show from the end".to_string(),
                required: false,
                default: Some(json!(100)),
                example: Some(json!(50)),
                enum_values: None,
            },
            SkillParameter {
                name: "since".to_string(),
                param_type: "string".to_string(),
                description: "Show logs since timestamp (e.g., '2024-01-01T00:00:00Z' or '1h')"
                    .to_string(),
                required: false,
                default: None,
                example: Some(json!("1h")),
                enum_values: None,
            },
            SkillParameter {
                name: "follow".to_string(),
                param_type: "boolean".to_string(),
                description: "Follow log output (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "timestamps".to_string(),
                param_type: "boolean".to_string(),
                description: "Show timestamps (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "docker_logs", "parameters": { "container": "mysql", "tail": 20 } })
    }

    fn example_output(&self) -> String {
        "2024-01-15T10:30:00Z [Note] [MY-010914] [Server] Shutdown complete\n2024-01-15T10:30:01Z [System] [MY-010116] [Server] /usr/sbin/mysqld: ready for connections".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let tail = get_param_u64(parameters, "tail", 100);
        let since = parameters.get("since").and_then(|v| v.as_str());
        let follow = get_param_bool(parameters, "follow", false);
        let timestamps = get_param_bool(parameters, "timestamps", false);
        let tail_str = tail.to_string();
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

        let opts = build_docker_opts(docker_host, 60);
        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!("Failed to get logs: {}", result.stderr));
        }
        Ok(if result.stdout.is_empty() {
            "No logs available".to_string()
        } else {
            result.stdout
        })
    }
}

/// A skill for getting detailed information about a Docker container.
#[derive(Debug)]
pub struct DockerInspectSkill;

#[async_trait::async_trait]
impl Skill for DockerInspectSkill {
    fn name(&self) -> &str {
        "docker_inspect"
    }
    fn description(&self) -> &str {
        "Get detailed JSON information about a Docker container"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when you need detailed container configuration, network settings, or mount information"
    }
    fn category(&self) -> &str {
        "devops"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_container")),
                enum_values: None,
            },
            SkillParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Go template format for output".to_string(),
                required: false,
                default: None,
                example: Some(json!("{{.Name}} {{.State.Status}}")),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "docker_inspect", "parameters": { "container": "nginx" } })
    }

    fn example_output(&self) -> String {
        "Detailed JSON container configuration".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let format = parameters.get("format").and_then(|v| v.as_str());

        let mut args = vec!["inspect"];
        if let Some(f) = format {
            args.push("--format");
            args.push(f);
        }
        args.push(&container);

        let opts = build_docker_opts(docker_host, 30);
        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!(
                "Failed to inspect container: {}",
                result.stderr
            ));
        }
        Ok(result.stdout)
    }
}

/// A skill for executing commands in a running Docker container.
#[derive(Debug)]
pub struct DockerExecSkill;

#[async_trait::async_trait]
impl Skill for DockerExecSkill {
    fn name(&self) -> &str {
        "docker_exec"
    }
    fn description(&self) -> &str {
        "Execute a command inside a running Docker container"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to run commands inside containers for debugging, maintenance, or automation"
    }
    fn category(&self) -> &str {
        "devops"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "docker_host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host".to_string(),
                required: false,
                default: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                example: Some(Value::String("tcp://localhost:2375".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name or ID".to_string(),
                required: true,
                default: None,
                example: Some(json!("my_app")),
                enum_values: None,
            },
            SkillParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to execute".to_string(),
                required: true,
                default: None,
                example: Some(json!("ls -la")),
                enum_values: None,
            },
            SkillParameter {
                name: "interactive".to_string(),
                param_type: "boolean".to_string(),
                description: "Keep STDIN open (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "tty".to_string(),
                param_type: "boolean".to_string(),
                description: "Allocate a pseudo-TTY (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "workdir".to_string(),
                param_type: "string".to_string(),
                description: "Working directory inside the container".to_string(),
                required: false,
                default: None,
                example: Some(json!("/app")),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "docker_exec", "parameters": { "container": "mysql", "command": "mysql -e 'SHOW DATABASES'" } })
    }

    fn example_output(&self) -> String {
        "Database\ninformation_schema\nmysql\nperformance_schema\nsys".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let docker_host = parameters.get("docker_host").and_then(|v| v.as_str());
        let container = get_param_string(parameters, "container")?;
        let command = get_param_string(parameters, "command")?;
        let interactive = get_param_bool(parameters, "interactive", false);
        let tty = get_param_bool(parameters, "tty", false);
        let workdir = parameters.get("workdir").and_then(|v| v.as_str());

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

        let opts = build_docker_opts(docker_host, 30);
        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!("Command failed: {}", result.stderr));
        }
        Ok(if result.stdout.is_empty() {
            "Command executed successfully (no output)".to_string()
        } else {
            result.stdout
        })
    }
}
