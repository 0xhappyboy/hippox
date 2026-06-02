use crate::config::{
    init_config_from_json_file, init_config_from_params_json_str, init_config_from_toml_file,
};
use crate::executors::Executor;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::tasks::{self, ExecutableTask, StepCallback, TaskPool, TaskStatus};
use crate::workflow::{WorkflowCallback, WorkflowExecutor, WorkflowMode};
use crate::{HippoxConfig, i18n};
use crate::{get_config, t};
use langhub::LLMClient;
use langhub::types::ModelProvider;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
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

/// Task for natural language processing
#[derive(Debug)]
pub(crate) struct NaturalLanguageTask {
    input: String,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
    skills_registry: String,
    instances_registry: String,
}

impl NaturalLanguageTask {
    pub(crate) fn new(
        input: String,
        workflow_executor: WorkflowExecutor,
        scheduler: SkillScheduler,
        skills_registry: String,
        instances_registry: String,
    ) -> Self {
        Self {
            input,
            workflow_executor,
            scheduler,
            skills_registry,
            instances_registry,
        }
    }
}

impl ExecutableTask for NaturalLanguageTask {
    fn execute(
        &self,
        step_callback: StepCallback,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let input = self.input.clone();
        let workflow_executor = self.workflow_executor.clone();
        let scheduler = self.scheduler.clone();
        let skills_registry = self.skills_registry.clone();
        let instances_registry = self.instances_registry.clone();

        Box::pin(async move {
            step_callback.on_step_start("natural_language", 0).await;
            let result = workflow_executor
                .execute(&scheduler, &input, &skills_registry, &instances_registry)
                .await;
            step_callback.on_workflow_complete(&result).await;
        })
    }
}

/// Task for SKILL.md file execution
#[derive(Debug)]
pub(crate) struct SkillMdTask {
    path: String,
    params: Option<HashMap<String, Value>>,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
    skills_registry: String,
    instances_registry: String,
}

impl SkillMdTask {
    pub(crate) fn new(
        path: String,
        params: Option<HashMap<String, Value>>,
        workflow_executor: WorkflowExecutor,
        scheduler: SkillScheduler,
        skills_registry: String,
        instances_registry: String,
    ) -> Self {
        Self {
            path,
            params,
            workflow_executor,
            scheduler,
            skills_registry,
            instances_registry,
        }
    }
}

impl ExecutableTask for SkillMdTask {
    fn execute(
        &self,
        step_callback: StepCallback,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let path = self.path.clone();
        let params = self.params.clone();
        let workflow_executor = self.workflow_executor.clone();
        let scheduler = self.scheduler.clone();
        let skills_registry = self.skills_registry.clone();
        let instances_registry = self.instances_registry.clone();

        Box::pin(async move {
            step_callback.on_step_start("skill_md", 0).await;

            let skill_file = match SkillLoader::load_from_path(&path) {
                Ok(Some(file)) => file,
                Ok(None) => {
                    let error_msg = format!("{}: {}", t!("error.skill_not_found"), path);
                    step_callback.on_workflow_failed(&error_msg).await;
                    return;
                }
                Err(e) => {
                    let error_msg = format!("{}: {}", t!("error.load_skill_failed"), e);
                    step_callback.on_workflow_failed(&error_msg).await;
                    return;
                }
            };

            let result = workflow_executor
                .execute_skill_md(
                    &scheduler,
                    &skill_file,
                    params.as_ref(),
                    &skills_registry,
                    &instances_registry,
                )
                .await;

            step_callback.on_workflow_complete(&result).await;
        })
    }
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
    task_pool: Arc<Mutex<TaskPool>>,
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
        // Create task pool for this instance
        let task_pool = Arc::new(Mutex::new(TaskPool::new()));
        // Start the execution engine
        {
            let mut pool = task_pool.lock().await;
            pool.start_engine(task_pool.clone());
        }
        Ok(Self {
            scheduler,
            executor,
            workflow_mode,
            workflow_executor,
            is_first_message: Arc::new(AtomicBool::new(false)),
            task_pool,
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

    /// Submit a natural language task and return task ID immediately
    ///
    /// This function creates a task, adds it to the task pool, and returns the task ID.
    /// The actual execution happens asynchronously in the background.
    ///
    /// # Arguments
    /// * `input` - Natural language input from the user
    /// * `_session_id` - Optional session ID (unused in core, for compatibility)
    /// * `_callback` - Optional callback for workflow execution progress
    ///
    /// # Returns
    /// The task ID as a string
    pub fn handle_natural_language(
        &self,
        input: &str,
        _session_id: Option<&str>,
        _callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let skills_registry = self.get_skills_registry();
        let instances_registry = self.get_instances_registry();

        let executable = Arc::new(NaturalLanguageTask::new(
            input.to_string(),
            self.workflow_executor.clone(),
            self.scheduler.clone(),
            skills_registry,
            instances_registry,
        ));

        let task_id = futures::executor::block_on(async {
            let mut pool = self.task_pool.lock().await;
            pool.create_task_with_executable(
                "natural_language".to_string(),
                input.to_string(),
                executable,
            )
        });

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
    pub fn handle_natural_language_batch(
        &self,
        inputs: Vec<(String, Option<String>, Option<Arc<dyn WorkflowCallback>>)>,
    ) -> Vec<String> {
        inputs
            .into_iter()
            .map(|(input, session_id, callback)| {
                self.handle_natural_language(&input, session_id.as_deref(), callback)
            })
            .collect()
    }

    /// Submit a SKILL.md workflow task and return task ID immediately
    ///
    /// # Arguments
    /// * `skill_md_path` - Path to the SKILL.md file
    /// * `params` - Optional parameters to substitute in the SKILL.md content
    /// * `_callback` - Optional callback for workflow execution progress
    ///
    /// # Returns
    /// The task ID as a string
    pub fn handle_skill_md(
        &self,
        skill_md_path: &str,
        params: Option<HashMap<String, Value>>,
        _callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let skills_registry = self.get_skills_registry();
        let instances_registry = self.get_instances_registry();

        let executable = Arc::new(SkillMdTask::new(
            skill_md_path.to_string(),
            params,
            self.workflow_executor.clone(),
            self.scheduler.clone(),
            skills_registry,
            instances_registry,
        ));

        let task_id = futures::executor::block_on(async {
            let mut pool = self.task_pool.lock().await;
            pool.create_task_with_executable(
                "skill_md".to_string(),
                skill_md_path.to_string(),
                executable,
            )
        });

        info!(
            "Created SKILL.md task: {} for path: {}",
            task_id, skill_md_path
        );
        task_id
    }

    /// Submit multiple SKILL.md tasks in batch and return task IDs immediately
    ///
    /// # Arguments
    /// * `tasks` - Vector of tuples (skill_md_path, params, callback)
    ///
    /// # Returns
    /// Vector of task IDs in the same order as inputs
    pub fn handle_skill_md_batch(
        &self,
        tasks: Vec<(
            String,
            Option<HashMap<String, Value>>,
            Option<Arc<dyn WorkflowCallback>>,
        )>,
    ) -> Vec<String> {
        tasks
            .into_iter()
            .map(|(path, params, callback)| self.handle_skill_md(&path, params, callback))
            .collect()
    }

    /// Get task status by ID
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        futures::executor::block_on(async {
            let pool = self.task_pool.lock().await;
            pool.get_task(task_id).map(|t| t.status)
        })
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<tasks::Task> {
        futures::executor::block_on(async {
            let pool = self.task_pool.lock().await;
            pool.get_task(task_id)
        })
    }

    /// Cancel a running or pending task
    pub fn cancel_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(async {
            let mut pool = self.task_pool.lock().await;
            pool.cancel_task(task_id)
        })
    }

    /// Pause a running task
    pub fn pause_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(async {
            let mut pool = self.task_pool.lock().await;
            pool.pause_task(task_id)
        })
    }

    /// Resume a paused task
    pub fn resume_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(async {
            let mut pool = self.task_pool.lock().await;
            pool.resume_task(task_id)
        })
    }

    /// Retry a failed task
    pub fn retry_task(&self, task_id: &str) -> bool {
        futures::executor::block_on(async {
            let mut pool = self.task_pool.lock().await;
            pool.retry_task(task_id)
        })
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
}
