//! Main Hippox core implementation

use crate::core::tasks::NaturalLanguageTask;
use crate::executors::Executor;
use crate::prompts::{
    build_skill_md_prompt, generate_instances_registry, generate_skills_registry,
};
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::tasks::{self, ExecutableTask, TaskStatus};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor, WorkflowMode};
use crate::{
    HippoxConfig, IdentityInformation, Pipeline, StageOneResult, SystemPipeline, get_config, i18n,
    needs_format_conversion, t, update_config,
};
use langhub::LLMClient;
use langhub::types::ModelProvider;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

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
        config: Option<HippoxConfig>,
    ) -> anyhow::Result<Self> {
        Self::with_workflow_mode(
            provider,
            api_key,
            extra_keys,
            config,
            WorkflowMode::default(),
        )
        .await
    }

    /// Create a new Hippox core instance with specified workflow mode
    pub async fn with_workflow_mode(
        provider: ModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config: Option<HippoxConfig>,
        workflow_mode: WorkflowMode,
    ) -> anyhow::Result<Self> {
        info!(
            "Initializing Hippox core with workflow mode: {}",
            workflow_mode
        );
        // init config
        update_config(|global| *global = config.unwrap_or_default())?;
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
        generate_skills_registry()
    }

    /// Get current instances registry as JSON string
    pub fn get_instances_registry(&self) -> String {
        generate_instances_registry()
    }

    /// Get identity information
    pub fn get_identity(&self) -> IdentityInformation {
        self.get_config().identity_information
    }

    /// Update identity information with a closure
    pub fn update_identity<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut IdentityInformation),
    {
        self.update_config(|config| {
            f(&mut config.identity_information);
        })
    }

    /// Set identity information directly
    pub fn set_identity(&self, identity: IdentityInformation) -> anyhow::Result<()> {
        self.update_config(|config| {
            config.identity_information = identity;
        })
    }

    /// Submit a natural language task and return task ID immediately
    ///
    /// This function creates a task, adds it to the global task pool, and returns the task ID.
    /// The actual execution happens asynchronously in the background.
    ///
    /// # Arguments
    /// * `input` - Natural language input from the user
    /// * `_session_id` - Optional session ID (unused in core, for compatibility)
    /// * `_callback` - Optional callback for workflow execution progress
    ///
    /// # Returns
    /// The task ID as a string
    pub fn submit(
        &self,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let executable = Arc::new(NaturalLanguageTask::new(
            input.to_string(),
            self.workflow_executor.clone(),
            self.scheduler.clone(),
        ));
        let task_id = futures::executor::block_on(tasks::create_task_with_executable(
            "natural_language".to_string(),
            input.to_string(),
            executable,
            callback,
        ));
        info!(
            "Created natural language task: {} with input: {}",
            task_id, input
        );
        task_id
    }

    /// Submit multiple natural language tasks in batch and return task IDs immediately
    ///
    /// # Arguments
    /// * `inputs` - Vector of tuples (input, session_id, callback)
    ///
    /// # Returns
    /// Vector of task IDs in the same order as inputs
    pub fn submit_batch(
        &self,
        inputs: Vec<(String, Option<String>, Option<Arc<dyn WorkflowCallback>>)>,
    ) -> Vec<String> {
        inputs
            .into_iter()
            .map(|(input, _session_id, callback)| self.submit(&input, callback))
            .collect()
    }

    /// Execute natural language directly without task pool, returning the result asynchronously.
    /// Execute natural language directly without task pool
    pub async fn execute(
        &self,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let pipeline = SystemPipeline::new();
        // Stage Zero: Classify intent
        let categories = match pipeline.stage_zero(&self.scheduler, input).await {
            Ok(result) => result.categories,
            Err(e) => {
                tracing::warn!("Stage Zero classification failed: {}, using all skills", e);
                vec![]
            }
        };
        // Stage One: Core workflow execution
        let stage_one_result = if categories.is_empty() {
            pipeline
                .stage_one(
                    self.workflow_mode,
                    &self.workflow_executor,
                    &self.scheduler,
                    input,
                    callback,
                )
                .await
        } else {
            let result = self
                .workflow_executor
                .execute_with_categories(&self.scheduler, input, &categories)
                .await;
            let json_output = match result {
                WorkflowExecutionResult::Completed(output) => output,
                WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => raw_json,
                WorkflowExecutionResult::Paused { partial_output, .. } => partial_output,
                WorkflowExecutionResult::Cancelled { .. } => String::new(),
                WorkflowExecutionResult::Failed { error, .. } => error,
            };
            StageOneResult {
                json_output,
                original_input: input.to_string(),
            }
        };
        if stage_one_result.json_output.is_empty() {
            return stage_one_result.json_output;
        }
        // Stage Two: Format conversion
        if needs_format_conversion(input) {
            let stage_two_result = pipeline
                .stage_two(
                    &self.scheduler,
                    &stage_one_result.original_input,
                    &stage_one_result.json_output,
                )
                .await;
            stage_two_result.final_output
        } else {
            stage_one_result.json_output
        }
    }

    /// Execute multiple natural language tasks directly without task pool.
    pub async fn execute_batch(
        &self,
        inputs: Vec<(String, Option<Arc<dyn WorkflowCallback>>)>,
    ) -> Vec<String> {
        let mut results = Vec::new();
        for (input, callback) in inputs {
            let result = self.execute(&input, callback).await;
            results.push(result);
        }
        results
    }

    /// Get task status by ID
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        futures::executor::block_on(tasks::get_task_status(task_id))
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<tasks::Task> {
        futures::executor::block_on(tasks::get_task(task_id))
    }

    /// Cancel a running or pending task
    pub fn cancel_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(tasks::cancel_task(task_id))
    }

    /// Pause a running task
    pub fn pause_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(tasks::pause_task(task_id))
    }

    /// Resume a paused task
    pub fn resume_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(tasks::resume_task(task_id))
    }

    /// Retry a failed task
    pub fn retry_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(tasks::retry_task(task_id))
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
