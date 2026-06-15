use crate::config::{get_docker_instance, list_docker_instances};
use crate::executors::types::{Skill, SkillParameter};
use crate::executors::{ExecOptions, exec_async};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Helper to get Docker instance config or fallback to default
fn get_docker_config(instance_id: Option<&str>) -> Result<crate::config::DockerConfig> {
    if let Some(id) = instance_id {
        get_docker_instance(id).ok_or_else(|| anyhow::anyhow!("Docker instance not found: {}", id))
    } else {
        let instances = list_docker_instances();
        instances.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!("No Docker instance configured. Please add a Docker instance first.")
        })
    }
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

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_docker_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "Docker instance ID (use 'list_docker_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("docker_local".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
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
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docker_ps",
            "parameters": {
                "instance_id": "docker_local",
                "all": true
            }
        })
    }

    fn example_output(&self) -> String {
        "CONTAINER ID   IMAGE     COMMAND   STATUS          PORTS     NAMES\nabc123def456   nginx     \"nginx\"   Up 2 hours      80/tcp    web_nginx [instance: docker_local]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let instance = get_docker_config(instance_id)?;

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());

        let all = parameters
            .get("all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let filter = parameters.get("filter").and_then(|v| v.as_str());
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("table");

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

        let mut opts = ExecOptions::new().with_timeout(instance.timeout);
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }

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

        Ok(format!("{} [instance: {}]", output, instance.name))
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

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_docker_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "Docker instance ID (use 'list_docker_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("docker_local".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
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
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docker_start_stop",
            "parameters": {
                "instance_id": "docker_local",
                "container": "redis",
                "action": "restart"
            }
        })
    }

    fn example_output(&self) -> String {
        "Container 'redis' restarted successfully [instance: docker_local]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let instance = get_docker_config(instance_id)?;

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());

        let container = parameters
            .get("container")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: container"))?;
        let action = parameters
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let docker_cmd = match action {
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
        args.push(container.to_string());
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let mut opts = ExecOptions::new();
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }

        let result = exec_async("docker", &args_ref, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!(
                "Failed to {} container: {}",
                action,
                result.stderr
            ));
        }

        Ok(format!(
            "Container '{}' {}ed successfully [instance: {}]",
            container, action, instance.name
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

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_docker_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "Docker instance ID (use 'list_docker_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("docker_local".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
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
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docker_logs",
            "parameters": {
                "instance_id": "docker_local",
                "container": "mysql",
                "tail": 20
            }
        })
    }

    fn example_output(&self) -> String {
        "2024-01-15T10:30:00Z [Note] [MY-010914] [Server] Shutdown complete\n2024-01-15T10:30:01Z [System] [MY-010116] [Server] /usr/sbin/mysqld: ready for connections [instance: docker_local]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let instance = get_docker_config(instance_id)?;

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());

        let container = parameters
            .get("container")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: container"))?;
        let tail = parameters
            .get("tail")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        let since = parameters.get("since").and_then(|v| v.as_str());
        let follow = parameters
            .get("follow")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let timestamps = parameters
            .get("timestamps")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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
        args.push(container);

        let mut opts = ExecOptions::new();
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }

        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!("Failed to get logs: {}", result.stderr));
        }

        let output = if result.stdout.is_empty() {
            "No logs available".to_string()
        } else {
            result.stdout
        };

        Ok(format!("{} [instance: {}]", output, instance.name))
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

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_docker_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "Docker instance ID (use 'list_docker_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("docker_local".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
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
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docker_inspect",
            "parameters": {
                "instance_id": "docker_local",
                "container": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Detailed JSON container configuration [instance: docker_local]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let instance = get_docker_config(instance_id)?;

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());

        let container = parameters
            .get("container")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: container"))?;
        let format = parameters.get("format").and_then(|v| v.as_str());

        let mut args = vec!["inspect"];
        if let Some(f) = format {
            args.push("--format");
            args.push(f);
        }
        args.push(container);

        let mut opts = ExecOptions::new();
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }

        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!(
                "Failed to inspect container: {}",
                result.stderr
            ));
        }

        Ok(format!("{} [instance: {}]", result.stdout, instance.name))
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

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_docker_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "Docker instance ID (use 'list_docker_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("docker_local".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
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
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Docker host (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("unix:///var/run/docker.sock".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docker_exec",
            "parameters": {
                "instance_id": "docker_local",
                "container": "mysql",
                "command": "mysql -e 'SHOW DATABASES'"
            }
        })
    }

    fn example_output(&self) -> String {
        "Database\ninformation_schema\nmysql\nperformance_schema\nsys [instance: docker_local]"
            .to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let instance = get_docker_config(instance_id)?;

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());

        let container = parameters
            .get("container")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: container"))?;
        let command = parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: command"))?;
        let interactive = parameters
            .get("interactive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tty = parameters
            .get("tty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
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
        args.push(container);
        args.push("sh");
        args.push("-c");
        args.push(command);

        let mut opts = ExecOptions::new();
        if !host.is_empty() {
            opts = opts.with_env("DOCKER_HOST", host);
        }

        let result = exec_async("docker", &args, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!("Command failed: {}", result.stderr));
        }

        let output = if result.stdout.is_empty() {
            "Command executed successfully (no output)".to_string()
        } else {
            result.stdout
        };

        Ok(format!("{} [instance: {}]", output, instance.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_ps() {
        let skill = DockerPsSkill;
        let params = HashMap::new();
        let result = skill.execute(&params).await;
        if let Ok(output) = result {
            assert!(output.contains("CONTAINER") || output.is_empty());
        }
    }
}
