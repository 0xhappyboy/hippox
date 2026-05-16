use crate::executors::Executor;
use crate::global::Config;
use crate::i18n;
use crate::protocols;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use crate::types::ProcessResult;
use langhub::LLMClient;
use langhub::types::ModelProvider;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;

/// Service configuration for which protocols to enable
#[derive(Clone)]
pub struct ServiceConfig {
    pub enable_cli: bool,
    pub enable_tcp: bool,
    pub enable_http: bool,
    pub enable_websocket: bool,
    pub enable_grpc: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            enable_cli: true,
            enable_tcp: false,
            enable_http: false,
            enable_websocket: false,
            enable_grpc: false,
        }
    }
}

#[derive(Clone)]
pub struct Hippox {
    scheduler: SkillScheduler,
    executor: Executor,
    conversations: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Hippox {
    pub async fn new(
        skills_dir: &str,
        provider: ModelProvider,
        lang: &str,
    ) -> anyhow::Result<Self> {
        i18n::set_language(lang);
        info!("Loading skills from: {}", skills_dir);
        let skills = SkillLoader::load_all(skills_dir)?;
        info!("🦛 Loaded {} skills", skills.len());
        for skill in &skills {
            info!("   - {}: {}", skill.name, skill.description);
        }
        let llm = LLMClient::new(provider)?;
        let scheduler = SkillScheduler::new(skills, llm);
        let executor = Executor::new();
        Ok(Self {
            scheduler,
            executor,
            conversations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the core with the given service configuration
    pub async fn start(self, config: ServiceConfig) -> anyhow::Result<()> {
        let core = Arc::new(self);
        if config.enable_cli {
            let core_cli = core.clone();
            tokio::spawn(async move {
                info!("Starting CLI interface");
                if let Err(e) = protocols::cli::run_cli(core_cli).await {
                    eprintln!("CLI error: {}", e);
                }
            });
        }
        if config.enable_tcp {
            let core_tcp = core.clone();
            tokio::spawn(async move {
                let addr = Config::tcp_address();
                info!("Starting TCP server on {}", addr);
                if let Err(e) = protocols::tcp::run_tcp_server(core_tcp, &addr).await {
                    eprintln!("TCP server error: {}", e);
                }
            });
        }
        if config.enable_http {
            let core_http = core.clone();
            tokio::spawn(async move {
                let addr = Config::http_address();
                info!("Starting HTTP server on http://{}", addr);
                if let Err(e) = protocols::http::run_http_server(core_http, &addr).await {
                    eprintln!("HTTP server error: {}", e);
                }
            });
        }
        if config.enable_websocket {
            let core_ws = core.clone();
            tokio::spawn(async move {
                let addr = Config::websocket_address();
                info!("Starting WebSocket server on ws://{}", addr);
                if let Err(e) = protocols::websocket::run_websocket_server(core_ws, &addr).await {
                    eprintln!("WebSocket server error: {}", e);
                }
            });
        }
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        info!("Shutting down...");
        Ok(())
    }

    pub async fn process(&self, input: &str) -> ProcessResult {
        let session_id = "default".to_string();
        let history = {
            let conversations = self.conversations.read().unwrap();
            conversations
                .get(&session_id)
                .map(|h| h.join("\n"))
                .unwrap_or_default()
        };
        let input_trimmed = input.trim();
        if input_trimmed == "clear" {
            let mut conversations = self.conversations.write().unwrap();
            conversations.remove(&session_id);
            return ProcessResult {
                response: t!("app.conversation_cleared").to_string(),
                matched: true,
                skill_name: None,
            };
        }
        if input_trimmed == "exit" || input_trimmed == "quit" {
            return ProcessResult {
                response: "goodbye".to_string(),
                matched: true,
                skill_name: None,
            };
        }
        if input_trimmed.is_empty() {
            return ProcessResult {
                response: String::new(),
                matched: false,
                skill_name: None,
            };
        }
        // First, try to parse as a skill call JSON from LLM
        if let Ok(call) = self.executor.parse_skill_call(input_trimmed) {
            match self.executor.execute(&call).await {
                Ok(response) => {
                    let mut conversations = self.conversations.write().unwrap();
                    let hist = conversations.entry(session_id).or_default();
                    hist.push(format!("User: {}", input));
                    hist.push(format!("Assistant: {}", response));
                    return ProcessResult {
                        response,
                        matched: true,
                        skill_name: Some(call.action),
                    };
                }
                Err(e) => {
                    return ProcessResult {
                        response: format!("Skill execution error: {}", e),
                        matched: false,
                        skill_name: None,
                    };
                }
            }
        }
        // Fallback to trigger-based skill selection
        match self.scheduler.select_skill(input).await {
            Ok(Some(skill)) => match self.scheduler.execute(skill, input, &history).await {
                Ok(response) => {
                    let mut conversations = self.conversations.write().unwrap();
                    let hist = conversations.entry(session_id).or_default();
                    hist.push(format!("User: {}", input));
                    hist.push(format!("Assistant: {}", response));
                    ProcessResult {
                        response,
                        matched: true,
                        skill_name: Some(skill.name.clone()),
                    }
                }
                Err(e) => ProcessResult {
                    response: t!("skill.error", e.to_string()),
                    matched: false,
                    skill_name: None,
                },
            },
            Ok(None) => match self.scheduler.fallback_chat(input).await {
                Ok(response) => ProcessResult {
                    response,
                    matched: false,
                    skill_name: None,
                },
                Err(e) => ProcessResult {
                    response: t!("skill.error", e.to_string()),
                    matched: false,
                    skill_name: None,
                },
            },
            Err(e) => ProcessResult {
                response: t!("skill.error", e.to_string()),
                matched: false,
                skill_name: None,
            },
        }
    }

    pub fn list_skills(&self) -> String {
        self.scheduler.list_skills()
    }
}
