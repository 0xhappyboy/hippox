//! Generic command execution utilities.
//!
//! This module provides a pure, business-agnostic interface for executing external commands.
//! No business logic, no specific tools (docker/kubectl/etc.), just command execution.

use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use tokio::time::timeout;

/// Command execution options
#[derive(Debug, Clone, Default)]
pub struct ExecOptions {
    /// Working directory
    pub cwd: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Capture stdout
    pub capture_stdout: bool,
    /// Capture stderr
    pub capture_stderr: bool,
}

impl ExecOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn with_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        self
    }

    pub fn with_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        self
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl ExecResult {
    fn from_output(output: Output) -> Self {
        Self {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }

    pub fn ok() -> Self {
        Self {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            success: true,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        Self {
            exit_code: -1,
            stdout: String::new(),
            stderr: msg.clone(),
            success: false,
        }
    }
}

/// Execute a command (blocking)
pub fn exec(program: &str, args: &[&str], opts: Option<ExecOptions>) -> Result<ExecResult> {
    let opts = opts.unwrap_or_default();
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = opts.cwd {
        cmd.current_dir(dir);
    }
    for (k, v) in opts.env {
        cmd.env(k, v);
    }
    if opts.capture_stdout {
        cmd.stdout(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null());
    }
    if opts.capture_stderr {
        cmd.stderr(Stdio::piped());
    } else {
        cmd.stderr(Stdio::null());
    }
    let output = if let Some(timeout_secs) = opts.timeout_secs {
        let handle = std::thread::spawn(move || cmd.output());
        match std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(timeout_secs));
            handle
        })
        .join()
        {
            Ok(join_handle) => match join_handle.join() {
                Ok(Ok(output)) => output,
                Ok(Err(e)) => return Ok(ExecResult::error(format!("IO error: {}", e))),
                Err(_) => return Ok(ExecResult::error("Command thread panicked".to_string())),
            },
            Err(_) => {
                return Ok(ExecResult::error(format!(
                    "Timeout after {}s",
                    timeout_secs
                )));
            }
        }
    } else {
        cmd.output()?
    };
    Ok(ExecResult::from_output(output))
}

/// Execute a command (async with timeout)
pub async fn exec_async(
    program: &str,
    args: &[&str],
    opts: Option<ExecOptions>,
) -> Result<ExecResult> {
    let opts = opts.unwrap_or_default();
    let timeout_secs = opts.timeout_secs.unwrap_or(30);
    let program_owned = program.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let cwd = opts.cwd.clone();
    let env = opts.env.clone();
    let capture_stdout = opts.capture_stdout;
    let capture_stderr = opts.capture_stderr;
    let result = timeout(Duration::from_secs(timeout_secs), async move {
        tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(&program_owned);
            cmd.args(&args_owned);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            for (k, v) in env {
                cmd.env(k, v);
            }
            if capture_stdout {
                cmd.stdout(Stdio::piped());
            } else {
                cmd.stdout(Stdio::null());
            }
            if capture_stderr {
                cmd.stderr(Stdio::piped());
            } else {
                cmd.stderr(Stdio::null());
            }
            match cmd.output() {
                Ok(output) => ExecResult::from_output(output),
                Err(e) => ExecResult::error(format!("IO error: {}", e)),
            }
        })
        .await
        .unwrap_or_else(|_| ExecResult::error("Task spawn failed".to_string()))
    })
    .await;
    match result {
        Ok(r) => Ok(r),
        Err(_) => Ok(ExecResult::error(format!(
            "Timeout after {}s",
            timeout_secs
        ))),
    }
}

/// Execute and return stdout on success
pub fn exec_stdout(program: &str, args: &[&str]) -> Result<String> {
    let result = exec(program, args, None)?;
    if result.success {
        Ok(result.stdout)
    } else {
        anyhow::bail!("Command failed: {}", result.stderr)
    }
}

/// Execute and return stdout (async)
pub async fn exec_stdout_async(program: &str, args: &[&str]) -> Result<String> {
    let result = exec_async(program, args, None).await?;
    if result.success {
        Ok(result.stdout)
    } else {
        anyhow::bail!("Command failed: {}", result.stderr)
    }
}

/// Execute and check if successful
pub fn exec_check(program: &str, args: &[&str]) -> bool {
    exec(program, args, None)
        .map(|r| r.success)
        .unwrap_or(false)
}

/// Execute with stdin input
pub fn exec_with_stdin(
    program: &str,
    args: &[&str],
    stdin_content: &str,
    opts: Option<ExecOptions>,
) -> Result<ExecResult> {
    let opts = opts.unwrap_or_default();
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    if let Some(dir) = opts.cwd {
        cmd.current_dir(dir);
    }
    for (k, v) in opts.env {
        cmd.env(k, v);
    }
    if opts.capture_stdout {
        cmd.stdout(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null());
    }
    if opts.capture_stderr {
        cmd.stderr(Stdio::piped());
    } else {
        cmd.stderr(Stdio::null());
    }
    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(stdin_content.as_bytes())?;
    }
    let output = child.wait_with_output()?;
    Ok(ExecResult::from_output(output))
}

/// Execute with stdin input (async)
pub async fn exec_with_stdin_async(
    program: &str,
    args: &[&str],
    stdin_content: &str,
    opts: Option<ExecOptions>,
) -> Result<ExecResult> {
    let opts = opts.unwrap_or_default();
    let timeout_secs = opts.timeout_secs.unwrap_or(30);
    let program_owned = program.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let cwd = opts.cwd.clone();
    let env = opts.env.clone();
    let capture_stdout = opts.capture_stdout;
    let capture_stderr = opts.capture_stderr;
    let stdin_owned = stdin_content.to_string();
    let result = timeout(Duration::from_secs(timeout_secs), async move {
        tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(&program_owned);
            cmd.args(&args_owned);
            cmd.stdin(Stdio::piped());
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            for (k, v) in env {
                cmd.env(k, v);
            }
            if capture_stdout {
                cmd.stdout(Stdio::piped());
            } else {
                cmd.stdout(Stdio::null());
            }
            if capture_stderr {
                cmd.stderr(Stdio::piped());
            } else {
                cmd.stderr(Stdio::null());
            }
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => return ExecResult::error(format!("Spawn failed: {}", e)),
            };
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(stdin_owned.as_bytes());
            }
            match child.wait_with_output() {
                Ok(output) => ExecResult::from_output(output),
                Err(e) => ExecResult::error(format!("Wait failed: {}", e)),
            }
        })
        .await
        .unwrap_or_else(|_| ExecResult::error("Task spawn failed".to_string()))
    })
    .await;
    match result {
        Ok(r) => Ok(r),
        Err(_) => Ok(ExecResult::error(format!(
            "Timeout after {}s",
            timeout_secs
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        let result = exec("echo", &["hello"], None).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_false() {
        let result = exec("false", &[], None).unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_async_echo() {
        let result = exec_async("echo", &["hello"], None).await.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_with_stdin() {
        let result = exec_with_stdin("grep", &["hello"], "hello world\nfoo bar", None).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello world"));
    }
}
