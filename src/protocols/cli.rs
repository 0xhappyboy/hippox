use crate::core::Hippox;
use std::{io::Write, sync::Arc};
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn run_cli(hippox: Arc<Hippox>) -> anyhow::Result<()> {
    println!("\n🚀 Hippo CLI - Connected to Core");
    println!("Available skills:\n{}\n", hippox.list_skills());
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let input = line.trim();
                let result = hippox.process(input).await;
                if result.response == "goodbye" {
                    println!("Goodbye!");
                    break;
                }
                if result.matched {
                    println!("🦛 {}\n", result.response);
                } else {
                    println!("❌ {}\n", result.response);
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    Ok(())
}
