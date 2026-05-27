use crate::config::{
    init_config_from_json_file, init_config_from_params_json_str, init_config_from_toml_file,
};
use crate::executors::Executor;
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
    workflow_mode: WorkflowMode,
    workflow_executor: WorkflowExecutor,
    is_first_message: Arc<AtomicBool>,
}

impl Hippox {
    /// Create a new Hippox core instance with default ReAct workflow mode
    pub async fn new(
        provider: ModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config_method: ConfigInitMethod,
    ) -> anyhow::Result<Self> {
        Self::with_workflow_mode(
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
        provider: ModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config_method: ConfigInitMethod,
        workflow_mode: WorkflowMode,
    ) -> anyhow::Result<Self> {
        info!(
            "Initializing Hippox core with workflow mode: {}",
            workflow_mode
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
        // init llm scheduler
        let scheduler = SkillScheduler::new(llm);
        let executor = Executor::new();
        let workflow_executor = WorkflowExecutor::new(workflow_mode);
        Ok(Self {
            scheduler,
            executor,
            workflow_mode,
            workflow_executor,
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

    /// Notify LLM about updated skills registry
    ///
    /// Call this after dynamically registering new skills.
    /// This will mark the session to resend the skills registry on next message.
    pub fn refresh_llm_skill_registry(&self) {
        self.is_first_message.store(false, Ordering::SeqCst);
    }

    /// Notify LLM about updated instances registry
    ///
    /// Call this after adding/removing instance configurations.
    /// This will mark the session to resend the instances registry on next message.
    pub fn refresh_llm_instances(&self) {
        self.is_first_message.store(false, Ordering::SeqCst);
    }

    /// Get current skills registry as JSON string
    pub fn get_skills_registry(&self) -> String {
        Self::generate_skills_registry()
    }

    /// Get current instances registry as JSON string
    pub fn get_instances_registry(&self) -> String {
        Self::generate_instances_registry()
    }

    /// Get welcome message with current registries
    pub fn get_welcome_message(&self) -> String {
        let skills = self.get_skills_registry();
        let instances = self.get_instances_registry();
        Self::generate_welcome_message(&skills, &instances)
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
        // Get current registries
        let skills_registry = self.get_skills_registry();
        let instances_registry = self.get_instances_registry();
        // Pass cached registries to executor
        executor
            .execute(
                &self.scheduler,
                input,
                session_id,
                &skills_registry,
                &instances_registry,
                is_first,
            )
            .await
    }

    /// Handle multiple natural language inputs in parallel
    pub async fn handle_natural_language_batch(
        &self,
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
            let self_clone = self.clone();
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

    /// Execute a SKILL.md workflow file
    ///
    /// # Arguments
    /// * `skill_md_path` - Path to the SKILL.md file
    ///   Format: The path should point directly to the SKILL.md file.
    ///   Example: "./skills/web-search/SKILL.md" or "/path/to/skills/my-skill/SKILL.md"
    /// * `params` - Optional parameters to substitute in the SKILL.md content
    ///   Placeholders in SKILL.md should use `{{parameter_name}}` format
    /// * `callback` - Optional callback for workflow execution progress
    ///
    /// # Returns
    /// The execution result as a string
    ///
    /// # SKILL.md File Format
    /// ```markdown
    /// ---
    /// name: my-skill
    /// description: What this skill does
    /// version: 1.0.0
    /// ---
    ///
    /// # Instructions
    /// Your workflow instructions here. Use {{param_name}} for variable substitution.
    /// ```
    pub async fn handle_skill_md(
        &self,
        skill_md_path: &str,
        params: Option<HashMap<String, Value>>,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let skill_file = match SkillLoader::load_from_path(skill_md_path) {
            Ok(Some(file)) => file,
            Ok(None) => {
                return format!("{}: {}", t!("error.skill_not_found"), skill_md_path);
            }
            Err(e) => {
                return format!("{}: {}", t!("error.load_skill_failed"), e);
            }
        };
        info!(
            "Executing SKILL.md: {} with workflow mode: {}",
            skill_file.name, self.workflow_mode
        );
        let skills_registry = self.get_skills_registry();
        let instances_registry = self.get_instances_registry();

        let mut executor = self.workflow_executor.clone();
        if let Some(cb) = callback {
            executor = executor.with_callback(cb);
        }
        let is_first = !self.is_first_message.load(Ordering::SeqCst);
        if is_first {
            self.is_first_message.store(true, Ordering::SeqCst);
        }
        executor
            .execute_skill_md(
                &self.scheduler,
                &skill_file,
                params.as_ref(),
                &skills_registry,
                &instances_registry,
                is_first,
            )
            .await
    }

    /// Execute multiple SKILL.md files in parallel
    ///
    /// # Arguments
    /// * `tasks` - Vector of tuples (skill_md_path, params, callback)
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
        for (skill_md_path, params, callback) in tasks {
            let self_clone = self.clone();
            let handle = tokio::spawn(async move {
                self_clone
                    .handle_skill_md(&skill_md_path, params, callback)
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

    /// List all available SKILL.md files in a directory
    ///
    /// # Arguments
    /// * `skills_dir` - Directory containing skill subdirectories with SKILL.md files
    pub fn list_skill_md_files(&self, skills_dir: &str) -> String {
        match SkillLoader::load_all(skills_dir) {
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

    /// Get all SKILL.md file names from a directory
    ///
    /// # Arguments
    /// * `skills_dir` - Directory containing skill subdirectories with SKILL.md files
    pub fn get_skill_md_names(&self, skills_dir: &str) -> Vec<String> {
        match SkillLoader::load_all(skills_dir) {
            Ok(skills) => skills.into_iter().map(|s| s.name).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Check if there are any atomic skills available
    pub fn has_atomic_skills(&self) -> bool {
        !crate::executors::registry::list_skills().is_empty()
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let config_json = r#"{
            "lang": "en"
        }"#;
        let hippox = Hippox::new(
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
