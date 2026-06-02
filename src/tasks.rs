//! Task management module for Hippox core
//!
//! This module provides a task pool for managing async task execution,
//! including task lifecycle management and automatic execution engine.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

/// Maximum number of concurrent tasks allowed
const MAX_CONCURRENT_TASKS: usize = 5;

/// Maximum number of tasks to keep in history
const MAX_HISTORY_TASKS: usize = 100;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TaskPriority {
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
pub(crate) enum TaskStatus {
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
pub(crate) enum StepStatus {
    Waiting,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Execution result of a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StepResult {
    pub(crate) step_index: usize,
    pub(crate) skill_name: String,
    pub(crate) status: StepStatus,
    pub(crate) output: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) start_time: Option<u64>,
    pub(crate) end_time: Option<u64>,
    pub(crate) duration_ms: Option<u64>,
}

impl StepResult {
    pub(crate) fn new(step_index: usize, skill_name: String) -> Self {
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

    pub(crate) fn started(&mut self) {
        self.status = StepStatus::Running;
        self.start_time = Some(Self::now_timestamp());
    }

    pub(crate) fn completed(&mut self, output: String) {
        self.status = StepStatus::Completed;
        self.output = Some(output);
        self.end_time = Some(Self::now_timestamp());
        if let Some(start) = self.start_time {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub(crate) fn failed(&mut self, error: String) {
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
pub(crate) trait ExecutableTask: Send + Sync + Debug {
    /// Execute the task with the given step callback
    fn execute(&self, step_callback: StepCallback)
    -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

/// Task structure representing a single execution unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Task {
    pub(crate) id: String,
    pub(crate) task_type: String,
    pub(crate) input: String,
    pub(crate) status: TaskStatus,
    pub(crate) priority: TaskPriority,
    pub(crate) steps: Vec<StepResult>,
    pub(crate) final_output: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) created_at: u64,
    pub(crate) started_at: Option<u64>,
    pub(crate) completed_at: Option<u64>,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) timeout_secs: u64,
    pub(crate) interruptible: bool,
    #[serde(skip)]
    pub(crate) resume_data: Option<String>,
    #[serde(skip)]
    pub(crate) executable: Option<Arc<dyn ExecutableTask>>,
}

impl Task {
    pub(crate) fn new(task_type: String, input: String) -> Self {
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

    pub(crate) fn with_executable(mut self, executable: Arc<dyn ExecutableTask>) -> Self {
        self.executable = Some(executable);
        self
    }

    pub(crate) fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub(crate) fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub(crate) fn non_interruptible(mut self) -> Self {
        self.interruptible = false;
        self
    }

    pub(crate) fn started(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Self::now_timestamp());
    }

    pub(crate) fn completed(&mut self, output: String) {
        self.status = TaskStatus::Completed;
        self.final_output = Some(output);
        self.completed_at = Some(Self::now_timestamp());
        if let Some(start) = self.started_at {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub(crate) fn failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Self::now_timestamp());
        if let Some(start) = self.started_at {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub(crate) fn cancelled(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Self::now_timestamp());
        if let Some(start) = self.started_at {
            self.duration_ms = Some((Self::now_timestamp() - start) * 1000);
        }
    }

    pub(crate) fn paused(&mut self) -> bool {
        if self.status == TaskStatus::Running && self.interruptible {
            self.status = TaskStatus::Paused;
            true
        } else {
            false
        }
    }

    pub(crate) fn resume(&mut self) -> bool {
        if self.status == TaskStatus::Paused {
            self.status = TaskStatus::Running;
            true
        } else {
            false
        }
    }

    pub(crate) fn is_timed_out(&self) -> bool {
        if self.timeout_secs > 0 && self.status == TaskStatus::Running {
            if let Some(started_at) = self.started_at {
                let elapsed = Self::now_timestamp() - started_at;
                return elapsed > self.timeout_secs;
            }
        }
        false
    }

    pub(crate) fn add_step(&mut self, skill_name: String) -> usize {
        let step_index = self.steps.len();
        self.steps.push(StepResult::new(step_index, skill_name));
        step_index
    }

    pub(crate) fn get_step_mut(&mut self, step_index: usize) -> Option<&mut StepResult> {
        self.steps.get_mut(step_index)
    }

    pub(crate) fn progress(&self) -> u8 {
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

    fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Step callback for tracking step execution progress
#[derive(Clone)]
pub(crate) struct StepCallback {
    task_id: String,
    task_pool: Arc<Mutex<TaskPool>>,
}

impl StepCallback {
    pub(crate) fn new(task_id: String, task_pool: Arc<Mutex<TaskPool>>) -> Self {
        Self { task_id, task_pool }
    }

    /// Called when a step starts
    pub(crate) async fn on_step_start(&self, step_name: &str, step_index: usize) {
        let mut pool = self.task_pool.lock().await;
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
    pub(crate) async fn on_step_success(&self, step_name: &str, step_index: usize, output: &str) {
        let mut pool = self.task_pool.lock().await;
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
    pub(crate) async fn on_step_failure(&self, step_name: &str, step_index: usize, error: &str) {
        let mut pool = self.task_pool.lock().await;
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
    pub(crate) async fn on_workflow_complete(&self, final_output: &str) {
        let mut pool = self.task_pool.lock().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            task.completed(final_output.to_string());
        }
    }

    /// Called when the workflow fails
    pub(crate) async fn on_workflow_failed(&self, error: &str) {
        let mut pool = self.task_pool.lock().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            task.failed(error.to_string());
        }
    }
}

/// Task pool for managing all tasks for a single Hippox instance
#[derive(Debug)]
pub(crate) struct TaskPool {
    tasks: HashMap<String, Task>,
    pending_queue: VecDeque<String>,
    running_tasks: Vec<String>,
    max_concurrent: usize,
    task_counter: AtomicUsize,
    shutdown: AtomicBool,
    notifier: Arc<Notify>,
    engine_handle: Option<tokio::task::JoinHandle<()>>,
}

impl TaskPool {
    pub(crate) fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            pending_queue: VecDeque::new(),
            running_tasks: Vec::new(),
            max_concurrent: MAX_CONCURRENT_TASKS,
            task_counter: AtomicUsize::new(0),
            shutdown: AtomicBool::new(false),
            notifier: Arc::new(Notify::new()),
            engine_handle: None,
        }
    }

    pub(crate) fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }

    pub(crate) fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub(crate) fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.notifier.notify_one();
        if let Some(handle) = self.engine_handle.take() {
            handle.abort();
        }
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Start the execution engine for this task pool
    pub(crate) fn start_engine(&mut self, task_pool_arc: Arc<Mutex<TaskPool>>) {
        let notifier = self.notifier.clone();
        let handle = tokio::spawn(Self::run_execution_engine(task_pool_arc, notifier));
        self.engine_handle = Some(handle);
    }

    /// Task execution engine - runs in the background, automatically executes queued tasks
    async fn run_execution_engine(task_pool: Arc<Mutex<TaskPool>>, notifier: Arc<Notify>) {
        loop {
            // Check if engine is shutting down
            {
                let pool = task_pool.lock().await;
                if pool.is_shutdown() {
                    break;
                }
            }
            // Get the next task to execute
            let task_id = {
                let mut pool = task_pool.lock().await;
                pool.next_task()
            };
            if let Some(task_id) = task_id {
                // Get the executable task
                let executable = {
                    let pool = task_pool.lock().await;
                    pool.get_task(&task_id).and_then(|t| t.executable.clone())
                };
                if let Some(executable_task) = executable {
                    let step_callback = StepCallback::new(task_id.clone(), task_pool.clone());
                    executable_task.execute(step_callback).await;
                } else {
                    // No executable found, mark task as failed
                    let mut pool = task_pool.lock().await;
                    if let Some(task) = pool.get_task_mut(&task_id) {
                        task.failed("No executable associated with task".to_string());
                    }
                    pool.complete_task(&task_id);
                }
            } else {
                // No tasks to execute, wait for notification
                notifier.notified().await;
            }
        }
    }

    /// Create and register a new task
    pub(crate) fn create_task(&mut self, task_type: String, input: String) -> String {
        let task = Task::new(task_type, input);
        let task_id = task.id.clone();
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.tasks.insert(task_id.clone(), task);
        self.enqueue_task(&task_id);
        self.notifier.notify_one();
        task_id
    }

    /// Create and register a new task with executable
    pub(crate) fn create_task_with_executable(
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
        self.notifier.notify_one();
        task_id
    }

    pub(crate) fn create_task_with<F>(&mut self, builder: F) -> String
    where
        F: FnOnce(Task) -> Task,
    {
        let base_task = Task::new(String::new(), String::new());
        let task = builder(base_task);
        let task_id = task.id.clone();
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.tasks.insert(task_id.clone(), task);
        self.enqueue_task(&task_id);
        self.notifier.notify_one();
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

    pub(crate) fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.get(task_id).cloned()
    }

    fn get_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(task_id)
    }

    pub(crate) fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> bool {
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

    pub(crate) fn cancel_task(&mut self, task_id: &str) -> bool {
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

    pub(crate) fn pause_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.status == TaskStatus::Running && task.interruptible {
                task.status = TaskStatus::Paused;
                self.running_tasks.retain(|id| id != task_id);
                return true;
            }
        }
        false
    }

    pub(crate) fn resume_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get(task_id) {
            if task.status == TaskStatus::Paused {
                self.enqueue_task(task_id);
                self.notifier.notify_one();
                return true;
            }
        }
        false
    }

    pub(crate) fn retry_task(&mut self, task_id: &str) -> bool {
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
                self.notifier.notify_one();
                return true;
            }
        }
        false
    }

    pub(crate) fn get_all_tasks(&self, limit: Option<usize>) -> Vec<Task> {
        let mut tasks: Vec<Task> = self.tasks.values().cloned().collect();
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let limit = limit.unwrap_or(MAX_HISTORY_TASKS);
        tasks.into_iter().take(limit).collect()
    }

    pub(crate) fn get_pending_tasks(&self) -> Vec<Task> {
        self.pending_queue
            .iter()
            .filter_map(|id| self.tasks.get(id))
            .cloned()
            .collect()
    }

    pub(crate) fn get_running_tasks(&self) -> Vec<Task> {
        self.running_tasks
            .iter()
            .filter_map(|id| self.tasks.get(id))
            .cloned()
            .collect()
    }

    pub(crate) fn running_count(&self) -> usize {
        self.running_tasks.len()
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.pending_queue.len()
    }

    pub(crate) fn has_task(&self, task_id: &str) -> bool {
        self.tasks.contains_key(task_id)
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::new()
    }
}
