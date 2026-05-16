use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::executors::types::{Skill, SkillParameter};

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

    fn category(&self) -> &str {
        "system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let timeout_secs = parameters
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        let working_dir = parameters.get("working_dir").and_then(|v| v.as_str());
        // Build command
        let (program, args) =
            if let Some(args_array) = parameters.get("args").and_then(|v| v.as_array()) {
                // Using args array
                if args_array.is_empty() {
                    anyhow::bail!("Args array is empty");
                }
                let program = args_array[0]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("First arg must be a string"))?;
                let args: Vec<&str> = args_array[1..].iter().filter_map(|v| v.as_str()).collect();
                (program.to_string(), args)
            } else if let Some(cmd_str) = parameters.get("command").and_then(|v| v.as_str()) {
                // Using command string - use shell
                #[cfg(target_family = "unix")]
                let (program, args) = ("/bin/sh".to_string(), vec!["-c", cmd_str]);
                #[cfg(target_family = "windows")]
                let (program, args) = ("cmd.exe".to_string(), vec!["/c", cmd_str]);
                (program, args)
            } else {
                anyhow::bail!("Missing required parameter: either 'command' or 'args'");
            };
        let mut cmd = Command::new(&program);
        cmd.args(&args);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        if let Some(env_obj) = parameters.get("env").and_then(|v| v.as_object()) {
            for (key, value) in env_obj {
                if let Some(val_str) = value.as_str() {
                    cmd.env(key, val_str);
                }
            }
        }
        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", program, e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            if stdout.is_empty() && !stderr.is_empty() {
                Ok(format!(
                    "Command executed successfully (stderr):\n{}",
                    stderr
                ))
            } else {
                Ok(stdout.to_string())
            }
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            Ok(format!(
                "Command failed with exit code {}:\nstdout: {}\nstderr: {}",
                exit_code, stdout, stderr
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
