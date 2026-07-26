//! SSH execution utilities
//!
//! This module provides SSH execution functionality for running commands
//! on remote hosts using the ssh2 crate.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::{io::Read, time::Duration};
use tracing::{debug, info, warn};
/// SSH execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
/// Execute a command on a remote host via SSH
///
/// # Arguments
/// * `host` - Remote hostname or IP address
/// * `port` - SSH port
/// * `username` - SSH username
/// * `password` - SSH password (optional if key_path is provided)
/// * `key_path` - Path to SSH private key (optional if password is provided)
/// * `command` - Command to execute
/// * `timeout_secs` - Timeout in seconds
///
/// # Returns
/// `DriverResult<SshExecResult>` containing stdout, stderr, and exit code
pub async fn ssh_exec(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    command: &str,
    timeout_secs: u64,
) -> DriverResult<SshExecResult> {
    use ssh2::Session;
    use std::net::TcpStream;
    use tokio::task::spawn_blocking;
    debug!("SSH exec: host={}, port={}, username={}, timeout={}s", host, port, username, timeout_secs);
    let host = host.to_string();
    let port = port;
    let username = username.to_string();
    let password = password.map(|s| s.to_string());
    let key_path = key_path.map(|s| s.to_string());
    let command = command.to_string();
    let timeout_secs = timeout_secs;
    if password.is_none() && key_path.is_none() {
        let err_msg = "No authentication method provided".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("authentication", err_msg));
    }
    let result = spawn_blocking(move || {
        let addr = format!("{}:{}", host, port);
        debug!("Connecting to {}:{}", host, port);
        let tcp = match TcpStream::connect(&addr) {
            Ok(t) => {
                debug!("TCP connection established");
                t
            }
            Err(e) => {
                let err_msg = format!("Failed to connect to {}:{}: {}", host, port, e);
                warn!("{}", err_msg);
                return Err::<_, String>(err_msg);
            }
        };
        if let Err(e) = tcp.set_read_timeout(Some(Duration::from_secs(timeout_secs))) {
            warn!("Failed to set read timeout: {}", e);
        }
        if let Err(e) = tcp.set_write_timeout(Some(Duration::from_secs(timeout_secs))) {
            warn!("Failed to set write timeout: {}", e);
        }
        let mut session = match Session::new() {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!("Failed to create SSH session: {}", e);
                warn!("{}", err_msg);
                return Err(err_msg);
            }
        };
        session.set_tcp_stream(tcp);
        if let Err(e) = session.handshake() {
            let err_msg = format!("SSH handshake failed: {}", e);
            warn!("{}", err_msg);
            return Err(err_msg);
        }
        debug!("SSH handshake completed");
        // Authenticate
        if let Some(key_path) = key_path {
            debug!("Authenticating with public key: {}", key_path);
            if let Err(e) = session.userauth_pubkey_file(&username, None, key_path.as_ref(), None) {
                let err_msg = format!("Public key authentication failed: {}", e);
                warn!("{}", err_msg);
                return Err(err_msg);
            }
            debug!("Public key authentication successful");
        } else if let Some(password) = password {
            debug!("Authenticating with password");
            if let Err(e) = session.userauth_password(&username, &password) {
                let err_msg = format!("Password authentication failed: {}", e);
                warn!("{}", err_msg);
                return Err(err_msg);
            }
            debug!("Password authentication successful");
        } else {
            let err_msg = "No authentication method provided".to_string();
            warn!("{}", err_msg);
            return Err(err_msg);
        }
        if !session.authenticated() {
            let err_msg = "Authentication failed".to_string();
            warn!("{}", err_msg);
            return Err(err_msg);
        }
        debug!("Opening channel for command: {}", command);
        let mut channel = match session.channel_session() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to open SSH channel: {}", e);
                warn!("{}", err_msg);
                return Err(err_msg);
            }
        };
        if let Err(e) = channel.exec(&command) {
            let err_msg = format!("Failed to execute command: {}", e);
            warn!("{}", err_msg);
            return Err(err_msg);
        }
        debug!("Command executed");
        // Read stdout
        let mut stdout = String::new();
        let mut buf = [0u8; 4096];
        loop {
            match channel.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    stdout.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    let err_msg = format!("Failed to read stdout: {}", e);
                    warn!("{}", err_msg);
                    break;
                }
            }
        }
        // Read stderr
        let mut stderr = String::new();
        let mut stderr_channel = channel.stderr();
        loop {
            match stderr_channel.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    stderr.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
            }
        }
        let exit_code = match channel.exit_status() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to get exit status: {}", e);
                warn!("{}", err_msg);
                return Err(err_msg);
            }
        };
        info!("SSH command completed: exit_code={}", exit_code);
        let _ = channel.close();
        let _ = session.disconnect(None, "Goodbye", None);
        return Ok::<_, String>(SshExecResult { stdout, stderr, exit_code });
    })
    .await;
    match result {
        Ok(Ok(r)) => return Ok(r),
        Ok(Err(e)) => return Err(DriverError::execution(e)),
        Err(e) => {
            let err_msg = format!("Task panicked: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    }
}
