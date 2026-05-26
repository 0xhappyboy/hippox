use crate::config::{
    init_config_from_json_file, init_config_from_params_json_str, init_config_from_toml_file,
};
use crate::executors::Executor;
use crate::memory::ConversationMemory;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::workflow::{WorkflowCallback, WorkflowExecutor, WorkflowMode};
use crate::{HippoxConfig, i18n};
use crate::{get_config, t};
use langhub::LLMClient;
use langhub::types::ModelProvider;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

const STARTUP_BANNER: &str = r#"
██╗  ██╗██╗██████╗ ██████╗  ██████╗ ██╗  ██╗
██║  ██║██║██╔══██╗██╔══██╗██╔═══██╗╚██╗██╔╝
███████║██║██████╔╝██████╔╝██║   ██║ ╚███╔╝ 
██╔══██║██║██╔═══╝ ██╔═══╝ ██║   ██║ ██╔██╗ 
██║  ██║██║██║     ██║     ╚██████╔╝██╔╝ ██╗
╚═╝  ╚═╝╚═╝╚═╝     ╚═╝      ╚═════╝ ╚═╝  ╚═╝
"#;

pub enum ConfigInitMethod {
    TomlFile(String),
    JsonFile(String),
    ParamsJsonStr(String),
}

/// Welcome message structure containing registry information
#[derive(Debug, Clone, serde::Serialize)]
pub struct WelcomeMessage {
    pub type_: String,
    pub message: String,
    pub skills_registry: Value,
    pub instances_registry: Value,
    pub version: String,
}

/// Core engine for Hippox
///
/// This is the main entry point for the Hippox engine. It handles:
/// - Natural language processing with atomic skill registry
/// - SKILL.md file execution for complex workflows
/// - Managing conversation history for natural language interactions
#[derive(Clone)]
pub struct Hippox {
    scheduler: SkillScheduler,
    executor: Executor,
    memory: ConversationMemory,
    skills_dir: PathBuf,
    workflow_mode: WorkflowMode,
    workflow_executor: WorkflowExecutor,
    // Cached registries (generated once at initialization)
    cached_skills_registry: String,
    cached_instances_registry: String,
    cached_welcome_message: String,
    is_first_message: Arc<AtomicBool>,
}

impl Hippox {
    /// Create a new Hippox core instance with default ReAct workflow mode
    pub async fn new(
        skills_dir: &str,
        provider: ModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config_method: ConfigInitMethod,
    ) -> anyhow::Result<Self> {
        Self::with_workflow_mode(
            skills_dir,
            provider,
            api_key,
            extra_keys,
            config_method,
            WorkflowMode::default(),
        )
        .await
    }

    /// Create a new Hippox core instance with specified workflow mode
    pub async fn with_workflow_mode(
        skills_dir: &str,
        provider: ModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config_method: ConfigInitMethod,
        workflow_mode: WorkflowMode,
    ) -> anyhow::Result<Self> {
        info!(
            "Initializing Hippox core with skills directory: {}, workflow mode: {}",
            skills_dir, workflow_mode
        );
        // init config
        match config_method {
            ConfigInitMethod::TomlFile(path) => init_config_from_toml_file(&path)?,
            ConfigInitMethod::JsonFile(path) => init_config_from_json_file(&path)?,
            ConfigInitMethod::ParamsJsonStr(json) => init_config_from_params_json_str(&json)?,
        }
        // set i18n
        let config = get_config();
        i18n::set_language(&config.lang);
        // init llm
        let llm = LLMClient::new_with_key(provider, api_key, extra_keys)?;
        // Generate cached registries (once at startup)
        let skills_registry = Self::generate_skills_registry();
        let instances_registry = Self::generate_instances_registry();
        let welcome_message = Self::generate_welcome_message(&skills_registry, &instances_registry);
        // init llm scheduler
        let scheduler = SkillScheduler::new(llm);
        let executor = Executor::new();
        let workflow_executor = WorkflowExecutor::new(workflow_mode);
        Ok(Self {
            scheduler,
            executor,
            memory: ConversationMemory::new(),
            skills_dir: PathBuf::from(skills_dir),
            workflow_mode,
            workflow_executor,
            cached_skills_registry: skills_registry,
            cached_instances_registry: instances_registry,
            cached_welcome_message: welcome_message,
            is_first_message: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Generate skills registry (atomic skills metadata)
    fn generate_skills_registry() -> String {
        let skills = crate::executors::registry::list_skills();
        let registry: Vec<serde_json::Value> = skills
            .iter()
            .filter_map(|name| {
                crate::executors::registry::get_skill(name).map(|skill| {
                    serde_json::json!({
                        "name": name,
                        "description": skill.description(),
                        "category": skill.category(),
                        "parameters": skill.parameters(),
                        "example_call": skill.example_call(),
                        "example_output": skill.example_output(),
                    })
                })
            })
            .collect();
        serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
    }

    /// Generate instances registry (configured database/service instances)
    fn generate_instances_registry() -> String {
        let config = get_config();
        let mut instances = serde_json::Map::new();

        // PostgreSQL instances
        let pg_instances: Vec<serde_json::Value> = config
            .postgresql_instances
            .values()
            .map(|inst| {
                json!({
                    "id": inst.id,
                    "name": inst.name,
                    "description": inst.description,
                    "type": "postgresql"
                })
            })
            .collect();
        if !pg_instances.is_empty() {
            instances.insert("postgresql".to_string(), json!(pg_instances));
        }

        // MySQL instances
        let mysql_instances: Vec<serde_json::Value> = config
            .mysql_instances
            .values()
            .map(|inst| {
                json!({
                    "id": inst.id,
                    "name": inst.name,
                    "description": inst.description,
                    "type": "mysql"
                })
            })
            .collect();
        if !mysql_instances.is_empty() {
            instances.insert("mysql".to_string(), json!(mysql_instances));
        }

        // Redis instances
        let redis_instances: Vec<serde_json::Value> = config
            .redis_instances
            .values()
            .map(|inst| {
                json!({
                    "id": inst.id,
                    "name": inst.name,
                    "description": inst.description,
                    "type": "redis"
                })
            })
            .collect();
        if !redis_instances.is_empty() {
            instances.insert("redis".to_string(), json!(redis_instances));
        }

        // SQLite instances
        let sqlite_instances: Vec<serde_json::Value> = config
            .sqlite_instances
            .values()
            .map(|inst| {
                json!({
                    "id": inst.id,
                    "name": inst.name,
                    "description": inst.description,
                    "type": "sqlite"
                })
            })
            .collect();
        if !sqlite_instances.is_empty() {
            instances.insert("sqlite".to_string(), json!(sqlite_instances));
        }

        // Docker instances
        let docker_instances: Vec<serde_json::Value> = config
            .docker_instances
            .values()
            .map(|inst| {
                json!({
                    "id": inst.id,
                    "name": inst.name,
                    "description": inst.description,
                    "type": "docker"
                })
            })
            .collect();
        if !docker_instances.is_empty() {
            instances.insert("docker".to_string(), json!(docker_instances));
        }

        // Kubernetes instances
        let k8s_instances: Vec<serde_json::Value> = config
            .k8s_instances
            .values()
            .map(|inst| {
                json!({
                    "id": inst.id,
                    "name": inst.name,
                    "description": inst.description,
                    "type": "kubernetes",
                    "namespace": inst.namespace
                })
            })
            .collect();
        if !k8s_instances.is_empty() {
            instances.insert("kubernetes".to_string(), json!(k8s_instances));
        }

        serde_json::to_string_pretty(&instances).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate welcome message with registries
    fn generate_welcome_message(skills_registry: &str, instances_registry: &str) -> String {
        let welcome = WelcomeMessage {
            type_: "welcome".to_string(),
            message: format!("{}\n\n{}", STARTUP_BANNER, t!("app.welcome_message")),
            skills_registry: serde_json::from_str(skills_registry).unwrap_or(Value::Null),
            instances_registry: serde_json::from_str(instances_registry).unwrap_or(Value::Null),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        serde_json::to_string_pretty(&welcome).unwrap_or_else(|_| {
            format!(
                "{{\"type\":\"welcome\",\"message\":\"{}\"}}",
                t!("app.welcome_message")
            )
        })
    }

    /// Get cached welcome message (for first response)
    pub fn get_welcome_message(&self) -> &str {
        &self.cached_welcome_message
    }

    /// Get cached skills registry
    pub fn get_skills_registry(&self) -> &str {
        &self.cached_skills_registry
    }

    /// Get cached instances registry
    pub fn get_instances_registry(&self) -> &str {
        &self.cached_instances_registry
    }

    /// Handle natural language input from user using configured workflow mode
    ///
    /// This function processes user natural language input using the workflow
    /// mode specified during initialization.
    ///
    /// # Arguments
    /// * `input` - Natural language input from the user
    /// * `session_id` - Optional session ID for conversation history
    ///                  (uses "default" if None)
    /// * `is_first_message` - Whether this is the first message (skip registry in prompt)
    ///
    /// # Returns
    /// The response string after processing
    pub async fn handle_natural_language(
        &self,
        input: &str,
        session_id: Option<&str>,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let session_id = session_id.unwrap_or("default");
        let mut executor = self.workflow_executor.clone();
        if let Some(cb) = callback {
            executor = executor.with_callback(cb);
        }
        // is first message
        let is_first = !self.is_first_message.load(Ordering::SeqCst);
        if is_first {
            self.is_first_message.store(true, Ordering::SeqCst);
        }
        // Pass cached registries to executor
        executor
            .execute(
                &self.scheduler,
                &self.memory,
                &self.skills_dir,
                input,
                session_id,
                &self.cached_skills_registry,
                &self.cached_instances_registry,
                is_first,
            )
            .await
    }
    /// Handle multiple natural language inputs in parallel
    pub async fn handle_natural_language_batch(
        &mut self,
        inputs: Vec<(String, Option<String>, Option<Arc<dyn WorkflowCallback>>)>,
    ) -> Vec<String> {
        if inputs.is_empty() {
            return Vec::new();
        }
        info!(
            "Processing {} natural language inputs in parallel with mode {:?}",
            inputs.len(),
            self.workflow_mode
        );
        let mut handles = Vec::new();
        for (input, session_id, callback) in inputs {
            let mut self_clone = self.clone();
            let handle = tokio::spawn(async move {
                self_clone
                    .handle_natural_language(&input, session_id.as_deref(), callback)
                    .await
            });
            handles.push(handle);
        }
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(format!("{}: {}", t!("error.task_panic"), e)),
            }
        }
        results
    }

    /// Handle SKILL.md file execution
    pub async fn handle_skill_md(
        &self,
        skill_name: &str,
        params: Option<HashMap<String, Value>>,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let skill_file =
            match SkillLoader::load_by_name(self.skills_dir.to_str().unwrap_or("."), skill_name) {
                Ok(Some(file)) => file,
                Ok(None) => {
                    return format!("{}: {}", t!("error.skill_not_found"), skill_name);
                }
                Err(e) => {
                    return format!("{}: {}", t!("error.load_skill_failed"), e);
                }
            };
        info!(
            "Executing SKILL.md: {} with workflow mode: {}",
            skill_name, self.workflow_mode
        );
        let mut instructions = skill_file.instructions;
        if let Some(params) = &params {
            for (key, value) in params {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                instructions = instructions.replace(&placeholder, &replacement);
            }
        }
        // Use cached registries
        let enhanced_input = format!(
            "{}\n\n## Available Atomic Skills\n{}\n\n## Available Instances\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
            instructions, self.cached_skills_registry, self.cached_instances_registry
        );
        let session_id = format!("skill_md_{}", skill_name);
        let mut executor = self.workflow_executor.clone();
        if let Some(cb) = callback {
            executor = executor.with_callback(cb);
        }
        // is first message
        let is_first = !self.is_first_message.load(Ordering::SeqCst);
        if is_first {
            self.is_first_message.store(true, Ordering::SeqCst);
        }
        executor
            .execute(
                &self.scheduler,
                &self.memory,
                &self.skills_dir,
                &enhanced_input,
                &session_id,
                &self.cached_skills_registry,
                &self.cached_instances_registry,
                is_first,
            )
            .await
    }

    /// Handle multiple SKILL.md files execution in parallel
    pub async fn handle_skill_md_batch(
        &self,
        tasks: Vec<(
            String,
            Option<HashMap<String, Value>>,
            Option<Arc<dyn WorkflowCallback>>,
        )>,
    ) -> Vec<String> {
        if tasks.is_empty() {
            return Vec::new();
        }
        info!(
            "Executing {} SKILL.md files in parallel with workflow mode: {:?}",
            tasks.len(),
            self.workflow_mode
        );
        let mut handles = Vec::new();
        for (skill_name, params, callback) in tasks {
            let mut self_clone = self.clone();
            let handle = tokio::spawn(async move {
                self_clone
                    .handle_skill_md(&skill_name, params, callback)
                    .await
            });
            handles.push(handle);
        }
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(format!("{}: {}", t!("error.task_panic"), e)),
            }
        }
        results
    }

    /// Clear conversation history for a session
    pub fn clear_conversation(&self, session_id: &str) {
        self.memory.clear_session(session_id);
    }

    /// Clear all conversation histories
    pub fn clear_all_conversations(&self) {
        self.memory.clear_all();
    }

    /// List all available atomic skills
    pub fn list_atomic_skills(&self) -> String {
        let skills = crate::executors::registry::list_skills();
        if skills.is_empty() {
            return t!("skill.no_skills_available").to_string();
        }
        let mut result = String::new();
        for name in skills {
            if let Some(skill) = crate::executors::registry::get_skill(&name) {
                let emoji = match skill.category() {
                    "file" => "📁",
                    "net" => "🌐",
                    "math" => "🔢",
                    "time" => "🕐",
                    "system" => "💻",
                    "db" => "🗄️",
                    "devops" => "🚀",
                    "document" => "📄",
                    "message" => "💬",
                    "task" => "⏰",
                    _ => "⚙️",
                };
                result.push_str(&format!(
                    "   {} - **{}**: {}\n",
                    emoji,
                    name,
                    skill.description()
                ));
            }
        }
        result
    }

    /// List all available SKILL.md files in the skills directory
    pub fn list_skill_md_files(&self) -> String {
        match SkillLoader::load_all(self.skills_dir.to_str().unwrap_or(".")) {
            Ok(skills) => {
                if skills.is_empty() {
                    return t!("skill.no_skill_md_available").to_string();
                }
                let mut result = String::new();
                for skill in skills {
                    let emoji = skill
                        .metadata
                        .as_ref()
                        .and_then(|m| m.emoji.as_ref())
                        .map(|e| e.as_str())
                        .unwrap_or("📋");
                    result.push_str(&format!(
                        "   {} - **{}**: {}\n",
                        emoji, skill.name, skill.description
                    ));
                }
                result
            }
            Err(e) => format!("{}: {}", t!("error.list_skills_failed"), e),
        }
    }

    /// Get all loaded atomic skill names
    pub fn get_atomic_skill_names(&self) -> Vec<String> {
        crate::executors::registry::list_skills()
    }

    /// Get all SKILL.md file names
    pub fn get_skill_md_names(&self) -> Vec<String> {
        match SkillLoader::load_all(self.skills_dir.to_str().unwrap_or(".")) {
            Ok(skills) => skills.into_iter().map(|s| s.name).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Check if there are any atomic skills available
    pub fn has_atomic_skills(&self) -> bool {
        !crate::executors::registry::list_skills().is_empty()
    }

    /// Get the skills directory path
    pub fn skills_directory(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// Get the executor
    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> &SkillScheduler {
        &self.scheduler
    }

    /// Get the current workflow mode
    pub fn workflow_mode(&self) -> WorkflowMode {
        self.workflow_mode
    }

    /// Update configuration
    pub fn update_config<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut HippoxConfig),
    {
        crate::config::update_config(f)
    }

    /// Get configuration
    pub fn get_config(&self) -> HippoxConfig {
        crate::config::get_config()
    }

    /// Reload registries (call after config changes)
    pub fn reload_registries(&mut self) {
        self.cached_skills_registry = Self::generate_skills_registry();
        self.cached_instances_registry = Self::generate_instances_registry();
        self.cached_welcome_message = Self::generate_welcome_message(
            &self.cached_skills_registry,
            &self.cached_instances_registry,
        );
        info!("Registries reloaded successfully");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_skill_md(dir: &tempfile::TempDir, skill_name: &str, description: &str) {
        let skill_dir = dir.path().join(skill_name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        let skill_md = skill_dir.join("SKILL.md");
        let content = format!(
            r#"---
name: {}
description: {}
version: 1.0.0
author: Test Author
---

# {} Skill

This is a test workflow for {}.

## Instructions
Process the request and return a result.
"#,
            skill_name, description, skill_name, description
        );
        std::fs::write(skill_md, content).unwrap();
    }

    #[test]
    fn test_generate_skills_registry() {
        let registry = Hippox::generate_skills_registry();
        assert!(!registry.is_empty());
        assert!(registry.contains("helloworld") || registry.contains("file_read"));
    }

    #[test]
    fn test_generate_instances_registry() {
        let registry = Hippox::generate_instances_registry();
        // Registry should be valid JSON even if empty
        assert!(serde_json::from_str::<Value>(&registry).is_ok());
    }

    #[tokio::test]
    async fn test_hippox_real_conversation() {
        use langhub::types::ModelProvider;
        use std::collections::HashMap;
        use std::io::{self, Write};
        let api_key = "";
        let provider = ModelProvider::DeepSeek;
        let skills_dir = "./skills";
        let config_json = r#"{
        "lang": "en"
    }"#;
        let hippox = Hippox::new(
            skills_dir,
            provider,
            Some(api_key.to_string()),
            None,
            ConfigInitMethod::ParamsJsonStr(config_json.to_string()),
        )
        .await;
        let r = hippox
            .unwrap()
            .handle_natural_language(
                "Tell me what skills I have and what my profile is.",
                None,
                None,
            )
            .await;
        println!("{}", r);
    }
}
