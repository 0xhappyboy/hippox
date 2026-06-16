use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    ExecOptions, SkillCategory, exec_async,
    types::{Skill, SkillParameter},
};

/// A skill for executing system commands and capturing their output.
#[derive(Debug)]
pub struct ExecCommandSkill;

#[async_trait::async_trait]
impl Skill for ExecCommandSkill {
    fn name(&self) -> &str {
        "exec_command"
    }

    fn description(&self) -> &str {
        "Execute a system command and return its output"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to run a shell command, execute a script, install software, or perform system operations"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "The shell command to execute (can include arguments)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ls -la".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "args".to_string(),
                param_type: "array".to_string(),
                description: "Command arguments as an array (alternative to command string)"
                    .to_string(),
                required: false,
                default: None,
                example: Some(json!(["ls", "-la", "/home/user"])),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Command timeout in seconds (default 30)".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "working_dir".to_string(),
                param_type: "string".to_string(),
                description: "Working directory for the command".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/home/user".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "env".to_string(),
                param_type: "object".to_string(),
                description: "Environment variables to set for the command".to_string(),
                required: false,
                default: None,
                example: Some(json!({"PATH": "/usr/local/bin", "DEBUG": "1"})),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "exec_command",
            "parameters": {
                "command": "echo 'Hello World'"
            }
        })
    }

    fn example_output(&self) -> String {
        "Hello World".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Terminal
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        let working_dir = parameters.get("working_dir").and_then(|v| v.as_str());
        let mut opts = ExecOptions::new()
            .with_timeout(timeout_secs)
            .with_stdout(true)
            .with_stderr(true);
        if let Some(dir) = working_dir {
            opts = opts.with_cwd(dir);
        }
        if let Some(env_obj) = parameters.get("env").and_then(|v| v.as_object()) {
            for (key, value) in env_obj {
                if let Some(val_str) = value.as_str() {
                    opts = opts.with_env(key, val_str);
                }
            }
        }
        let result = if let Some(args_array) = parameters.get("args").and_then(|v| v.as_array()) {
            if args_array.is_empty() {
                anyhow::bail!("Args array is empty");
            }
            let program = args_array[0]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("First arg must be a string"))?;
            let args: Vec<&str> = args_array[1..].iter().filter_map(|v| v.as_str()).collect();
            exec_async(program, &args, Some(opts)).await?
        } else if let Some(cmd_str) = parameters.get("command").and_then(|v| v.as_str()) {
            #[cfg(target_family = "unix")]
            let (program, args) = ("/bin/sh", vec!["-c", cmd_str]);
            #[cfg(target_family = "windows")]
            let (program, args) = ("cmd.exe", vec!["/c", cmd_str]);
            exec_async(program, &args, Some(opts)).await?
        } else {
            anyhow::bail!("Missing required parameter: either 'command' or 'args'");
        };
        if result.success {
            if result.stdout.is_empty() && !result.stderr.is_empty() {
                Ok(format!(
                    "Command executed successfully (stderr):\n{}",
                    result.stderr
                ))
            } else {
                Ok(result.stdout)
            }
        } else {
            Ok(format!(
                "Command failed with exit code {}:\nstdout: {}\nstderr: {}",
                result.exit_code, result.stdout, result.stderr
            ))
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        let has_command = parameters.contains_key("command");
        let has_args = parameters.contains_key("args");
        if !has_command && !has_args {
            anyhow::bail!("Missing required parameter: either 'command' or 'args'");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_parameters() {
        let skill = ExecCommandSkill;
        let mut params = HashMap::new();
        params.insert("command".to_string(), json!("echo test"));
        assert!(skill.validate(&params).is_ok());
        let mut params = HashMap::new();
        params.insert("args".to_string(), json!(["echo", "test"]));
        assert!(skill.validate(&params).is_ok());
        let params = HashMap::new();
        assert!(skill.validate(&params).is_err());
    }

    #[tokio::test]
    async fn test_execute_simple_command() {
        let skill = ExecCommandSkill;
        let mut params = HashMap::new();
        params.insert("command".to_string(), json!("echo Hello World"));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_execute_with_args_array() {
        let skill = ExecCommandSkill;
        let mut params = HashMap::new();
        #[cfg(target_family = "unix")]
        let args = json!(["echo", "Testing", "args", "array"]);
        #[cfg(target_family = "windows")]
        let args = json!(["cmd.exe", "/c", "echo", "Testing", "args", "array"]);
        params.insert("args".to_string(), args);
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Testing") && output.contains("args") && output.contains("array"));
    }
}
