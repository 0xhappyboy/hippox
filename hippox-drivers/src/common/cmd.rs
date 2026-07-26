// cmd.rs
//! Generic command execution utilities.
//!
//! This module provides a pure, business-agnostic interface for executing external commands.
//! No business logic, no specific tools (docker/kubectl/etc.), just command execution.
use crate::DriverError;
use crate::result::DriverResult;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};
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
    /// Create a new empty options instance
    pub fn new() -> Self {
        debug!("Creating new ExecOptions instance");
        return Self::default();
    }
    /// Set working directory
    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        return self;
    }
    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        return self;
    }
    /// Set timeout in seconds
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        return self;
    }
    /// Set whether to capture stdout
    pub fn with_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        return self;
    }
    /// Set whether to capture stderr
    pub fn with_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        return self;
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
    /// Create ExecResult from process Output
    fn from_output(output: Output) -> Self {
        return Self {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        };
    }
    /// Create a successful result
    pub fn ok() -> Self {
        return Self { exit_code: 0, stdout: String::new(), stderr: String::new(), success: true };
    }
    /// Create an error result
    pub fn error(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        return Self { exit_code: -1, stdout: String::new(), stderr: msg.clone(), success: false };
    }
}
/// Execute a command (blocking)
pub fn exec(program: &str, args: &[&str], opts: Option<ExecOptions>) -> DriverResult<ExecResult> {
    debug!("Executing command: {} with {:?} args", program, args);
    let opts = opts.unwrap_or_default();
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = opts.cwd {
        cmd.current_dir(dir.clone());
        debug!("Set working directory: {}", dir);
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
        debug!("Command will timeout after {} seconds", timeout_secs);
        let handle = std::thread::spawn(move || cmd.output());
        match std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(timeout_secs));
            return handle;
        })
        .join()
        {
            Ok(join_handle) => match join_handle.join() {
                Ok(Ok(output)) => output,
                Ok(Err(e)) => {
                    let err_msg = format!("IO error: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
                Err(_) => {
                    let err_msg = "Command thread panicked".to_string();
                    warn!("{}", err_msg);
                    return Err(DriverError::internal(err_msg));
                }
            },
            Err(_) => {
                let err_msg = format!("Timeout after {}s", timeout_secs);
                warn!("{}", err_msg);
                return Err(DriverError::timeout(Some(err_msg)));
            }
        }
    } else {
        match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                let err_msg = format!("Failed to execute command: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    };
    let result = ExecResult::from_output(output);
    info!("Command executed: exit_code={}, success={}", result.exit_code, result.success);
    return Ok(result);
}
/// Execute a command (async with timeout)
pub async fn exec_async(program: &str, args: &[&str], opts: Option<ExecOptions>) -> DriverResult<ExecResult> {
    debug!("Executing async command: {} with {:?} args", program, args);
    let opts = opts.unwrap_or_default();
    let timeout_secs = opts.timeout_secs.unwrap_or(30);
    let program_owned = program.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let cwd = opts.cwd.clone();
    let env = opts.env.clone();
    let capture_stdout = opts.capture_stdout;
    let capture_stderr = opts.capture_stderr;
    info!("Async command timeout: {}s", timeout_secs);
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
        Ok(r) => {
            info!("Async command completed: exit_code={}", r.exit_code);
            return Ok(r);
        }
        Err(_) => {
            let err_msg = format!("Timeout after {}s", timeout_secs);
            warn!("{}", err_msg);
            return Err(DriverError::timeout(Some(err_msg)));
        }
    }
}
/// Execute and return stdout on success
pub fn exec_stdout(program: &str, args: &[&str]) -> DriverResult<String> {
    debug!("Executing stdout command: {}", program);
    let result = exec(program, args, None)?;
    if result.success {
        info!("Stdout command succeeded: {}", program);
        return Ok(result.stdout);
    } else {
        let err_msg = format!("Command failed: {}", result.stderr);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
/// Execute and return stdout (async)
pub async fn exec_stdout_async(program: &str, args: &[&str]) -> DriverResult<String> {
    debug!("Executing async stdout command: {}", program);
    let result = exec_async(program, args, None).await?;
    if result.success {
        info!("Async stdout command succeeded: {}", program);
        return Ok(result.stdout);
    } else {
        let err_msg = format!("Command failed: {}", result.stderr);
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
}
/// Execute and check if successful
pub fn exec_check(program: &str, args: &[&str]) -> bool {
    debug!("Checking command: {}", program);
    let success = exec(program, args, None).map(|r| r.success).unwrap_or(false);
    info!("Command check result: {}", success);
    return success;
}
/// Execute with stdin input
pub fn exec_with_stdin(program: &str, args: &[&str], stdin_content: &str, opts: Option<ExecOptions>) -> DriverResult<ExecResult> {
    debug!("Executing with stdin: {}", program);
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
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to spawn command: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(stdin_content.as_bytes()) {
            let err_msg = format!("Failed to write stdin: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            let err_msg = format!("Failed to wait for output: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    let result = ExecResult::from_output(output);
    info!("Command with stdin completed: exit_code={}", result.exit_code);
    return Ok(result);
}
/// Execute with stdin input (async)
pub async fn exec_with_stdin_async(program: &str, args: &[&str], stdin_content: &str, opts: Option<ExecOptions>) -> DriverResult<ExecResult> {
    debug!("Executing async with stdin: {}", program);
    let opts = opts.unwrap_or_default();
    let timeout_secs = opts.timeout_secs.unwrap_or(30);
    let program_owned = program.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let cwd = opts.cwd.clone();
    let env = opts.env.clone();
    let capture_stdout = opts.capture_stdout;
    let capture_stderr = opts.capture_stderr;
    let stdin_owned = stdin_content.to_string();
    info!("Async stdin command timeout: {}s", timeout_secs);
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
        Ok(r) => {
            info!("Async stdin command completed: exit_code={}", r.exit_code);
            return Ok(r);
        }
        Err(_) => {
            let err_msg = format!("Timeout after {}s", timeout_secs);
            warn!("{}", err_msg);
            return Err(DriverError::timeout(Some(err_msg)));
        }
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
