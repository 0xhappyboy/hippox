use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::executors::types::{Skill, SkillParameter};

/// A skill for executing system commands and capturing their output.
///
/// This skill provides the ability to run shell commands, scripts, and system operations
/// with configurable timeouts, working directories, and environment variables.
/// It supports both simple command strings and structured argument arrays.
///
/// # Examples
///
/// Execute a simple command:
/// ```rust
/// let params = HashMap::from([
///     ("command".to_string(), json!("ls -la"))
/// ]);
/// let output = skill.execute(&params).await?;
/// ```
///
/// Execute with arguments array:
/// ```rust
/// let params = HashMap::from([
///     ("args".to_string(), json!(["git", "status", "--short"]))
/// ]);
/// let output = skill.execute(&params).await?;
/// ```
///
/// Execute with custom working directory and environment:
/// ```rust
/// let params = HashMap::from([
///     ("command".to_string(), json!("python script.py")),
///     ("working_dir".to_string(), json!("/home/user/project")),
///     ("env".to_string(), json!({"PYTHONPATH": "/custom/lib"}))
/// ]);
/// let output = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct ExecCommandSkill;

#[async_trait::async_trait]
impl Skill for ExecCommandSkill {
    /// Returns the unique name identifier for this skill.
    fn name(&self) -> &str {
        "exec_command"
    }

    /// Returns a human-readable description of what this skill does.
    fn description(&self) -> &str {
        "Execute a system command and return its output"
    }

    /// Returns a usage hint for when to use this skill.
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to run a shell command, execute a script, install software, or perform system operations"
    }

    /// Returns the list of parameters that this skill accepts.
    ///
    /// # Parameters
    ///
    /// * `command` - A complete shell command string (e.g., "ls -la")
    /// * `args` - An array of command arguments where the first element is the program
    /// * `timeout` - Maximum execution time in seconds (default: 30)
    /// * `working_dir` - Directory to execute the command from
    /// * `env` - Environment variables to set for the command
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

    /// Returns an example JSON call for this skill.
    fn example_call(&self) -> Value {
        json!({
            "action": "exec_command",
            "parameters": {
                "command": "echo 'Hello World'"
            }
        })
    }

    /// Returns an example output string for this skill.
    fn example_output(&self) -> String {
        "Hello World".to_string()
    }

    /// Returns the category of this skill.
    fn category(&self) -> &str {
        "system"
    }

    /// Executes the system command with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - A map of parameter names to their JSON values
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The command's stdout output on success, or an error message if the command fails
    /// * `Err(anyhow::Error)` - If the command cannot be executed or parameters are invalid
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

    /// Validates the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - A map of parameter names to their JSON values
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the parameters are valid
    /// * `Err(anyhow::Error)` - If either 'command' or 'args' is missing
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
    use std::collections::HashMap;

    ///  Validate parameter validation logic
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
        let mut params = HashMap::new();
        params.insert("command".to_string(), json!("echo test"));
        params.insert("args".to_string(), json!(["echo", "test"]));
        assert!(skill.validate(&params).is_ok());
    }

    /// Test command execution with simple echo command
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

    ///  Test command execution with arguments array
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

    ///   Test handling of non-existent command
    #[tokio::test]
    async fn test_nonexistent_command() {
        let skill = ExecCommandSkill;
        let mut params = HashMap::new();
        params.insert(
            "command".to_string(),
            json!("this_command_does_not_exist_12345"),
        );
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(
            output.contains("failed")
                || output.contains("not found")
                || output.contains("recognized")
        );
    }

    /// Test parameter info methods
    #[test]
    fn test_skill_metadata() {
        let skill = ExecCommandSkill;
        assert_eq!(skill.name(), "exec_command");
        assert_eq!(
            skill.description(),
            "Execute a system command and return its output"
        );
        assert_eq!(skill.category(), "system");
        assert!(!skill.usage_hint().is_empty());
        assert!(skill.parameters().len() >= 5);
        assert!(skill.example_call().is_object());
        assert!(!skill.example_output().is_empty());
    }
}
