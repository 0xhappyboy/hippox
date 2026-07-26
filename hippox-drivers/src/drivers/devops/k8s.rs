//! Kubernetes container orchestration utilities.
//!
//! This module provides drivers for Kubernetes operations including:
//! - Pod management (list, describe, logs, exec)
//! - Deployment management (list, scale, restart)
//! - Cluster management (nodes, namespaces)
//! - Resource management (apply, delete)
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
/// Builds kubectl execution environment options
///
/// # Arguments
/// * `kubeconfig` - Kubeconfig file path
///
/// # Returns
/// * `ExecOptions` - Configured execution options
fn build_kubectl_env(kubeconfig: Option<&str>) -> ExecOptions {
    let mut opts = ExecOptions::new();
    if let Some(kc) = kubeconfig {
        if !kc.is_empty() {
            opts = opts.with_env("KUBECONFIG", kc);
        }
    }
    return opts;
}
/// Executes a kubectl command
///
/// # Arguments
/// * `args` - Command arguments
/// * `kubeconfig` - Kubeconfig file path
/// * `timeout` - Command timeout in seconds
///
/// # Returns
/// * `DriverResult<String>` - Command output on success
async fn exec_kubectl(args: &[&str], kubeconfig: Option<&str>, timeout: u64) -> DriverResult<String> {
    debug!("Executing kubectl with args: {:?}", args);
    let opts = build_kubectl_env(kubeconfig).with_timeout(timeout);
    let result = exec_async("kubectl", args, Some(opts)).await.map_err(|e| DriverError::execution(format!("kubectl execution failed: {}", e)))?;
    if result.success {
        info!("kubectl command executed successfully");
        return Ok(result.stdout);
    } else {
        return Err(DriverError::execution(format!("kubectl failed: {}", result.stderr)));
    }
}
// ========== Pod Management Drivers ==========
/// Driver for listing Kubernetes pods
#[derive(Debug)]
pub struct K8sGetPodsDriver;
#[async_trait::async_trait]
impl Driver for K8sGetPodsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_get_pods";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List k8s pods in a namespace";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to see running pods, check pod status, or find pod names in a k8s cluster";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("kube-system")),
                enum_values: None,
            },
            DriverParameter {
                name: "all_namespaces".to_string(),
                param_type: "boolean".to_string(),
                description: "List pods in all namespaces".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "Label selector to filter pods (e.g., 'app=nginx')".to_string(),
                required: false,
                default: None,
                example: Some(json!("app=myapp")),
                enum_values: None,
            },
            DriverParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: wide, json, yaml".to_string(),
                required: false,
                default: Some(json!("wide")),
                example: Some(json!("json")),
                enum_values: Some(vec!["wide".to_string(), "json".to_string(), "yaml".to_string()]),
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
            "action": "k8s_get_pods",
            "parameters": {
                "namespace": "production",
                "selector": "app=web"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "NAME                     READY   STATUS    RESTARTS   AGE   IP           NODE\nweb-7b4c8d9f6-abc12       1/1     Running   0          5d    10.244.1.2   node-1\nweb-7b4c8d9f6-def34       1/1     Running   0          5d    10.244.2.3   node-2".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_get_pods driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let all_namespaces = get_param_bool(parameters, "all_namespaces", false);
        let selector = parameters.get("selector").and_then(|v| v.as_str());
        let output = parameters.get("output").and_then(|v| v.as_str()).unwrap_or("wide");
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
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
        debug!("Listing pods in namespace: {:?}", namespace);
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                let pretty =
                    serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
                info!("Successfully listed pods in JSON format");
                return Ok(pretty);
            }
        }
        info!("Successfully listed pods");
        return Ok(result);
    }
}
/// Driver for describing a Kubernetes pod
#[derive(Debug)]
pub struct K8sDescribePodDriver;
#[async_trait::async_trait]
impl Driver for K8sDescribePodDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_describe_pod";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get detailed information about a k8s pod";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to debug pod issues, check pod events, or get detailed pod configuration";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "pod".to_string(),
                param_type: "string".to_string(),
                description: "Pod name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-pod-abc123")),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
            "action": "k8s_describe_pod",
            "parameters": {
                "pod": "nginx-7b4c8d9f6-abc12",
                "namespace": "default"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Name:         nginx-7b4c8d9f6-abc12\nNamespace:    default\nPriority:     0\nNode:         node-1/192.168.1.10\n...".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_describe_pod driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let pod = get_param_string(parameters, "pod")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
        let mut args = vec!["describe", "pod", &pod];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        } else {
            args.push("-n");
            args.push("default");
        }
        debug!("Describing pod: {} in namespace: {:?}", pod, namespace);
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        info!("Successfully described pod: {}", pod);
        return Ok(result);
    }
}
/// Driver for getting Kubernetes pod logs
#[derive(Debug)]
pub struct K8sGetLogsDriver;
#[async_trait::async_trait]
impl Driver for K8sGetLogsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_get_logs";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get logs from a k8s pod";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to debug pod issues, check application logs, or monitor pod output";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "pod".to_string(),
                param_type: "string".to_string(),
                description: "Pod name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app-abc123")),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
            },
            DriverParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name (for pods with multiple containers)".to_string(),
                required: false,
                default: None,
                example: Some(json!("app")),
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
                description: "Show logs since duration (e.g., '1h', '30m')".to_string(),
                required: false,
                default: None,
                example: Some(json!("1h")),
                enum_values: None,
            },
            DriverParameter {
                name: "previous".to_string(),
                param_type: "boolean".to_string(),
                description: "Get logs from previous container instance".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
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
            "action": "k8s_get_logs",
            "parameters": {
                "pod": "nginx-7b4c8d9f6-abc12",
                "tail": 50
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "2024-01-15T10:30:00Z [info] Server started\n2024-01-15T10:30:01Z [info] Listening on port 80".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_get_logs driver");
        // Extract required parameters
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
        // Build kubectl command arguments
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
        debug!("Getting logs for pod: {}", pod);
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        let output = if result.is_empty() { "No logs available".to_string() } else { result };
        info!("Successfully retrieved logs for pod: {}", pod);
        return Ok(output);
    }
}
/// Driver for executing commands in a Kubernetes pod
#[derive(Debug)]
pub struct K8sExecDriver;
#[async_trait::async_trait]
impl Driver for K8sExecDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_exec";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute a command inside a k8s pod";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to run commands inside pods for debugging, maintenance, or diagnostics";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "pod".to_string(),
                param_type: "string".to_string(),
                description: "Pod name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app-abc123")),
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
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
            },
            DriverParameter {
                name: "container".to_string(),
                param_type: "string".to_string(),
                description: "Container name (for pods with multiple containers)".to_string(),
                required: false,
                default: None,
                example: Some(json!("app")),
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
            "action": "k8s_exec",
            "parameters": {
                "pod": "mysql-abc123",
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
        debug!("Executing k8s_exec driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let pod = get_param_string(parameters, "pod")?;
        let command = get_param_string(parameters, "command")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let container = parameters.get("container").and_then(|v| v.as_str());
        let interactive = get_param_bool(parameters, "interactive", false);
        let tty = get_param_bool(parameters, "tty", false);
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
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
        debug!("Executing command in pod {}: {}", pod, command);
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        let output = if result.is_empty() { "Command executed successfully (no output)".to_string() } else { result };
        info!("Successfully executed command in pod: {}", pod);
        return Ok(output);
    }
}
// ========== Deployment Management Drivers ==========
/// Driver for listing Kubernetes deployments
#[derive(Debug)]
pub struct K8sGetDeploymentsDriver;
#[async_trait::async_trait]
impl Driver for K8sGetDeploymentsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_get_deployments";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List k8s deployments in a namespace";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to check deployment status, replicas, and rollout history";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
            },
            DriverParameter {
                name: "all_namespaces".to_string(),
                param_type: "boolean".to_string(),
                description: "List deployments in all namespaces".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: wide, json, yaml".to_string(),
                required: false,
                default: Some(json!("wide")),
                example: Some(json!("json")),
                enum_values: Some(vec!["wide".to_string(), "json".to_string(), "yaml".to_string()]),
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
            "action": "k8s_get_deployments",
            "parameters": {
                "namespace": "default"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "NAME    READY   UP-TO-DATE   AVAILABLE   AGE\nnginx   3/3     3            3           5d".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_get_deployments driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let all_namespaces = get_param_bool(parameters, "all_namespaces", false);
        let output = parameters.get("output").and_then(|v| v.as_str()).unwrap_or("wide");
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
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
        debug!("Listing deployments in namespace: {:?}", namespace);
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                let pretty =
                    serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
                info!("Successfully listed deployments in JSON format");
                return Ok(pretty);
            }
        }
        info!("Successfully listed deployments");
        return Ok(result);
    }
}
/// Driver for scaling a Kubernetes deployment
#[derive(Debug)]
pub struct K8sScaleDeploymentDriver;
#[async_trait::async_trait]
impl Driver for K8sScaleDeploymentDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_scale_deployment";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Scale a k8s deployment to the desired number of replicas";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to scale applications up or down based on load";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "deployment".to_string(),
                param_type: "string".to_string(),
                description: "Deployment name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app")),
                enum_values: None,
            },
            DriverParameter {
                name: "replicas".to_string(),
                param_type: "integer".to_string(),
                description: "Number of replicas".to_string(),
                required: true,
                default: None,
                example: Some(json!(3)),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
            "action": "k8s_scale_deployment",
            "parameters": {
                "deployment": "nginx",
                "replicas": 5,
                "namespace": "default"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Deployment 'nginx' scaled to 5 replicas".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_scale_deployment driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let deployment = get_param_string(parameters, "deployment")?;
        let replicas = get_param_u64(parameters, "replicas", 1);
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);
        let replicas_str = replicas.to_string();
        // Build kubectl command arguments
        let mut args = vec!["scale", "deployment", &deployment, "--replicas", &replicas_str];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        debug!("Scaling deployment {} to {} replicas", deployment, replicas);
        exec_kubectl(&args, kubeconfig, timeout).await?;
        info!("Deployment '{}' scaled to {} replicas", deployment, replicas);
        return Ok(format!("Deployment '{}' scaled to {} replicas", deployment, replicas));
    }
}
/// Driver for restarting a Kubernetes deployment
#[derive(Debug)]
pub struct K8sRestartDeploymentDriver;
#[async_trait::async_trait]
impl Driver for K8sRestartDeploymentDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_restart_deployment";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Restart a k8s deployment by rolling restart";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to restart applications after config changes or to recover from issues";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "deployment".to_string(),
                param_type: "string".to_string(),
                description: "Deployment name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-app")),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
            "action": "k8s_restart_deployment",
            "parameters": {
                "deployment": "nginx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Deployment 'nginx' restarted successfully".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_restart_deployment driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let deployment = get_param_string(parameters, "deployment")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
        let mut args = vec!["rollout", "restart", "deployment", &deployment];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        debug!("Restarting deployment: {}", deployment);
        exec_kubectl(&args, kubeconfig, timeout).await?;
        info!("Deployment '{}' restarted successfully", deployment);
        return Ok(format!("Deployment '{}' restarted successfully", deployment));
    }
}
// ========== Cluster Management Drivers ==========
/// Driver for listing Kubernetes nodes
#[derive(Debug)]
pub struct K8sGetNodesDriver;
#[async_trait::async_trait]
impl Driver for K8sGetNodesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_get_nodes";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List k8s cluster nodes and their status";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to check node health, capacity, and resource utilization";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: wide, json, yaml".to_string(),
                required: false,
                default: Some(json!("wide")),
                example: Some(json!("json")),
                enum_values: Some(vec!["wide".to_string(), "json".to_string(), "yaml".to_string()]),
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
            "action": "k8s_get_nodes",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "NAME     STATUS   ROLES    AGE   VERSION   INTERNAL-IP   EXTERNAL-IP\nnode-1   Ready    master   10d   v1.28.0   192.168.1.10   <none>\nnode-2   Ready    worker   10d   v1.28.0   192.168.1.11   <none>".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_get_nodes driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let output = parameters.get("output").and_then(|v| v.as_str()).unwrap_or("wide");
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
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
        debug!("Listing cluster nodes");
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                let pretty =
                    serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
                info!("Successfully listed nodes in JSON format");
                return Ok(pretty);
            }
        }
        info!("Successfully listed nodes");
        return Ok(result);
    }
}
/// Driver for listing Kubernetes namespaces
#[derive(Debug)]
pub struct K8sGetNamespacesDriver;
#[async_trait::async_trait]
impl Driver for K8sGetNamespacesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_get_namespaces";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List k8s namespaces";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to see available namespaces and their status";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output format: json, yaml".to_string(),
                required: false,
                default: Some(json!("table")),
                example: Some(json!("json")),
                enum_values: Some(vec!["table".to_string(), "json".to_string(), "yaml".to_string()]),
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
            "action": "k8s_get_namespaces",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "NAME              STATUS   AGE\ndefault           Active   10d\nkube-system       Active   10d\nkube-public       Active   10d"
            .to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_get_namespaces driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let output = parameters.get("output").and_then(|v| v.as_str()).unwrap_or("table");
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
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
        debug!("Listing namespaces");
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        if output == "json" {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                let pretty =
                    serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
                info!("Successfully listed namespaces in JSON format");
                return Ok(pretty);
            }
        }
        info!("Successfully listed namespaces");
        return Ok(result);
    }
}
// ========== Resource Management Drivers ==========
/// Driver for applying Kubernetes YAML/JSON manifests
#[derive(Debug)]
pub struct K8sApplyYamlDriver;
#[async_trait::async_trait]
impl Driver for K8sApplyYamlDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_apply_yaml";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Apply a k8s YAML or JSON manifest";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to create or update k8s resources from manifests";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "manifest".to_string(),
                param_type: "string".to_string(),
                description: "YAML or JSON manifest content".to_string(),
                required: true,
                default: None,
                example: Some(json!("apiVersion: v1\nkind: Pod\nmetadata:\n  name: my-pod")),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
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
            "action": "k8s_apply_yaml",
            "parameters": {
                "manifest": "apiVersion: v1\nkind: Pod\nmetadata:\n  name: nginx\nspec:\n  containers:\n  - name: nginx\n    image: nginx:latest"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "pod/nginx created".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_apply_yaml driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let manifest = get_param_string(parameters, "manifest")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
        let mut args = vec!["apply", "-f", "-"];
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        debug!("Applying manifest");
        let opts = build_kubectl_env(kubeconfig).with_timeout(timeout);
        let result = exec_with_stdin_async("kubectl", &args, &manifest, Some(opts))
            .await
            .map_err(|e| DriverError::execution(format!("Apply failed: {}", e)))?;
        if !result.success {
            return Err(DriverError::execution(format!("Apply failed: {}", result.stderr)));
        }
        info!("Successfully applied manifest");
        return Ok(result.stdout);
    }
}
/// Driver for deleting Kubernetes resources
#[derive(Debug)]
pub struct K8sDeleteResourceDriver;
#[async_trait::async_trait]
impl Driver for K8sDeleteResourceDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "k8s_delete_resource";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Delete a k8s resource (pod, deployment, service, etc.)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to remove unwanted resources from the cluster";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "kubeconfig".to_string(),
                param_type: "string".to_string(),
                description: "Kubeconfig file path".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/path/to/kubeconfig".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "context".to_string(),
                param_type: "string".to_string(),
                description: "K8s context".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("prod-cluster".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "resource_type".to_string(),
                param_type: "string".to_string(),
                description: "Resource type (pod, deployment, service, configmap, secret, etc.)".to_string(),
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
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Resource name".to_string(),
                required: true,
                default: None,
                example: Some(json!("my-pod")),
                enum_values: None,
            },
            DriverParameter {
                name: "namespace".to_string(),
                param_type: "string".to_string(),
                description: "k8s namespace".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(json!("production")),
                enum_values: None,
            },
            DriverParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force delete (for pods)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
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
            "action": "k8s_delete_resource",
            "parameters": {
                "resource_type": "deployment",
                "name": "nginx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "deployment.apps/nginx deleted".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing k8s_delete_resource driver");
        // Extract required parameters
        let kubeconfig = parameters.get("kubeconfig").and_then(|v| v.as_str());
        let context = parameters.get("context").and_then(|v| v.as_str());
        let resource_type = get_param_string(parameters, "resource_type")?;
        let name = get_param_string(parameters, "name")?;
        let namespace = parameters.get("namespace").and_then(|v| v.as_str());
        let force = get_param_bool(parameters, "force", false);
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build kubectl command arguments
        let mut args = vec!["delete", &resource_type, &name];
        if force && resource_type == "pod" {
            args.push("--force");
            args.push("--grace-period=0");
        }
        if let Some(ns) = namespace {
            args.push("-n");
            args.push(ns);
        }
        debug!("Deleting {}: {}", resource_type, name);
        let result = exec_kubectl(&args, kubeconfig, timeout).await?;
        info!("Successfully deleted {}: {}", resource_type, name);
        return Ok(result);
    }
}
