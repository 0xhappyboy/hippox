//! Public API functions for task management

use std::sync::Arc;

use super::executor::ExecutableTask;
use super::types::{MAX_HISTORY_TASKS, TASK_POOL, Task, TaskPool, TaskStatus};
use crate::workflow::WorkflowCallback;

/// Create a new task (without executable)
pub async fn create_task(task_type: String, input: String) -> String {
    let mut pool = TASK_POOL.write().await;
    pool.create_task(task_type, input)
}

/// Create a new task with executable and callback
pub async fn create_task_with_executable(
    task_type: String,
    input: String,
    executable: Arc<dyn ExecutableTask>,
    callback: Option<Arc<dyn WorkflowCallback>>,
) -> String {
    let mut pool = TASK_POOL.write().await;
    pool.create_task_with_executable(task_type, input, executable, callback)
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
