mod core;
mod global;
mod i18n;
mod protocols;
mod service;
mod skill_loader;
mod skill_scheduler;
mod types;

use langhub::types::ModelProvider;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    i18n::init();
    let lang = env::var("HIPPO_LANG").unwrap_or_else(|_| "en".to_string());
    let provider = match env::var("HIPPO_PROVIDER").as_deref() {
        Ok("deepseek") => ModelProvider::DeepSeek,
        Ok("anthropic") => ModelProvider::Anthropic,
        Ok("google") => ModelProvider::Google,
        _ => ModelProvider::OpenAI,
    };
    let mut hippo = core::Core::new("skills", provider, &lang).await?;
    hippo.run().await?;
    Ok(())
}
