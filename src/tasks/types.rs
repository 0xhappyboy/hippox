//! Data models for task management

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Notify, RwLock};
use uuid::Uuid;

use super::executor::ExecutableTask;
use crate::workflow::WorkflowCallback;

/// Maximum number of concurrent tasks allowed
pub const MAX_CONCURRENT_TASKS: usize = 10;

/// Maximum number of tasks to keep in history
pub const MAX_HISTORY_TASKS: usize = 100;

/// Global static task pool (auto-initialized at program start)
pub static TASK_POOL: Lazy<Arc<RwLock<TaskPool>>> = Lazy::new(|| {
    let pool = Arc::new(RwLock::new(TaskPool::new()));
    let pool_clone = pool.clone();
    crate::tasks::engine::start_execution_engine(pool_clone);
    pool
});

/// Global notifier for task queue (wakes up the execution engine)
pub static TASK_NOTIFIER: Lazy<Notify> = Lazy::new(Notify::new);

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
    pub(crate) executable: Option<Arc<dyn ExecutableTask>>,
    pub callback: Option<Arc<dyn WorkflowCallback>>,
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
            executable: self.executable.clone(),
            callback: self.callback.clone(),
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
            .field("callback", &self.callback.as_ref().map(|_| "<callback>"))
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
            callback: None,
        }
    }

    pub fn with_executable(mut self, executable: Arc<dyn ExecutableTask>) -> Self {
        self.task_type = executable.task_type().to_string();
        self.input = executable.input().to_string();
        self.executable = Some(executable);
        self
    }

    pub fn with_callback(mut self, callback: Option<Arc<dyn WorkflowCallback>>) -> Self {
        self.callback = callback;
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

/// Global task pool structure
pub struct TaskPool {
    pub(crate) tasks: HashMap<String, Task>,
    pub(crate) pending_queue: VecDeque<String>,
    pub(crate) running_tasks: Vec<String>,
    pub(crate) max_concurrent: usize,
    pub(crate) task_counter: AtomicUsize,
    pub(crate) shutdown: AtomicBool,
}

impl TaskPool {
    pub fn new() -> Self {
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

    pub fn create_task(&mut self, task_type: String, input: String) -> String {
        let task = Task::new(task_type, input);
        let task_id = task.id.clone();
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.tasks.insert(task_id.clone(), task);
        self.enqueue_task(&task_id);
        TASK_NOTIFIER.notify_one();
        task_id
    }

    pub fn create_task_with_executable(
        &mut self,
        task_type: String,
        input: String,
        executable: Arc<dyn ExecutableTask>,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> String {
        let task = Task::new(task_type, input)
            .with_executable(executable)
            .with_callback(callback);
        let task_id = task.id.clone();
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.tasks.insert(task_id.clone(), task);
        self.enqueue_task(&task_id);
        TASK_NOTIFIER.notify_one();
        task_id
    }

    pub(crate) fn enqueue_task(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get(task_id) {
            // Allow both Pending and Paused tasks to be enqueued
            if task.status == TaskStatus::Pending || task.status == TaskStatus::Paused {
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
    }

    pub(crate) fn next_task(&mut self) -> Option<String> {
        if self.running_tasks.len() >= self.max_concurrent {
            return None;
        }
        while let Some(task_id) = self.pending_queue.pop_front() {
            if let Some(task) = self.tasks.get(&task_id) {
                // Allow both Pending and Paused tasks to be executed
                if task.status == TaskStatus::Pending || task.status == TaskStatus::Paused {
                    // When resuming a paused task, change its status back to Running
                    if task.status == TaskStatus::Paused {
                        if let Some(task_mut) = self.tasks.get_mut(&task_id) {
                            task_mut.status = TaskStatus::Running;
                        }
                    }
                    self.running_tasks.push(task_id.clone());
                    return Some(task_id);
                }
            }
        }
        None
    }

    pub(crate) fn complete_task(&mut self, task_id: &str) {
        self.running_tasks.retain(|id| id != task_id);
    }

    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.get(task_id).cloned()
    }

    pub(crate) fn get_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
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
                    // Allow pausing both Running and Pending tasks
                    if (task.status == TaskStatus::Running || task.status == TaskStatus::Pending)
                        && task.interruptible
                    {
                        task.status = TaskStatus::Paused;
                        // Remove from running and pending queues but DO NOT call complete_task
                        self.running_tasks.retain(|id| id != task_id);
                        self.pending_queue.retain(|id| id != task_id);
                        return true;
                    }
                    false
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
            if (task.status == TaskStatus::Running || task.status == TaskStatus::Pending)
                && task.interruptible
            {
                task.status = TaskStatus::Paused;
                self.running_tasks.retain(|id| id != task_id);
                self.pending_queue.retain(|id| id != task_id);
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
