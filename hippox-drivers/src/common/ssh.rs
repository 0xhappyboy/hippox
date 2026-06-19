//! SSH execution utilities

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{io::Read, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub async fn ssh_exec(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    command: &str,
    timeout_secs: u64,
) -> Result<SshExecResult> {
    use ssh2::Session;
    use std::net::TcpStream;
    use tokio::task::spawn_blocking;
    let host = host.to_string();
    let port = port;
    let username = username.to_string();
    let password = password.map(|s| s.to_string());
    let key_path = key_path.map(|s| s.to_string());
    let command = command.to_string();
    let timeout_secs = timeout_secs;
    let result = spawn_blocking(move || {
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr)?;
        tcp.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(timeout_secs)))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        if let Some(key_path) = key_path {
            session.userauth_pubkey_file(&username, None, key_path.as_ref(), None)?;
        } else if let Some(password) = password {
            session.userauth_password(&username, &password)?;
        } else {
            anyhow::bail!("No authentication method provided");
        }
        if !session.authenticated() {
            anyhow::bail!("Authentication failed");
        }
        let mut channel = session.channel_session()?;
        channel.exec(&command)?;
        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = channel.read(&mut buf)?;
            if n == 0 {
                break;
            }
            stdout.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
        let mut stderr_channel = channel.stderr();
        loop {
            let n = stderr_channel.read(&mut buf)?;
            if n == 0 {
                break;
            }
            stderr.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
        let exit_code = channel.exit_status()?;
        channel.wait_close()?;
        Ok::<_, anyhow::Error>(SshExecResult {
            stdout,
            stderr,
            exit_code,
        })
    })
    .await??;
    Ok(result)
}
