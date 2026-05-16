use std::sync::Arc;

use crate::core::Hippox;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

pub async fn run_tcp_server(hippox: Arc<Hippox>, addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("TCP server listening on {}", addr);
    loop {
        let (stream, addr) = listener.accept().await?;
        let hippox = hippox.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_tcp_client(hippox, stream).await {
                error!("TCP client {} error: {}", addr, e);
            }
        });
    }
}

async fn handle_tcp_client(hippox: Arc<Hippox>, mut stream: TcpStream) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    writer
        .write_all(b"Hippo TCP Server\nAvailable skills: \n")
        .await?;
    writer.write_all(hippox.list_skills().as_bytes()).await?;
    writer.write_all(b"\n> ").await?;
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let input = line.trim();
                let result = hippox.process(input).await;
                if result.response == "goodbye" {
                    writer.write_all(b"Goodbye!\n").await?;
                    break;
                }
                if result.matched {
                    writer
                        .write_all(format!("🦛 {}\n> ", result.response).as_bytes())
                        .await?;
                } else {
                    writer
                        .write_all(format!("❌ {}\n> ", result.response).as_bytes())
                        .await?;
                }
            }
            Err(e) => {
                writer
                    .write_all(format!("Error: {}\n", e).as_bytes())
                    .await?;
                break;
            }
        }
    }
    Ok(())
}
