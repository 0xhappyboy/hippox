//! Main Hippox core implementation

use crate::core::tasks::NaturalLanguageTask;
use crate::executors::Executor;
use crate::prompts::{
    build_skill_md_prompt, generate_instances_registry, generate_skills_registry,
};
use crate::skill_scheduler::SkillScheduler;
use crate::tasks::{self, ExecutableTask, TaskStatus};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor, WorkflowMode};
use crate::{
    HippoxBatchResult, HippoxBoolResult, HippoxConfig, HippoxResult, HippoxStringResult,
    HippoxVoidResult, IdentityInformation, IntentAnalysisResult, Pipeline, SystemPipeline,
    WorkflowExecResult, get_config, i18n, needs_format_conversion, t, update_config,
};
use langhub::LLMClient;
use langhub::types::{ChatMessage, ModelProvider};
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
    pub fn refresh_llm_skill_registry(&self) -> HippoxVoidResult {
        self.is_first_message.store(false, Ordering::SeqCst);
        HippoxResult::ok(())
    }

    /// Notify LLM about updated instances registry
    ///
    /// Call this after adding/removing instance configurations.
    /// This will mark the session to resend the instances registry on next message.
    pub fn refresh_llm_instances(&self) -> HippoxVoidResult {
        self.is_first_message.store(false, Ordering::SeqCst);
        HippoxResult::ok(())
    }

    /// Get current skills registry as JSON string
    pub fn get_skills_registry(&self) -> HippoxStringResult {
        HippoxResult::ok(generate_skills_registry())
    }

    /// Get current instances registry as JSON string
    pub fn get_instances_registry(&self) -> HippoxStringResult {
        HippoxResult::ok(generate_instances_registry())
    }

    /// Get identity information
    pub fn get_identity(&self) -> HippoxResult<IdentityInformation> {
        HippoxResult::ok(self.get_config().identity_information)
    }

    /// Update identity information with a closure
    pub fn update_identity<F>(&self, f: F) -> HippoxVoidResult
    where
        F: FnOnce(&mut IdentityInformation),
    {
        match self.update_config(|config| {
            f(&mut config.identity_information);
        }) {
            Ok(_) => HippoxResult::ok(()),
            Err(e) => HippoxResult::system_error(e.to_string()),
        }
    }

    /// Set identity information directly
    pub fn set_identity(&self, identity: IdentityInformation) -> HippoxVoidResult {
        match self.update_config(|config| {
            config.identity_information = identity;
        }) {
            Ok(_) => HippoxResult::ok(()),
            Err(e) => HippoxResult::system_error(e.to_string()),
        }
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
    /// The task ID as a string wrapped in HippoxResult
    pub fn submit(
        &self,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> HippoxStringResult {
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
        HippoxResult::ok(task_id)
    }

    /// Submit multiple natural language tasks in batch and return task IDs immediately
    ///
    /// # Arguments
    /// * `inputs` - Vector of tuples (input, session_id, callback)
    ///
    /// # Returns
    /// Vector of task IDs in the same order as inputs wrapped in HippoxResult
    pub fn submit_batch(
        &self,
        inputs: Vec<(String, Option<String>, Option<Arc<dyn WorkflowCallback>>)>,
    ) -> HippoxBatchResult {
        let task_ids: Vec<String> = inputs
            .into_iter()
            .map(|(input, _session_id, callback)| {
                self.submit(&input, callback).unwrap_or(String::new())
            })
            .collect();
        HippoxResult::ok(task_ids)
    }

    pub async fn execute_batch(
        &self,
        inputs: Vec<(String, Option<Arc<dyn WorkflowCallback>>)>,
    ) -> HippoxBatchResult {
        let mut results = Vec::new();
        for (input, callback) in inputs {
            results.push(
                self.execute(&input, callback)
                    .await
                    .unwrap_or(String::new()),
            );
        }
        HippoxResult::ok(results)
    }

    /// Execute natural language directly without task pool, returning the result asynchronously.
    pub async fn execute(
        &self,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> HippoxStringResult {
        let pipeline = SystemPipeline::new();
        // Step 1: intent analysis
        let intent_result = match pipeline.intent_analysis(&self.scheduler, input).await {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Intent analysis failed: {}, using raw input", e);
                IntentAnalysisResult {
                    categories: vec![],
                    clean_intent: input.to_string(),
                }
            }
        };
        let clean_intent = &intent_result.clean_intent;
        let categories = &intent_result.categories;
        // Step 2: Workflow execution
        let workflow_result = if categories.is_empty() {
            pipeline
                .workflow_execution(
                    self.workflow_mode,
                    &self.workflow_executor,
                    &self.scheduler,
                    clean_intent,
                    callback,
                )
                .await
        } else {
            let result = self
                .workflow_executor
                .execute_with_categories(&self.scheduler, clean_intent, categories)
                .await;
            let json_output = match result {
                WorkflowExecutionResult::Completed(output) => output,
                WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => raw_json,
                _ => String::new(),
            };
            WorkflowExecResult {
                json_output,
                original_input: clean_intent.to_string(),
            }
        };
        let final_output = if needs_format_conversion(input) {
            let format_result = pipeline
                .response_formatting(&self.scheduler, input, &workflow_result.json_output)
                .await;
            format_result.final_output
        } else {
            workflow_result.json_output
        };
        HippoxResult::ok(final_output)
    }

    /// heartbeat
    pub async fn heartbeat(&self) -> HippoxStringResult {
        let mut messages: Vec<ChatMessage> = Vec::new();
        messages.push(ChatMessage::user("hi"));
        match self.scheduler.get_llm().chat(messages).await {
            Ok(response) => HippoxResult::ok(response),
            Err(e) => HippoxResult::network_error(e.to_string()),
        }
    }

    /// Get task status by ID
    pub fn get_task_status(&self, task_id: &str) -> HippoxResult<TaskStatus> {
        match futures::executor::block_on(tasks::get_task_status(task_id)) {
            Some(status) => HippoxResult::ok(status),
            None => HippoxResult::system_error(format!("Task not found: {}", task_id)),
        }
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> HippoxResult<tasks::Task> {
        match futures::executor::block_on(tasks::get_task(task_id)) {
            Some(task) => HippoxResult::ok(task),
            None => HippoxResult::system_error(format!("Task not found: {}", task_id)),
        }
    }

    /// Cancel a running or pending task
    pub fn cancel_task(&self, task_id: &str) -> HippoxBoolResult {
        match futures::executor::block_on(tasks::cancel_task(task_id)) {
            true => HippoxResult::ok(true),
            false => HippoxResult::system_error(format!("Failed to cancel task: {}", task_id)),
        }
    }

    /// Pause a running task
    pub fn pause_task(&self, task_id: &str) -> HippoxBoolResult {
        match futures::executor::block_on(tasks::pause_task(task_id)) {
            true => HippoxResult::ok(true),
            false => HippoxResult::system_error(format!("Failed to pause task: {}", task_id)),
        }
    }

    /// Resume a paused task
    pub fn resume_task(&self, task_id: &str) -> HippoxBoolResult {
        match futures::executor::block_on(tasks::resume_task(task_id)) {
            true => HippoxResult::ok(true),
            false => HippoxResult::system_error(format!("Failed to resume task: {}", task_id)),
        }
    }

    /// Retry a failed task
    pub fn retry_task(&self, task_id: &str) -> HippoxBoolResult {
        match futures::executor::block_on(tasks::retry_task(task_id)) {
            true => HippoxResult::ok(true),
            false => HippoxResult::system_error(format!("Failed to retry task: {}", task_id)),
        }
    }

    /// List all available atomic skills
    pub fn list_atomic_skills(&self) -> HippoxStringResult {
        let skills = crate::executors::registry::list_skills();
        if skills.is_empty() {
            return HippoxResult::ok(t!("skill.no_skills_available").to_string());
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
        HippoxResult::ok(result)
    }

    /// Get all loaded atomic skill names
    pub fn get_atomic_skill_names(&self) -> HippoxBatchResult {
        HippoxResult::ok(crate::executors::registry::list_skills())
    }

    /// Check if there are any atomic skills available
    pub fn has_atomic_skills(&self) -> HippoxBoolResult {
        HippoxResult::ok(!crate::executors::registry::list_skills().is_empty())
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
