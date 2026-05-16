mod core;
mod executors;
mod global;
mod i18n;
mod protocols;
mod skill_loader;
mod skill_scheduler;
mod types;

pub use core::{Core, ServiceConfig};
pub use langhub::types::ModelProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio;

    #[tokio::test]
    async fn test_main_logic() -> anyhow::Result<()> {
        tracing_subscriber::fmt().init();
        i18n::init();
        let lang = env::var("HIPPO_LANG").unwrap_or_else(|_| "en".to_string());
        let provider = match env::var("HIPPO_LLM_PROVIDER_KEY").as_deref() {
            Ok("deepseek") => ModelProvider::DeepSeek,
            Ok("anthropic") => ModelProvider::Anthropic,
            Ok("google") => ModelProvider::Google,
            _ => ModelProvider::OpenAI,
        };
        let core = Core::new("skills", provider, &lang).await?;
        // Configure which protocols to enable
        let config = ServiceConfig {
            enable_cli: env::var("HIPPO_ENABLE_CLI")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true),
            enable_tcp: env::var("HIPPO_ENABLE_TCP")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            enable_http: env::var("HIPPO_ENABLE_HTTP")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            enable_websocket: env::var("HIPPO_ENABLE_WS")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            enable_grpc: false,
        };
        core.start(config).await?;
        Ok(())
    }
}
