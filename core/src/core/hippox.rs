//! Main Hippox core implementation

use crate::core::tasks::NaturalLanguageTask;
use crate::driver_scheduler::DriverScheduler;
use crate::prompts::{build_driver_md_prompt, generate_drivers_registry};
use crate::tasks::{self, ExecutableTask, TaskStatus};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor, WorkflowMode};
use crate::{
    HippoxBatchResult, HippoxBoolResult, HippoxConfig, HippoxResult, HippoxStringResult,
    HippoxVoidResult, IdentityInformation, IntentAnalysisResult, Pipeline, SystemPipeline,
    WorkflowExecResult, get_config, i18n, needs_format_conversion, t, update_config,
};
use hippox_drivers::{
    DriverCallback, DriverCategory, Executor, get_all_drivers, list_drivers_names,
};
use langhub::LLMClient;
use langhub::types::{ChatMessage, ModelProvider};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::info;

/// Global input token count for the entire process
pub static INPUT_TOKEN_COUNT: AtomicU64 = AtomicU64::new(0);

/// Global output token count for the entire process
pub static OUTPUT_TOKEN_COUNT: AtomicU64 = AtomicU64::new(0);

/// Core engine for Hippox
///
/// This is the main entry point for the Hippox engine. It handles:
/// - Natural language processing with atomic driver registry
/// - SKILL.md file execution for complex workflows
/// - Managing conversation history for natural language interactions
#[derive(Clone)]
pub struct Hippox {
    scheduler: DriverScheduler,
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
        let scheduler = DriverScheduler::new(llm);
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

    /// Notify LLM about updated drivers registry
    ///
    /// Call this after dynamically registering new drivers.
    /// This will mark the session to resend the drivers registry on next message.
    pub fn refresh_llm_driver_registry(&self) -> HippoxVoidResult {
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

    /// Get current drivers registry as JSON string
    pub fn get_drivers_registry(&self) -> HippoxStringResult {
        HippoxResult::ok(generate_drivers_registry())
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
        workflow_callback: Option<Arc<dyn WorkflowCallback>>,
        driver_callback: Option<Arc<dyn DriverCallback>>,
    ) -> HippoxStringResult {
        let executable = Arc::new(NaturalLanguageTask::new(
            input.to_string(),
            self.workflow_executor.clone(),
            self.scheduler.clone(),
            workflow_callback,
            driver_callback,
        ));
        let result = futures::executor::block_on(tasks::create_task_with_executable(
            "natural_language".to_string(),
            input.to_string(),
            executable,
        ));
        if result.is_ok() {
            let task_id = result.unwrap();
            info!(
                "Created natural language task: {} with input: {}",
                task_id, input
            );
            HippoxResult::ok(task_id)
        } else {
            let error = result.error.unwrap_or_else(|| "Unknown error".to_string());
            tracing::error!("Failed to create task: {}", error);
            HippoxResult::system_error(format!("Failed to create task: {}", error))
        }
    }

    /// Submit multiple natural language tasks in batch and return task IDs immediately
    ///
    /// # Arguments
    /// * `inputs` - Vector of tuples (input, session_id, workflow_callback, driver_callback)
    ///
    /// # Returns
    /// Vector of task IDs in the same order as inputs wrapped in HippoxResult
    pub fn submit_batch(
        &self,
        inputs: Vec<(
            String,
            Option<String>,
            Option<Arc<dyn WorkflowCallback>>,
            Option<Arc<dyn DriverCallback>>,
        )>,
    ) -> HippoxBatchResult {
        let task_ids: Vec<String> = inputs
            .into_iter()
            .map(|(input, _session_id, workflow_callback, driver_callback)| {
                self.submit(&input, workflow_callback, driver_callback)
                    .unwrap_or(String::new())
            })
            .collect();
        HippoxResult::ok(task_ids)
    }

    /// Execute multiple natural language tasks in batch and return results directly
    ///
    /// # Arguments
    /// * `inputs` - Vector of tuples (input, workflow_callback, driver_callback)
    ///
    /// # Returns
    /// Vector of results in the same order as inputs wrapped in HippoxBatchResult
    pub async fn execute_batch(
        &self,
        inputs: Vec<(
            String,
            Option<Arc<dyn WorkflowCallback>>,
            Option<Arc<dyn DriverCallback>>,
        )>,
    ) -> HippoxBatchResult {
        let mut results = Vec::new();
        for (input, workflow_callback, driver_callback) in inputs {
            results.push(
                self.execute(&input, workflow_callback, driver_callback)
                    .await
                    .unwrap_or(String::new()),
            );
        }
        HippoxResult::ok(results)
    }

    /// Execute natural language directly without task pool, returning the result asynchronously.
    ///
    /// Note: This function uses the task pool **only** for token counting via `TaskStateUpdater`.
    /// The actual execution logic runs synchronously in the current thread, not through
    /// the background execution engine.
    ///
    /// # Example
    /// ```
    /// # async fn example() -> anyhow::Result<()> {
    /// let result = hippox.execute("What is the weather today?", None).await?;
    /// println!("{}", result);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Compare with [`submit()`](Self::submit):
    /// - `execute()`: Blocks until completion, returns result directly
    /// - `submit()`: Returns task ID immediately, use [`wait_task()`](Self::wait_task) to get result
    pub async fn execute(
        &self,
        input: &str,
        workflow_callback: Option<Arc<dyn WorkflowCallback>>,
        driver_callback: Option<Arc<dyn DriverCallback>>,
    ) -> HippoxStringResult {
        let temp_task_id = uuid::Uuid::new_v4().to_string();
        {
            let mut pool = tasks::TASK_POOL.write().await;
            let task = tasks::Task::new("temp".to_string(), input.to_string());
            pool.tasks.insert(temp_task_id.clone(), task);
        }
        let pipeline = SystemPipeline::new();
        // Step 1: intent analysis
        let intent_result = match pipeline
            .intent_analysis(&self.scheduler, input, &temp_task_id)
            .await
        {
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
        let workflow_executor_with_id = self
            .workflow_executor
            .clone()
            .with_task_id(temp_task_id.clone());
        // workflow callback
        let workflow_executor_with_callbacks = if let Some(cb) = workflow_callback {
            workflow_executor_with_id.with_workflow_callback(cb)
        } else {
            workflow_executor_with_id
        };
        // driver callback
        let workflow_executor_with_driver_cb = if let Some(cb) = driver_callback {
            workflow_executor_with_callbacks.with_driver_callback(cb)
        } else {
            workflow_executor_with_callbacks
        };
        let workflow_result = if categories.is_empty() {
            pipeline
                .workflow_execution(
                    self.workflow_mode,
                    &workflow_executor_with_driver_cb,
                    &self.scheduler,
                    clean_intent,
                )
                .await
        } else {
            let result = workflow_executor_with_driver_cb
                .clone()
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
        // Step 3: format conversion
        let final_output = if needs_format_conversion(input) {
            let format_result = pipeline
                .response_formatting(
                    &self.scheduler,
                    input,
                    &workflow_result.json_output,
                    &temp_task_id,
                )
                .await;
            format_result.final_output
        } else {
            workflow_result.json_output
        };
        let (input_tokens, output_tokens) = tasks::get_task(&temp_task_id)
            .await
            .map(|task| (task.input_token_count, task.output_token_count))
            .unwrap_or((0, 0));
        {
            let mut pool = tasks::TASK_POOL.write().await;
            // Remove temporary tasks from the task pool.
            pool.tasks.remove(&temp_task_id);
        }
        INPUT_TOKEN_COUNT.fetch_add(input_tokens, std::sync::atomic::Ordering::Relaxed);
        OUTPUT_TOKEN_COUNT.fetch_add(output_tokens, std::sync::atomic::Ordering::Relaxed);
        HippoxResult::ok_with_tokens(final_output, input_tokens, output_tokens)
    }

    /// heartbeat
    pub async fn heartbeat(&self) -> HippoxStringResult {
        let mut messages: Vec<ChatMessage> = Vec::new();
        messages.push(ChatMessage::user("hi"));
        match self.scheduler.chat_raw(messages).await {
            Ok(result) => {
                let usage = result.extract_usage();
                let input_tokens = usage.as_ref().map(|u| u.prompt_tokens as u64).unwrap_or(0);
                let output_tokens = usage
                    .as_ref()
                    .map(|u| u.completion_tokens as u64)
                    .unwrap_or(0);
                INPUT_TOKEN_COUNT.fetch_add(input_tokens, std::sync::atomic::Ordering::Relaxed);
                OUTPUT_TOKEN_COUNT.fetch_add(output_tokens, std::sync::atomic::Ordering::Relaxed);
                HippoxResult::ok_with_tokens(result.text, input_tokens, output_tokens)
            }
            Err(e) => HippoxResult::network_error(e.to_string()),
        }
    }

    /// List all available atomic drivers
    pub fn list_atomic_drivers(&self) -> HippoxStringResult {
        let drivers = get_all_drivers();
        if drivers.is_empty() {
            return HippoxResult::ok(t!("driver.no_drivers_available").to_string());
        }
        let mut result = String::new();
        for driver in drivers {
            let emoji = driver.category().icon();
            result.push_str(&format!(
                "   {} - **{}**: {}\n",
                emoji,
                driver.name(),
                driver.description()
            ));
        }
        HippoxResult::ok(result)
    }

    /// Get all loaded atomic driver names
    pub fn get_driver_names(&self) -> HippoxBatchResult {
        HippoxResult::ok(list_drivers_names())
    }

    /// Check if there are any atomic drivers available
    pub fn has_atomic_drivers(&self) -> HippoxBoolResult {
        HippoxResult::ok(!list_drivers_names().is_empty())
    }

    /// Get the executor
    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> &DriverScheduler {
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

    /// Get current global input token count
    ///
    /// This returns the total input tokens consumed across all tasks
    /// in the entire process lifetime.
    ///
    /// # Returns
    /// The total input token count as u64
    ///
    /// # Example
    /// ```
    /// let hippox = Hippox::builder(ModelProvider::OpenAI).build().await?;
    /// let input_tokens = hippox.get_current_input_token_count();
    /// println!("Total input tokens: {}", input_tokens);
    /// ```
    pub fn get_current_input_token_count(&self) -> u64 {
        INPUT_TOKEN_COUNT.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get current global output token count
    ///
    /// This returns the total output tokens consumed across all tasks
    /// in the entire process lifetime.
    ///
    /// # Returns
    /// The total output token count as u64
    ///
    /// # Example
    /// ```
    /// let hippox = Hippox::builder(ModelProvider::OpenAI).build().await?;
    /// let output_tokens = hippox.get_current_output_token_count();
    /// println!("Total output tokens: {}", output_tokens);
    /// ```
    pub fn get_current_output_token_count(&self) -> u64 {
        OUTPUT_TOKEN_COUNT.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Storage task pool to a JSON file and remove completed tasks from memory
    ///
    /// This function saves all completed/failed/cancelled/timeout tasks from the task pool
    /// to a JSON file at the specified path, then removes them from memory to free up resources.
    /// Only terminal state tasks (cannot be executed again) are processed.
    ///
    /// # Arguments
    /// * `path` - The file path to save the JSON file (e.g., "./task_pool.json")
    ///
    /// # Returns
    /// `HippoxVoidResult` - Ok(()) on success, or error on failure
    ///
    /// # Example
    /// ```ignore
    /// let hippox = Hippox::builder(ModelProvider::OpenAI).build().await?;
    /// hippox.storage_task_pool("./tasks_backup.json".to_string());
    /// ```
    pub fn storage_task_pool(&self, path: String) -> HippoxVoidResult {
        use futures::executor::block_on;
        // Get all terminal state tasks and remove them from pool atomically
        let (exported_tasks, removed_count) = block_on(async {
            let mut pool = tasks::TASK_POOL.write().await;
            // Collect terminal state tasks
            let terminal_tasks: Vec<tasks::Task> = pool
                .tasks
                .values()
                .filter(|task| {
                    matches!(
                        task.status,
                        TaskStatus::Completed
                            | TaskStatus::Failed
                            | TaskStatus::Cancelled
                            | TaskStatus::Timeout
                    )
                })
                .cloned()
                .collect();
            let removed_count = terminal_tasks.len();
            // Remove them from the pool
            for task in &terminal_tasks {
                pool.tasks.remove(&task.id);
                // Also clean up from pending_queue and running_tasks just in case
                pool.pending_queue.retain(|id| id != &task.id);
                pool.running_tasks.retain(|id| id != &task.id);
            }
            (terminal_tasks, removed_count)
        });
        if exported_tasks.is_empty() {
            info!("No terminal state tasks to backup and remove");
            return HippoxResult::ok(());
        }
        let json_data = json!({
            "export_time": chrono::Local::now().to_rfc3339(),
            "total_count": exported_tasks.len(),
            "tasks": exported_tasks.iter().map(|task| {
                json!({
                    "id": task.id,
                    "task_type": task.task_type,
                    "input": task.input,
                    "status": format!("{:?}", task.status),
                    "final_output": task.final_output,
                    "error": task.error,
                    "created_at": task.created_at,
                    "started_at": task.started_at,
                    "completed_at": task.completed_at,
                    "duration_ms": task.duration_ms,
                    "input_token_count": task.input_token_count,
                    "output_token_count": task.output_token_count,
                    "steps": task.steps.iter().map(|step| {
                        json!({
                            "driver_name": step.driver_name,
                            "status": format!("{:?}", step.status),
                            "output": step.output,
                            "error": step.error,
                            "duration_ms": step.duration_ms,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        let json_string = match serde_json::to_string_pretty(&json_data) {
            Ok(s) => s,
            Err(e) => {
                // If serialization fails, the tasks have already been removed
                // Log error but return error result
                tracing::error!("Failed to serialize tasks: {}", e);
                return HippoxResult::system_error(format!("Failed to serialize tasks: {}", e));
            }
        };
        match fs::write(&path, json_string) {
            Ok(_) => {
                info!(
                    "Successfully backed up and removed {} terminal tasks to: {}",
                    removed_count, path
                );
                HippoxResult::ok(())
            }
            Err(e) => {
                // File write failed, but tasks are already removed!
                // This is a problem - data loss has occurred.
                tracing::error!(
                    "Failed to write backup file after removing tasks! Data loss occurred. Path: {}, Error: {}",
                    path,
                    e
                );
                HippoxResult::system_error(format!(
                    "Failed to write file {} after removing tasks (data may be lost): {}",
                    path, e
                ))
            }
        }
    }
}
