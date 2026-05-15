use crate::i18n;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use crate::types::ProcessResult;
use langhub::LLMClient;
use langhub::types::ModelProvider;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Clone)]
pub struct Core {
    scheduler: SkillScheduler,
    conversations: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Core {
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
        Ok(Self {
            scheduler,
            conversations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        println!("\n🦛 {}", t!("app.running"));
        println!("{}", t!("app.available_skills"));
        println!("{}", self.scheduler.list_skills());
        println!("\n💡 {}", t!("app.try_saying"));
        println!("   {}\n", t!("app.type_exit"));
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin);
        let mut line = String::new();
        let session_id = "default".to_string();
        loop {
            print!("> ");
            std::io::Write::flush(&mut std::io::stdout())?;
            line.clear();
            match tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    let input = line.trim();
                    if input == "exit" || input == "quit" {
                        println!("{}", t!("app.goodbye"));
                        break;
                    }
                    if input == "clear" {
                        let mut conversations = self.conversations.write().unwrap();
                        conversations.remove(&session_id);
                        println!("{}\n", t!("app.conversation_cleared"));
                        continue;
                    }
                    if input.is_empty() {
                        continue;
                    }
                    let history = {
                        let conversations = self.conversations.read().unwrap();
                        conversations
                            .get(&session_id)
                            .map(|h| h.join("\n"))
                            .unwrap_or_default()
                    };
                    match self.scheduler.select_skill(input).await {
                        Ok(Some(skill)) => {
                            let response = self.scheduler.execute(skill, input, &history).await?;
                            println!("🦛 {}\n", response);
                            let mut conversations = self.conversations.write().unwrap();
                            let hist = conversations.entry(session_id.clone()).or_default();
                            hist.push(format!("User: {}", input));
                            hist.push(format!("Assistant: {}", response));
                        }
                        Ok(None) => {
                            println!("❌ {}", t!("skill.no_match", input));
                            let response = self.scheduler.fallback_chat(input).await?;
                            println!("🦛 {}\n", response);
                        }
                        Err(e) => {
                            println!("❌ {}", t!("skill.error", e.to_string()));
                        }
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
        // Match all skills
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
