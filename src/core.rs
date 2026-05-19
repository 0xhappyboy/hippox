use crate::config::{
    init_config_from_env, init_config_from_json_file, init_config_from_params_json_str,
    init_config_from_toml_file,
};
use crate::executors::Executor;
use crate::memory::ConversationMemory;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::workflow::{WorkflowExecutor, WorkflowMode};
use crate::{HippoxConfig, i18n};
use crate::{get_config, t};
use langhub::LLMClient;
use langhub::types::ModelProvider;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
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
    Env,
    TomlFile(String),
    JsonFile(String),
    ParamsJsonStr(String),
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
            ConfigInitMethod::Env => init_config_from_env(),
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
        println!("{}", STARTUP_BANNER);
        Ok(Self {
            scheduler,
            executor,
            memory: ConversationMemory::new(),
            skills_dir: PathBuf::from(skills_dir),
            workflow_mode,
            workflow_executor,
        })
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
    pub async fn handle_natural_language(&self, input: &str, session_id: Option<&str>) -> String {
        let session_id = session_id.unwrap_or("default");
        self.workflow_executor
            .execute(
                &self.scheduler,
                &self.memory,
                &self.skills_dir,
                input,
                session_id,
            )
            .await
    }

    /// Handle multiple natural language inputs in parallel
    ///
    /// This function processes multiple natural language inputs concurrently.
    /// Each input uses its own session ID or shares the same session.
    ///
    /// # Arguments
    /// * `inputs` - A vector of tuples: `Vec<(String, Option<String>)>`
    ///     - First element: The natural language input text
    ///     - Second element: Optional session ID for conversation history
    ///       (uses "default" if None)
    ///
    /// # Returns
    /// A vector of response strings in the **same order** as the input tasks.
    pub async fn handle_natural_language_batch(
        &self,
        inputs: Vec<(String, Option<String>)>,
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
        for (input, session_id) in inputs {
            let self_clone = self.clone();
            let handle = tokio::spawn(async move {
                self_clone
                    .handle_natural_language(&input, session_id.as_deref())
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
    ///
    /// This function loads and executes a SKILL.md file as a predefined workflow.
    /// It uses the workflow executor to actually call atomic skills, following
    /// the configured workflow mode (ReAct, Batch, Chain, PlanAndExecute).
    ///
    /// # Arguments
    /// * `skill_name` - Name of the skill (subdirectory name containing SKILL.md)
    /// * `params` - Optional parameters to pass to the skill execution
    ///
    /// # Returns
    /// The execution result as a string
    pub async fn handle_skill_md(
        &self,
        skill_name: &str,
        params: Option<HashMap<String, Value>>,
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
        let registry_json = self.get_atomic_skills_registry();
        let enhanced_input = format!(
            "{}\n\n## Available Atomic Skills\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
            instructions, registry_json
        );
        let session_id = format!("skill_md_{}", skill_name);
        self.workflow_executor
            .execute(
                &self.scheduler,
                &self.memory,
                &self.skills_dir,
                &enhanced_input,
                &session_id,
            )
            .await
    }

    /// Handle multiple SKILL.md files execution in parallel
    ///
    /// This function executes multiple SKILL.md workflows concurrently.
    /// Each skill execution uses its own session ID and follows the configured workflow mode.
    ///
    /// # Arguments
    /// * `tasks` - A vector of tuples: `Vec<(String, Option<HashMap<String, Value>>)>`
    ///     - First element: The skill name
    ///     - Second element: Optional parameters for the skill
    ///
    /// # Returns
    /// A vector of execution results in the same order as the input tasks
    pub async fn handle_skill_md_batch(
        &self,
        tasks: Vec<(String, Option<HashMap<String, Value>>)>,
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
        for (skill_name, params) in tasks {
            let self_clone = self.clone();
            let handle =
                tokio::spawn(async move { self_clone.handle_skill_md(&skill_name, params).await });
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

    /// Get the atomic skills registry as JSON string
    fn get_atomic_skills_registry(&self) -> String {
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
                    })
                })
            })
            .collect();
        serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
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

    #[tokio::test]
    async fn test_hippox_new_with_default_mode() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
        )
        .await;
        assert!(hippox.is_ok());
        let hippox = hippox.unwrap();
        assert_eq!(hippox.workflow_mode(), WorkflowMode::ReAct);
    }

    #[tokio::test]
    async fn test_hippox_new_with_batch_mode() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::with_workflow_mode(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
            WorkflowMode::Batch,
        )
        .await;
        assert!(hippox.is_ok());
        let hippox = hippox.unwrap();
        assert_eq!(hippox.workflow_mode(), WorkflowMode::Batch);
    }

    #[tokio::test]
    async fn test_hippox_new_with_chain_mode() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::with_workflow_mode(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
            WorkflowMode::Chain,
        )
        .await;
        assert!(hippox.is_ok());
        let hippox = hippox.unwrap();
        assert_eq!(hippox.workflow_mode(), WorkflowMode::Chain);
    }

    #[tokio::test]
    async fn test_hippox_new_with_plan_and_execute_mode() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::with_workflow_mode(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
            WorkflowMode::PlanAndExecute,
        )
        .await;
        assert!(hippox.is_ok());
        let hippox = hippox.unwrap();
        assert_eq!(hippox.workflow_mode(), WorkflowMode::PlanAndExecute);
    }

    #[tokio::test]
    async fn test_list_atomic_skills() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
        )
        .await
        .unwrap();
        let skills = hippox.list_atomic_skills();
        assert!(skills.contains("calculator") || skills.contains("helloworld"));
    }

    #[tokio::test]
    async fn test_clear_conversation() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
        )
        .await
        .unwrap();
        hippox.clear_conversation("test-session");
        hippox.clear_all_conversations();
    }
}
