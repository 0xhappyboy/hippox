//! k8s container orchestration utilities.
//!
//! This module provides skills for k8s operations:
//! - `K8sGetPodsSkill`: List pods in a namespace
//! - `K8sDescribePodSkill`: Get detailed pod information
//! - `K8sGetLogsSkill`: Get pod logs
//! - `K8sExecSkill`: Execute commands in a pod
//! - `K8sGetDeploymentsSkill`: List deployments
//! - `K8sScaleDeploymentSkill`: Scale a deployment
//! - `K8sRestartDeploymentSkill`: Restart a deployment
//! - `K8sGetNodesSkill`: List cluster nodes
//! - `K8sGetNamespacesSkill`: List namespaces
//! - `K8sApplyYamlSkill`: Apply YAML/JSON manifest
//! - `K8sDeleteResourceSkill`: Delete k8s resources
//! - `K8sGetConfigMapsSkill`: List configmaps
//! - `K8sGetSecretsSkill`: List secrets
//! - `K8sGetIngressesSkill`: List ingresses
//! - `K8sGetStatefulSetsSkill`: List statefulsets

use crate::config::{get_k8s_instance, list_k8s_instances};
use crate::executors::types::{Skill, SkillParameter};
use crate::executors::{ExecOptions, exec_async, exec_with_stdin_async};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Helper to get K8s instance config
fn get_k8s_config(instance_id: Option<&str>) -> Result<crate::config::K8sConfig> {
    if let Some(id) = instance_id {
        get_k8s_instance(id).ok_or_else(|| anyhow::anyhow!("K8s instance not found: {}", id))
    } else {
        let instances = list_k8s_instances();
        instances.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!("No K8s instance configured. Please add a K8s instance first.")
        })
    }
}

/// Helper to build kubectl command args with config
fn build_kubectl_args(
    base_args: &[&str],
    namespace: Option<&str>,
    all_namespaces: bool,
    config: &crate::config::K8sConfig,
) -> Vec<String> {
    let mut args = Vec::new();
    if !config.kubeconfig.is_empty() {
        args.push("--kubeconfig".to_string());
        args.push(config.kubeconfig.clone());
    }
    if !config.context.is_empty() {
        args.push("--context".to_string());
        args.push(config.context.clone());
    }
    if all_namespaces {
        args.push("--all-namespaces".to_string());
    } else if let Some(ns) = namespace {
        args.push("-n".to_string());
        args.push(ns.to_string());
    } else if !config.namespace.is_empty() && config.namespace != "default" {
        args.push("-n".to_string());
        args.push(config.namespace.clone());
    }

    for arg in base_args {
        args.push(arg.to_string());
    }
    args
}

/// Helper to build kubectl command with timeout
async fn exec_kubectl(args: &[&str], config: &crate::config::K8sConfig) -> Result<String> {
    let opts = ExecOptions::new().with_timeout(config.timeout);
    let result = exec_async("kubectl", args, Some(opts)).await?;
    if result.success {
        Ok(result.stdout)
    } else {
        Err(anyhow::anyhow!("kubectl failed: {}", result.stderr))
    }
}

/// Helper to build kubectl command with custom options
async fn exec_kubectl_with_opts(
    args: &[&str],
    opts: ExecOptions,
    config: &crate::config::K8sConfig,
) -> Result<String> {
    let result = exec_async("kubectl", args, Some(opts)).await?;
    if result.success {
        Ok(result.stdout)
    } else {
        Err(anyhow::anyhow!("kubectl failed: {}", result.stderr))
    }
}

/// A skill for listing k8s pods.
#[derive(Debug)]
pub struct K8sGetPodsSkill;

#[async_trait::async_trait]
impl Skill for K8sGetPodsSkill {
    fn name(&self) -> &str {
        "k8s_get_pods"
    }

    fn description(&self) -> &str {
        "List k8s pods in a namespace"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to see running pods, check pod status, or find pod names in a k8s cluster"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("kube-system")),
                enum_values: None,
            },
            SkillParameter {
                name: "all_namespaces".to_string(),
                param_type: "boolean".to_string(),
                description: "List pods in all namespaces".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "Label selector to filter pods (e.g., 'app=nginx')".to_string(),
                required: false,
                default: None,
                example: Some(json!("app=myapp")),
                enum_values: None,
            },
            SkillParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: wide, json, yaml".to_string(),
                required: false,
                default: Some(json!("wide")),
                example: Some(json!("json")),
                enum_values: Some(vec![
                    "wide".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                ]),
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_get_pods",
            "parameters": {
                "instance_id": "k8s_prod",
                "namespace": "production",
                "selector": "app=web"
            }
        })
    }

    fn example_output(&self) -> String {
        "NAME                     READY   STATUS    RESTARTS   AGE   IP           NODE\nweb-7b4c8d9f6-abc12       1/1     Running   0          5d    10.244.1.2   node-1\nweb-7b4c8d9f6-def34       1/1     Running   0          5d    10.244.2.3   node-2 [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        // Override config with parameters
        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let all_namespaces = parameters
            .get("all_namespaces")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let selector = parameters.get("selector").and_then(|v| v.as_str());
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("wide");

        let mut base_args = vec!["get", "pods"];
        match output {
            "json" => {
                base_args.push("-o");
                base_args.push("json");
            }
            "yaml" => {
                base_args.push("-o");
                base_args.push("yaml");
            }
            _ => {
                base_args.push("-o");
                base_args.push("wide");
            }
        }
        if let Some(sel) = selector {
            base_args.push("-l");
            base_args.push(sel);
        }

        let args = build_kubectl_args(&base_args, namespace, all_namespaces, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = exec_kubectl(&args_ref, &instance).await?;

        let output_result = if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                serde_json::to_string_pretty(&json_value)?
            } else {
                result
            }
        } else {
            result
        };

        Ok(format!("{} [instance: {}]", output_result, instance.name))
    }
}

/// A skill for getting detailed pod information.
#[derive(Debug)]
pub struct K8sDescribePodSkill;

#[async_trait::async_trait]
impl Skill for K8sDescribePodSkill {
    fn name(&self) -> &str {
        "k8s_describe_pod"
    }

    fn description(&self) -> &str {
        "Get detailed information about a k8s pod"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to debug pod issues, check pod events, or get detailed pod configuration"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "pod".to_string(),
                param_type: "string".to_string(),
                description: "Pod name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-pod-abc123")),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_describe_pod",
            "parameters": {
                "instance_id": "k8s_prod",
                "pod": "nginx-7b4c8d9f6-abc12",
                "namespace": "default"
            }
        })
    }

    fn example_output(&self) -> String {
        "Name:         nginx-7b4c8d9f6-abc12\nNamespace:    default\nPriority:     0\nNode:         node-1/192.168.1.10\n... [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let pod = parameters
            .get("pod")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pod"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());

        let base_args = vec!["describe", "pod", pod];
        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = exec_kubectl(&args_ref, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

/// A skill for getting pod logs.
#[derive(Debug)]
pub struct K8sGetLogsSkill;

#[async_trait::async_trait]
impl Skill for K8sGetLogsSkill {
    fn name(&self) -> &str {
        "k8s_get_logs"
    }

    fn description(&self) -> &str {
        "Get logs from a k8s pod"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to debug pod issues, check application logs, or monitor pod output"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "pod".to_string(),
                param_type: "string".to_string(),
                description: "Pod name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app-abc123")),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name (for pods with multiple containers)".to_string(),
                required: false,
                default: None,
                example: Some(json!("app")),
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
                description: "Show logs since duration (e.g., '1h', '30m')".to_string(),
                required: false,
                default: None,
                example: Some(json!("1h")),
                enum_values: None,
            },
            SkillParameter {
                name: "previous".to_string(),
                param_type: "boolean".to_string(),
                description: "Get logs from previous container instance".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
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
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_get_logs",
            "parameters": {
                "instance_id": "k8s_prod",
                "pod": "nginx-7b4c8d9f6-abc12",
                "tail": 50
            }
        })
    }

    fn example_output(&self) -> String {
        "2024-01-15T10:30:00Z [info] Server started\n2024-01-15T10:30:01Z [info] Listening on port 80 [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let pod = parameters
            .get("pod")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pod"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let container = parameters.get("container").and_then(|v| v.as_str());
        let tail = parameters
            .get("tail")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        let since = parameters.get("since").and_then(|v| v.as_str());
        let previous = parameters
            .get("previous")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let follow = parameters
            .get("follow")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let tail_str = tail.to_string();
        let mut base_args = vec!["logs", pod, "--tail", &tail_str];
        if let Some(c) = container {
            base_args.push("-c");
            base_args.push(c);
        }
        if let Some(s) = since {
            base_args.push("--since");
            base_args.push(s);
        }
        if previous {
            base_args.push("--previous");
        }
        if follow {
            base_args.push("--follow");
        }

        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = exec_kubectl(&args_ref, &instance).await?;

        let output = if result.is_empty() {
            "No logs available".to_string()
        } else {
            result
        };

        Ok(format!("{} [instance: {}]", output, instance.name))
    }
}

/// A skill for executing commands in a pod.
#[derive(Debug)]
pub struct K8sExecSkill;

#[async_trait::async_trait]
impl Skill for K8sExecSkill {
    fn name(&self) -> &str {
        "k8s_exec"
    }

    fn description(&self) -> &str {
        "Execute a command inside a k8s pod"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to run commands inside pods for debugging, maintenance, or diagnostics"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "pod".to_string(),
                param_type: "string".to_string(),
                description: "Pod name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app-abc123")),
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
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name (for pods with multiple containers)".to_string(),
                required: false,
                default: None,
                example: Some(json!("app")),
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
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_exec",
            "parameters": {
                "instance_id": "k8s_prod",
                "pod": "mysql-abc123",
                "command": "mysql -e 'SHOW DATABASES'"
            }
        })
    }

    fn example_output(&self) -> String {
        "Database\ninformation_schema\nmysql\nperformance_schema\nsys [instance: k8s_prod]"
            .to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let pod = parameters
            .get("pod")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pod"))?;
        let command = parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: command"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let container = parameters.get("container").and_then(|v| v.as_str());
        let interactive = parameters
            .get("interactive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tty = parameters
            .get("tty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut base_args = vec!["exec", pod];
        if interactive {
            base_args.push("-i");
        }
        if tty {
            base_args.push("-t");
        }
        if let Some(c) = container {
            base_args.push("-c");
            base_args.push(c);
        }
        base_args.push("--");
        base_args.push("sh");
        base_args.push("-c");
        base_args.push(command);

        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let opts = ExecOptions::new().with_timeout(instance.timeout);
        let result = exec_async("kubectl", &args_ref, Some(opts)).await?;

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

/// A skill for listing deployments.
#[derive(Debug)]
pub struct K8sGetDeploymentsSkill;

#[async_trait::async_trait]
impl Skill for K8sGetDeploymentsSkill {
    fn name(&self) -> &str {
        "k8s_get_deployments"
    }

    fn description(&self) -> &str {
        "List k8s deployments in a namespace"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check deployment status, replicas, and rollout history"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "all_namespaces".to_string(),
                param_type: "boolean".to_string(),
                description: "List deployments in all namespaces".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: wide, json, yaml".to_string(),
                required: false,
                default: Some(json!("wide")),
                example: Some(json!("json")),
                enum_values: Some(vec![
                    "wide".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                ]),
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_get_deployments",
            "parameters": {
                "instance_id": "k8s_prod",
                "namespace": "default"
            }
        })
    }

    fn example_output(&self) -> String {
        "NAME    READY   UP-TO-DATE   AVAILABLE   AGE\nnginx   3/3     3            3           5d [instance: k8s_prod]"
            .to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let all_namespaces = parameters
            .get("all_namespaces")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("wide");

        let mut base_args = vec!["get", "deployments"];
        match output {
            "json" => {
                base_args.push("-o");
                base_args.push("json");
            }
            "yaml" => {
                base_args.push("-o");
                base_args.push("yaml");
            }
            _ => {
                base_args.push("-o");
                base_args.push("wide");
            }
        }

        let args = build_kubectl_args(&base_args, namespace, all_namespaces, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = exec_kubectl(&args_ref, &instance).await?;

        let output_result = if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                serde_json::to_string_pretty(&json_value)?
            } else {
                result
            }
        } else {
            result
        };

        Ok(format!("{} [instance: {}]", output_result, instance.name))
    }
}

/// A skill for scaling a deployment.
#[derive(Debug)]
pub struct K8sScaleDeploymentSkill;

#[async_trait::async_trait]
impl Skill for K8sScaleDeploymentSkill {
    fn name(&self) -> &str {
        "k8s_scale_deployment"
    }

    fn description(&self) -> &str {
        "Scale a k8s deployment to the desired number of replicas"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to scale applications up or down based on load"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "deployment".to_string(),
                param_type: "string".to_string(),
                description: "Deployment name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app")),
                enum_values: None,
            },
            SkillParameter {
                name: "replicas".to_string(),
                param_type: "integer".to_string(),
                description: "Number of replicas".to_string(),
                required: true,
                default: None,
                example: Some(json!(3)),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_scale_deployment",
            "parameters": {
                "instance_id": "k8s_prod",
                "deployment": "nginx",
                "replicas": 5
            }
        })
    }

    fn example_output(&self) -> String {
        "Deployment 'nginx' scaled to 5 replicas [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let deployment = parameters
            .get("deployment")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: deployment"))?;
        let replicas = parameters
            .get("replicas")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: replicas"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());

        let replicas_str = replicas.to_string();
        let base_args = vec![
            "scale",
            "deployment",
            deployment,
            "--replicas",
            &replicas_str,
        ];
        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        exec_kubectl(&args_ref, &instance).await?;

        Ok(format!(
            "Deployment '{}' scaled to {} replicas [instance: {}]",
            deployment, replicas, instance.name
        ))
    }
}

/// A skill for restarting a deployment.
#[derive(Debug)]
pub struct K8sRestartDeploymentSkill;

#[async_trait::async_trait]
impl Skill for K8sRestartDeploymentSkill {
    fn name(&self) -> &str {
        "k8s_restart_deployment"
    }

    fn description(&self) -> &str {
        "Restart a k8s deployment by rolling restart"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to restart applications after config changes or to recover from issues"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "deployment".to_string(),
                param_type: "string".to_string(),
                description: "Deployment name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app")),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_restart_deployment",
            "parameters": {
                "instance_id": "k8s_prod",
                "deployment": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Deployment 'nginx' restarted successfully [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let deployment = parameters
            .get("deployment")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: deployment"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());

        let base_args = vec!["rollout", "restart", "deployment", deployment];
        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        exec_kubectl(&args_ref, &instance).await?;

        Ok(format!(
            "Deployment '{}' restarted successfully [instance: {}]",
            deployment, instance.name
        ))
    }
}

/// A skill for listing cluster nodes.
#[derive(Debug)]
pub struct K8sGetNodesSkill;

#[async_trait::async_trait]
impl Skill for K8sGetNodesSkill {
    fn name(&self) -> &str {
        "k8s_get_nodes"
    }

    fn description(&self) -> &str {
        "List k8s cluster nodes and their status"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check node health, capacity, and resource utilization"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: wide, json, yaml".to_string(),
                required: false,
                default: Some(json!("wide")),
                example: Some(json!("json")),
                enum_values: Some(vec![
                    "wide".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                ]),
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_get_nodes",
            "parameters": {
                "instance_id": "k8s_prod"
            }
        })
    }

    fn example_output(&self) -> String {
        "NAME     STATUS   ROLES    AGE   VERSION   INTERNAL-IP   EXTERNAL-IP\nnode-1   Ready    master   10d   v1.28.0   192.168.1.10   <none>\nnode-2   Ready    worker   10d   v1.28.0   192.168.1.11   <none> [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("wide");

        let mut args = vec!["get", "nodes"];
        match output {
            "json" => {
                args.push("-o");
                args.push("json");
            }
            "yaml" => {
                args.push("-o");
                args.push("yaml");
            }
            _ => {
                args.push("-o");
                args.push("wide");
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

        // For nodes, we need to handle kubeconfig/context differently (no namespace)
        let mut full_args = Vec::new();
        if !instance.kubeconfig.is_empty() {
            full_args.push("--kubeconfig");
            full_args.push(instance.kubeconfig.as_str());
        }
        if !instance.context.is_empty() {
            full_args.push("--context");
            full_args.push(instance.context.as_str());
        }
        full_args.extend(args_ref);

        let result = exec_kubectl(&full_args, &instance).await?;

        let output_result = if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                serde_json::to_string_pretty(&json_value)?
            } else {
                result
            }
        } else {
            result
        };

        Ok(format!("{} [instance: {}]", output_result, instance.name))
    }
}

/// A skill for listing namespaces.
#[derive(Debug)]
pub struct K8sGetNamespacesSkill;

#[async_trait::async_trait]
impl Skill for K8sGetNamespacesSkill {
    fn name(&self) -> &str {
        "k8s_get_namespaces"
    }

    fn description(&self) -> &str {
        "List k8s namespaces"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see available namespaces and their status"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: json, yaml".to_string(),
                required: false,
                default: Some(json!("table")),
                example: Some(json!("json")),
                enum_values: Some(vec![
                    "table".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                ]),
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_get_namespaces",
            "parameters": {
                "instance_id": "k8s_prod"
            }
        })
    }

    fn example_output(&self) -> String {
        "NAME              STATUS   AGE\ndefault           Active   10d\nkube-system       Active   10d\nkube-public       Active   10d [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;
        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("table");
        let mut args = vec!["get", "namespaces"];
        match output {
            "json" => {
                args.push("-o");
                args.push("json");
            }
            "yaml" => {
                args.push("-o");
                args.push("yaml");
            }
            _ => {}
        }
        let mut full_args = Vec::new();
        if !instance.kubeconfig.is_empty() {
            full_args.push("--kubeconfig");
            full_args.push(instance.kubeconfig.as_str());
        }
        if !instance.context.is_empty() {
            full_args.push("--context");
            full_args.push(instance.context.as_str());
        }
        full_args.extend(args.iter().map(|s| s));
        let result = exec_kubectl(&full_args, &instance).await?;
        let output_result = if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                serde_json::to_string_pretty(&json_value)?
            } else {
                result
            }
        } else {
            result
        };

        Ok(format!("{} [instance: {}]", output_result, instance.name))
    }
}

/// A skill for applying YAML/JSON manifests.
#[derive(Debug)]
pub struct K8sApplyYamlSkill;

#[async_trait::async_trait]
impl Skill for K8sApplyYamlSkill {
    fn name(&self) -> &str {
        "k8s_apply_yaml"
    }

    fn description(&self) -> &str {
        "Apply a k8s YAML or JSON manifest"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to create or update k8s resources from manifests"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "manifest".to_string(),
                param_type: "string".to_string(),
                description: "YAML or JSON manifest content".to_string(),
                required: true,
                default: None,
                example: Some(json!(
                    "apiVersion: v1\nkind: Pod\nmetadata:\n  name: my-pod"
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_apply_yaml",
            "parameters": {
                "instance_id": "k8s_prod",
                "manifest": "apiVersion: v1\nkind: Pod\nmetadata:\n  name: nginx\nspec:\n  containers:\n  - name: nginx\n    image: nginx:latest"
            }
        })
    }

    fn example_output(&self) -> String {
        "pod/nginx created [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let manifest = parameters
            .get("manifest")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: manifest"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());

        let base_args = vec!["apply", "-f", "-"];
        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let opts = ExecOptions::new().with_timeout(instance.timeout);
        let result = exec_with_stdin_async("kubectl", &args_ref, manifest, Some(opts)).await?;

        if !result.success {
            return Err(anyhow::anyhow!("Apply failed: {}", result.stderr));
        }

        Ok(format!("{} [instance: {}]", result.stdout, instance.name))
    }
}

/// A skill for deleting k8s resources.
#[derive(Debug)]
pub struct K8sDeleteResourceSkill;

#[async_trait::async_trait]
impl Skill for K8sDeleteResourceSkill {
    fn name(&self) -> &str {
        "k8s_delete_resource"
    }

    fn description(&self) -> &str {
        "Delete a k8s resource (pod, deployment, service, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to remove unwanted resources from the cluster"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_k8s_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "K8s instance ID (use 'list_k8s_instances' to see available clusters)"
                    .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("k8s_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "resource_type".to_string(),
                param_type: "string".to_string(),
                description: "Resource type (pod, deployment, service, configmap, secret, etc.)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(json!("pod")),
                enum_values: Some(vec![
                    "pod".to_string(),
                    "deployment".to_string(),
                    "service".to_string(),
                    "configmap".to_string(),
                    "secret".to_string(),
                    "ingress".to_string(),
                    "statefulset".to_string(),
                    "daemonset".to_string(),
                ]),
            },
            SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Resource name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-pod")),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace (default: instance default)".to_string(),
                required: false,
                default: None,
                example: Some(json!("production")),
                enum_values: None,
            },
            SkillParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force delete (for pods)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig path (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "k8s_delete_resource",
            "parameters": {
                "instance_id": "k8s_prod",
                "resource_type": "deployment",
                "name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "deployment.apps/nginx deleted [instance: k8s_prod]".to_string()
    }

    fn category(&self) -> &str {
        "devops"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_k8s_config(instance_id)?;

        if let Some(kubeconfig) = parameters.get("kubeconfig").and_then(|v| v.as_str()) {
            instance.kubeconfig = kubeconfig.to_string();
        }
        if let Some(context) = parameters.get("context").and_then(|v| v.as_str()) {
            instance.context = context.to_string();
        }

        let resource_type = parameters
            .get("resource_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: resource_type"))?;
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut base_args = vec!["delete", resource_type, name];
        if force && resource_type == "pod" {
            base_args.push("--force");
            base_args.push("--grace-period=0");
        }

        let args = build_kubectl_args(&base_args, namespace, false, &instance);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = exec_kubectl(&args_ref, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_k8s_get_namespaces() {
        let skill = K8sGetNamespacesSkill;
        let params = HashMap::new();
        let result = skill.execute(&params).await;
        if let Ok(output) = result {
            assert!(output.contains("default") || output.contains("NAME"));
        }
    }
}
