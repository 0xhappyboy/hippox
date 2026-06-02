//! Task management module for Hippox core
//!
//! This module provides a GLOBAL static task pool (independent of Hippox instances)
//! with automatic background execution engine that starts at program load.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Notify, RwLock};
use uuid::Uuid;

/// Maximum number of concurrent tasks allowed
const MAX_CONCURRENT_TASKS: usize = 10;

/// Maximum number of tasks to keep in history
const MAX_HISTORY_TASKS: usize = 100;

/// Global static task pool (auto-initialized at program start)
pub static TASK_POOL: Lazy<Arc<RwLock<TaskPool>>> = Lazy::new(|| {
    let pool = Arc::new(RwLock::new(TaskPool::new()));
    // Start the execution engine automatically
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        start_execution_engine(pool_clone);
    });
    pool
});

/// Global notifier for task queue (wakes up the execution engine)
static TASK_NOTIFIER: Lazy<Notify> = Lazy::new(Notify::new);

fn start_execution_engine(pool: Arc<RwLock<TaskPool>>) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            run_execution_engine(pool).await;
        });
        return;
    }
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for execution engine");
        rt.block_on(async {
            run_execution_engine(pool).await;
        });
    });
}

/// Background execution engine - runs automatically when the static variable is initialized
async fn run_execution_engine(task_pool: Arc<RwLock<TaskPool>>) {
    loop {
        // Check if engine is shutting down
        {
            let pool = task_pool.read().await;
            if pool.is_shutdown() {
                break;
            }
        }

        // Get the next task to execute
        let task_id = {
            let mut pool = task_pool.write().await;
            pool.next_task()
        };

        if let Some(task_id) = task_id {
            // Get the executable task
            let executable = {
                let pool = task_pool.read().await;
                pool.get_task(&task_id).and_then(|t| t.get_executable())
            };

            if let Some(executable_task) = executable {
                let step_callback = StepCallback::new(task_id.clone());
                executable_task.execute(step_callback).await;
            } else {
                // No executable found, mark task as failed
                let mut pool = task_pool.write().await;
                if let Some(task) = pool.get_task_mut(&task_id) {
                    task.failed("No executable associated with task".to_string());
                }
                pool.complete_task(&task_id);
            }
        } else {
            // No tasks to execute, wait for notification
            TASK_NOTIFIER.notified().await;
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Overall task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
    Timeout,
}

/// Individual step status within a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Waiting,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Execution result of a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: usize,
    pub skill_name: String,
    pub status: StepStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub duration_ms: Option<u64>,
}

impl StepResult {
    pub fn new(step_index: usize, skill_name: String) -> Self {
        Self {
            step_index,
            skill_name,
            status: StepStatus::Waiting,
            output: None,
            error: None,
            start_time: None,
            end_time: None,
            duration_ms: None,
        }
    }

    pub fn started(&mut self) {
        self.status = StepStatus::Running;
        self.start_time = Some(Self::now_timestamp());
    }

    pub fn completed(&mut self, output: String) {
        self.status = StepStatus::Completed;
        self.output = Some(output);
        self.end_time = Some(Self::now_timestamp());
        if let Some(start) = self.start_time {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub fn failed(&mut self, error: String) {
        self.status = StepStatus::Failed;
        self.error = Some(error);
        self.end_time = Some(Self::now_timestamp());
        if let Some(start) = self.start_time {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Task trait - each task must implement this to be executable
pub trait ExecutableTask: Send + Sync + Debug {
    /// Execute the task with the given step callback
    fn execute(&self, step_callback: StepCallback)
    -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    /// Get the task type identifier
    fn task_type(&self) -> &str;
    /// Get the input for the task
    fn input(&self) -> &str;
}

/// Task structure representing a single execution unit
pub struct Task {
    pub id: String,
    pub task_type: String,
    pub input: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub steps: Vec<StepResult>,
    pub final_output: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub timeout_secs: u64,
    pub interruptible: bool,
    pub resume_data: Option<String>,
    executable: Option<Arc<dyn ExecutableTask>>,
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            task_type: self.task_type.clone(),
            input: self.input.clone(),
            status: self.status.clone(),
            priority: self.priority.clone(),
            steps: self.steps.clone(),
            final_output: self.final_output.clone(),
            error: self.error.clone(),
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            duration_ms: self.duration_ms,
            timeout_secs: self.timeout_secs,
            interruptible: self.interruptible,
            resume_data: self.resume_data.clone(),
            executable: None, // Skip cloning executable
        }
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("task_type", &self.task_type)
            .field("input", &self.input)
            .field("status", &self.status)
            .field("priority", &self.priority)
            .field("steps", &self.steps)
            .field("final_output", &self.final_output)
            .field("error", &self.error)
            .field("created_at", &self.created_at)
            .field("started_at", &self.started_at)
            .field("completed_at", &self.completed_at)
            .field("duration_ms", &self.duration_ms)
            .field("timeout_secs", &self.timeout_secs)
            .field("interruptible", &self.interruptible)
            .field("resume_data", &self.resume_data)
            .field("executable", &"<skipped>")
            .finish()
    }
}

impl Task {
    pub fn new(task_type: String, input: String) -> Self {
        let now = Self::now_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            input,
            status: TaskStatus::Pending,
            priority: TaskPriority::Normal,
            steps: Vec::new(),
            final_output: None,
            error: None,
            created_at: now,
            started_at: None,
            completed_at: None,
            duration_ms: None,
            timeout_secs: 0,
            interruptible: true,
            resume_data: None,
            executable: None,
        }
    }

    pub fn with_executable(mut self, executable: Arc<dyn ExecutableTask>) -> Self {
        self.task_type = executable.task_type().to_string();
        self.input = executable.input().to_string();
        self.executable = Some(executable);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn non_interruptible(mut self) -> Self {
        self.interruptible = false;
        self
    }

    pub fn started(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Self::now_timestamp());
    }

    pub fn completed(&mut self, output: String) {
        self.status = TaskStatus::Completed;
        self.final_output = Some(output);
        self.completed_at = Some(Self::now_timestamp());
        if let Some(start) = self.started_at {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub fn failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Self::now_timestamp());
        if let Some(start) = self.started_at {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub fn cancelled(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Self::now_timestamp());
        if let Some(start) = self.started_at {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub fn paused(&mut self) -> bool {
        if self.status == TaskStatus::Running && self.interruptible {
            self.status = TaskStatus::Paused;
            true
        } else {
            false
        }
    }

    pub fn resume(&mut self) -> bool {
        if self.status == TaskStatus::Paused {
            self.status = TaskStatus::Running;
            true
        } else {
            false
        }
    }

    pub fn is_timed_out(&self) -> bool {
        if self.timeout_secs > 0 && self.status == TaskStatus::Running {
            if let Some(started_at) = self.started_at {
                let elapsed = Self::now_timestamp() - started_at;
                return elapsed > self.timeout_secs;
            }
        }
        false
    }

    pub fn add_step(&mut self, skill_name: String) -> usize {
        let step_index = self.steps.len();
        self.steps.push(StepResult::new(step_index, skill_name));
        step_index
    }

    pub fn get_step_mut(&mut self, step_index: usize) -> Option<&mut StepResult> {
        self.steps.get_mut(step_index)
    }

    pub fn progress(&self) -> u8 {
        if self.steps.is_empty() {
            match self.status {
                TaskStatus::Completed => 100,
                TaskStatus::Failed | TaskStatus::Cancelled => 0,
                _ => 0,
            }
        } else {
            let completed = self
                .steps
                .iter()
                .filter(|s| s.status == StepStatus::Completed)
                .count();
            ((completed * 100) / self.steps.len()) as u8
        }
    }

    pub fn get_executable(&self) -> Option<Arc<dyn ExecutableTask>> {
        self.executable.clone()
    }

    fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Step callback for tracking step execution progress
#[derive(Clone)]
pub struct StepCallback {
    task_id: String,
}

impl StepCallback {
    pub fn new(task_id: String) -> Self {
        Self { task_id }
    }

    /// Called when a step starts
    pub async fn on_step_start(&self, step_name: &str, step_index: usize) {
        let mut pool = TASK_POOL.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            if step_index < task.steps.len() {
                task.steps[step_index].started();
            } else {
                let idx = task.add_step(step_name.to_string());
                if idx == step_index {
                    if let Some(step) = task.get_step_mut(step_index) {
                        step.started();
                    }
                }
            }
        }
    }

    /// Called when a step completes successfully
    pub async fn on_step_success(&self, step_name: &str, step_index: usize, output: &str) {
        let mut pool = TASK_POOL.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            if step_index < task.steps.len() {
                task.steps[step_index].completed(output.to_string());
            } else {
                let idx = task.add_step(step_name.to_string());
                if idx == step_index {
                    if let Some(step) = task.get_step_mut(step_index) {
                        step.completed(output.to_string());
                    }
                }
            }
        }
    }

    /// Called when a step fails
    pub async fn on_step_failure(&self, step_name: &str, step_index: usize, error: &str) {
        let mut pool = TASK_POOL.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            if step_index < task.steps.len() {
                task.steps[step_index].failed(error.to_string());
            } else {
                let idx = task.add_step(step_name.to_string());
                if idx == step_index {
                    if let Some(step) = task.get_step_mut(step_index) {
                        step.failed(error.to_string());
                    }
                }
            }
        }
    }

    /// Called when the entire workflow completes
    pub async fn on_workflow_complete(&self, final_output: &str) {
        let mut pool = TASK_POOL.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            task.completed(final_output.to_string());
        }
    }

    /// Called when the workflow fails
    pub async fn on_workflow_failed(&self, error: &str) {
        let mut pool = TASK_POOL.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            task.failed(error.to_string());
        }
    }
}

/// Global task pool structure
pub struct TaskPool {
    tasks: HashMap<String, Task>,
    pending_queue: VecDeque<String>,
    running_tasks: Vec<String>,
    max_concurrent: usize,
    task_counter: AtomicUsize,
    shutdown: AtomicBool,
}

impl TaskPool {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            pending_queue: VecDeque::new(),
            running_tasks: Vec::new(),
            max_concurrent: MAX_CONCURRENT_TASKS,
            task_counter: AtomicUsize::new(0),
            shutdown: AtomicBool::new(false),
        }
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        TASK_NOTIFIER.notify_one();
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Create and register a new task
    pub fn create_task(&mut self, task_type: String, input: String) -> String {
        let task = Task::new(task_type, input);
        let task_id = task.id.clone();
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.tasks.insert(task_id.clone(), task);
        self.enqueue_task(&task_id);
        TASK_NOTIFIER.notify_one();
        task_id
    }

    /// Create and register a new task with executable
    pub fn create_task_with_executable(
        &mut self,
        task_type: String,
        input: String,
        executable: Arc<dyn ExecutableTask>,
    ) -> String {
        let task = Task::new(task_type, input).with_executable(executable);
        let task_id = task.id.clone();
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.tasks.insert(task_id.clone(), task);
        self.enqueue_task(&task_id);
        TASK_NOTIFIER.notify_one();
        task_id
    }

    fn enqueue_task(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get(task_id) {
            let priority = task.priority;
            let insert_pos = self
                .pending_queue
                .iter()
                .position(|id| {
                    self.tasks
                        .get(id)
                        .map(|t| t.priority < priority)
                        .unwrap_or(false)
                })
                .unwrap_or(self.pending_queue.len());
            self.pending_queue.insert(insert_pos, task_id.to_string());
        }
    }

    /// Get the next task to execute (internal use by execution engine)
    fn next_task(&mut self) -> Option<String> {
        if self.running_tasks.len() >= self.max_concurrent {
            return None;
        }
        while let Some(task_id) = self.pending_queue.pop_front() {
            if let Some(task) = self.tasks.get(&task_id) {
                if task.status == TaskStatus::Pending {
                    self.running_tasks.push(task_id.clone());
                    return Some(task_id);
                }
            }
        }
        None
    }

    /// Mark a task as completed (remove from running queue)
    fn complete_task(&mut self, task_id: &str) {
        self.running_tasks.retain(|id| id != task_id);
    }

    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.get(task_id).cloned()
    }

    fn get_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(task_id)
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            match status {
                TaskStatus::Running => {
                    if task.status == TaskStatus::Pending {
                        task.started();
                        return true;
                    } else if task.status == TaskStatus::Paused {
                        task.status = TaskStatus::Running;
                        return true;
                    }
                    false
                }
                TaskStatus::Completed => {
                    if let Some(output) = task.final_output.clone() {
                        task.completed(output);
                    } else {
                        task.completed(String::new());
                    }
                    self.complete_task(task_id);
                    true
                }
                TaskStatus::Failed => {
                    let error = task.error.clone().unwrap_or_default();
                    task.failed(error);
                    self.complete_task(task_id);
                    true
                }
                TaskStatus::Cancelled => {
                    task.cancelled();
                    self.complete_task(task_id);
                    true
                }
                TaskStatus::Paused => {
                    task.paused();
                    self.complete_task(task_id);
                    true
                }
                _ => true,
            }
        } else {
            false
        }
    }

    pub fn cancel_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.status == TaskStatus::Pending {
                task.cancelled();
                self.pending_queue.retain(|id| id != task_id);
                return true;
            } else if task.status == TaskStatus::Running && task.interruptible {
                task.cancelled();
                self.running_tasks.retain(|id| id != task_id);
                return true;
            } else if task.status == TaskStatus::Paused {
                task.cancelled();
                return true;
            }
        }
        false
    }

    pub fn pause_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.status == TaskStatus::Running && task.interruptible {
                task.status = TaskStatus::Paused;
                self.running_tasks.retain(|id| id != task_id);
                return true;
            }
        }
        false
    }

    pub fn resume_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get(task_id) {
            if task.status == TaskStatus::Paused {
                self.enqueue_task(task_id);
                TASK_NOTIFIER.notify_one();
                return true;
            }
        }
        false
    }

    pub fn retry_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.status == TaskStatus::Failed {
                task.status = TaskStatus::Pending;
                task.error = None;
                task.completed_at = None;
                task.duration_ms = None;
                for step in &mut task.steps {
                    if step.status == StepStatus::Failed {
                        step.status = StepStatus::Waiting;
                        step.error = None;
                        step.output = None;
                        step.end_time = None;
                        step.duration_ms = None;
                    }
                }
                self.enqueue_task(task_id);
                TASK_NOTIFIER.notify_one();
                return true;
            }
        }
        false
    }

    pub fn get_all_tasks(&self, limit: Option<usize>) -> Vec<Task> {
        let mut tasks: Vec<Task> = self.tasks.values().cloned().collect();
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let limit = limit.unwrap_or(MAX_HISTORY_TASKS);
        tasks.into_iter().take(limit).collect()
    }

    pub fn running_count(&self) -> usize {
        self.running_tasks.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_queue.len()
    }

    pub fn has_task(&self, task_id: &str) -> bool {
        self.tasks.contains_key(task_id)
    }
}

/// Create a new task (without executable)
pub async fn create_task(task_type: String, input: String) -> String {
    let mut pool = TASK_POOL.write().await;
    pool.create_task(task_type, input)
}

/// Create a new task with executable
pub async fn create_task_with_executable(
    task_type: String,
    input: String,
    executable: Arc<dyn ExecutableTask>,
) -> String {
    let mut pool = TASK_POOL.write().await;
    pool.create_task_with_executable(task_type, input, executable)
}

/// Get task by ID
pub async fn get_task(task_id: &str) -> Option<Task> {
    let pool = TASK_POOL.read().await;
    pool.get_task(task_id)
}

/// Get task status by ID
pub async fn get_task_status(task_id: &str) -> Option<TaskStatus> {
    let pool = TASK_POOL.read().await;
    pool.get_task(task_id).map(|t| t.status)
}

/// Update task status
pub async fn update_task_status(task_id: &str, status: TaskStatus) -> bool {
    let mut pool = TASK_POOL.write().await;
    pool.update_task_status(task_id, status)
}

/// Cancel a task
pub async fn cancel_task(task_id: &str) -> bool {
    let mut pool = TASK_POOL.write().await;
    pool.cancel_task(task_id)
}

/// Pause a task
pub async fn pause_task(task_id: &str) -> bool {
    let mut pool = TASK_POOL.write().await;
    pool.pause_task(task_id)
}

/// Resume a paused task
pub async fn resume_task(task_id: &str) -> bool {
    let mut pool = TASK_POOL.write().await;
    pool.resume_task(task_id)
}

/// Retry a failed task
pub async fn retry_task(task_id: &str) -> bool {
    let mut pool = TASK_POOL.write().await;
    pool.retry_task(task_id)
}

/// Get all tasks
pub async fn get_all_tasks(limit: Option<usize>) -> Vec<Task> {
    let pool = TASK_POOL.read().await;
    pool.get_all_tasks(limit)
}

/// Set maximum concurrent tasks
pub async fn set_max_concurrent(max: usize) {
    let mut pool = TASK_POOL.write().await;
    pool.set_max_concurrent(max);
}

/// Get running task count
pub async fn running_count() -> usize {
    let pool = TASK_POOL.read().await;
    pool.running_count()
}

/// Get pending task count
pub async fn pending_count() -> usize {
    let pool = TASK_POOL.read().await;
    pool.pending_count()
}

/// Shutdown the task pool
pub async fn shutdown_task_pool() {
    let mut pool = TASK_POOL.write().await;
    pool.shutdown();
}
