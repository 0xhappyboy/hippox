// k8s.rs
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

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use crate::{ExecOptions, SkillCategory, exec_async, exec_with_stdin_async};

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

fn build_kubectl_env(kubeconfig: Option<&str>) -> ExecOptions {
    let mut opts = ExecOptions::new();
    if let Some(kc) = kubeconfig {
        if !kc.is_empty() {
            opts = opts.with_env("KUBECONFIG", kc);
        }
    }
    opts
}

async fn exec_kubectl(args: &[&str], kubeconfig: Option<&str>, timeout: u64) -> Result<String> {
    let opts = build_kubectl_env(kubeconfig).with_timeout(timeout);
    let result = exec_async("kubectl", args, Some(opts)).await?;
    if result.success {
        Ok(result.stdout)
    } else {
        Err(anyhow::anyhow!("kubectl failed: {}", result.stderr))
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
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
        json!({ "action": "k8s_get_pods", "parameters": { "namespace": "production", "selector": "app=web" } })
    }

    fn example_output(&self) -> String {
        "NAME                     READY   STATUS    RESTARTS   AGE   IP           NODE\nweb-7b4c8d9f6-abc12       1/1     Running   0          5d    10.244.1.2   node-1\nweb-7b4c8d9f6-def34       1/1     Running   0          5d    10.244.2.3   node-2".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let all_namespaces = get_param_bool(parameters, "all_namespaces", false);
        let selector = parameters.get("selector").and_then(|v| v.as_str());
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("wide");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["get", "pods"];
        if all_namespaces {
            args.push("--all-namespaces");
        } else if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        if let Some(sel) = selector {
            args.push("-l");
            args.push(sel);
        }
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
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                return Ok(serde_json::to_string_pretty(&json_value)?);
            }
        }
        Ok(result)
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
        json!({ "action": "k8s_describe_pod", "parameters": { "pod": "nginx-7b4c8d9f6-abc12", "namespace": "default" } })
    }

    fn example_output(&self) -> String {
        "Name:         nginx-7b4c8d9f6-abc12\nNamespace:    default\nPriority:     0\nNode:         node-1/192.168.1.10\n...".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let pod = get_param_string(parameters, "pod")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["describe", "pod", &pod];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        } else {
            args.push("-n");
            args.push("default");
        }
        exec_kubectl(&args, kubeconfig, timeout).await
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
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
        json!({ "action": "k8s_get_logs", "parameters": { "pod": "nginx-7b4c8d9f6-abc12", "tail": 50 } })
    }

    fn example_output(&self) -> String {
        "2024-01-15T10:30:00Z [info] Server started\n2024-01-15T10:30:01Z [info] Listening on port 80".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let pod = get_param_string(parameters, "pod")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let container = parameters.get("container").and_then(|v| v.as_str());
        let tail = get_param_u64(parameters, "tail", 100);
        let since = parameters.get("since").and_then(|v| v.as_str());
        let previous = get_param_bool(parameters, "previous", false);
        let follow = get_param_bool(parameters, "follow", false);
        let timeout = get_param_u64(parameters, "timeout", 30);
        let tail_str = tail.to_string();
        let mut args = vec!["logs", &pod, "--tail", &tail_str];
        if let Some(c) = container {
            args.push("-c");
            args.push(c);
        }
        if let Some(s) = since {
            args.push("--since");
            args.push(s);
        }
        if previous {
            args.push("--previous");
        }
        if follow {
            args.push("--follow");
        }
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }

        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        Ok(if result.is_empty() {
            "No logs available".to_string()
        } else {
            result
        })
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
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
        json!({ "action": "k8s_exec", "parameters": { "pod": "mysql-abc123", "command": "mysql -e 'SHOW DATABASES'" } })
    }

    fn example_output(&self) -> String {
        "Database\ninformation_schema\nmysql\nperformance_schema\nsys".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let pod = get_param_string(parameters, "pod")?;
        let command = get_param_string(parameters, "command")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let container = parameters.get("container").and_then(|v| v.as_str());
        let interactive = get_param_bool(parameters, "interactive", false);
        let tty = get_param_bool(parameters, "tty", false);
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["exec", &pod];
        if interactive {
            args.push("-i");
        }
        if tty {
            args.push("-t");
        }
        if let Some(c) = container {
            args.push("-c");
            args.push(c);
        }
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        args.push("--");
        args.push("sh");
        args.push("-c");
        args.push(&command);

        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        Ok(if result.is_empty() {
            "Command executed successfully (no output)".to_string()
        } else {
            result
        })
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
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
        json!({ "action": "k8s_get_deployments", "parameters": { "namespace": "default" } })
    }

    fn example_output(&self) -> String {
        "NAME    READY   UP-TO-DATE   AVAILABLE   AGE\nnginx   3/3     3            3           5d"
            .to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let all_namespaces = get_param_bool(parameters, "all_namespaces", false);
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("wide");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["get", "deployments"];
        if all_namespaces {
            args.push("--all-namespaces");
        } else if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
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

        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                return Ok(serde_json::to_string_pretty(&json_value)?);
            }
        }
        Ok(result)
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
        json!({ "action": "k8s_scale_deployment", "parameters": { "deployment": "nginx", "replicas": 5, "namespace": "default" } })
    }

    fn example_output(&self) -> String {
        "Deployment 'nginx' scaled to 5 replicas".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let deployment = get_param_string(parameters, "deployment")?;
        let replicas = get_param_u64(parameters, "replicas", 1);
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);
        let replicas_str = replicas.to_string();
        let mut args = vec![
            "scale",
            "deployment",
            &deployment,
            "--replicas",
            &replicas_str,
        ];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        exec_kubectl(&args, kubeconfig, timeout).await?;
        Ok(format!(
            "Deployment '{}' scaled to {} replicas",
            deployment, replicas
        ))
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
        json!({ "action": "k8s_restart_deployment", "parameters": { "deployment": "nginx" } })
    }

    fn example_output(&self) -> String {
        "Deployment 'nginx' restarted successfully".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let deployment = get_param_string(parameters, "deployment")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["rollout", "restart", "deployment", &deployment];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }

        exec_kubectl(&args, kubeconfig, timeout).await?;
        Ok(format!(
            "Deployment '{}' restarted successfully",
            deployment
        ))
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
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
        json!({ "action": "k8s_get_nodes", "parameters": {} })
    }

    fn example_output(&self) -> String {
        "NAME     STATUS   ROLES    AGE   VERSION   INTERNAL-IP   EXTERNAL-IP\nnode-1   Ready    master   10d   v1.28.0   192.168.1.10   <none>\nnode-2   Ready    worker   10d   v1.28.0   192.168.1.11   <none>".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("wide");
        let timeout = get_param_u64(parameters, "timeout", 30);

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

        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                return Ok(serde_json::to_string_pretty(&json_value)?);
            }
        }
        Ok(result)
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
        json!({ "action": "k8s_get_namespaces", "parameters": {} })
    }

    fn example_output(&self) -> String {
        "NAME              STATUS   AGE\ndefault           Active   10d\nkube-system       Active   10d\nkube-public       Active   10d".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("table");
        let timeout = get_param_u64(parameters, "timeout", 30);

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

        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                return Ok(serde_json::to_string_pretty(&json_value)?);
            }
        }
        Ok(result)
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
        json!({ "action": "k8s_apply_yaml", "parameters": { "manifest": "apiVersion: v1\nkind: Pod\nmetadata:\n  name: nginx\nspec:\n  containers:\n  - name: nginx\n    image: nginx:latest" } })
    }

    fn example_output(&self) -> String {
        "pod/nginx created".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let manifest = get_param_string(parameters, "manifest")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["apply", "-f", "-"];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        let opts = build_kubectl_env(kubeconfig).with_timeout(timeout);
        let result = exec_with_stdin_async("kubectl", &args, &manifest, Some(opts)).await?;
        if !result.success {
            return Err(anyhow::anyhow!("Apply failed: {}", result.stderr));
        }
        Ok(result.stdout)
    }
}

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
    fn category(&self) -> SkillCategory {
        SkillCategory::Devops
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
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
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
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
        json!({ "action": "k8s_delete_resource", "parameters": { "resource_type": "deployment", "name": "nginx" } })
    }

    fn example_output(&self) -> String {
        "deployment.apps/nginx deleted".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let resource_type = get_param_string(parameters, "resource_type")?;
        let name = get_param_string(parameters, "name")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let force = get_param_bool(parameters, "force", false);
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut args = vec!["delete", &resource_type, &name];
        if force && resource_type == "pod" {
            args.push("--force");
            args.push("--grace-period=0");
        }
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }

        exec_kubectl(&args, kubeconfig, timeout).await
    }
}
