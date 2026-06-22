//! Public API functions for task management

use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use super::executor::ExecutableTask;
use super::types::{MAX_HISTORY_TASKS, TASK_POOL, Task, TaskPool, TaskStatus};
use crate::workflow::WorkflowCallback;
use crate::{
    HippoxBatchResult, HippoxBoolResult, HippoxResult, HippoxStringResult, HippoxVoidResult,
};

/// Create a new task (without executable)
pub async fn create_task(task_type: String, input: String) -> HippoxStringResult {
    let mut pool = TASK_POOL.write().await;
    let task_id = pool.create_task(task_type, input);
    HippoxResult::ok(task_id)
}

/// Create a new task with executable
pub async fn create_task_with_executable(
    task_type: String,
    input: String,
    executable: Arc<dyn ExecutableTask>,
) -> HippoxStringResult {
    let mut pool = TASK_POOL.write().await;
    let task_id = pool.create_task_with_executable(task_type, input, executable);
    HippoxResult::ok(task_id)
}

/// Get task by ID
pub async fn get_task(task_id: &str) -> HippoxResult<Task> {
    let pool = TASK_POOL.read().await;
    match pool.get_task(task_id) {
        Some(task) => HippoxResult::ok(task),
        None => HippoxResult::system_error(format!("Task not found: {}", task_id)),
    }
}

/// Get task status by ID
pub async fn get_task_status(task_id: &str) -> HippoxResult<TaskStatus> {
    let pool = TASK_POOL.read().await;
    match pool.get_task(task_id) {
        Some(task) => HippoxResult::ok(task.status),
        None => HippoxResult::system_error(format!("Task not found: {}", task_id)),
    }
}

/// Update task status
pub async fn update_task_status(task_id: &str, status: TaskStatus) -> HippoxBoolResult {
    let mut pool = TASK_POOL.write().await;
    let success = pool.update_task_status(task_id, status);
    if success {
        HippoxResult::ok(true)
    } else {
        HippoxResult::system_error(format!("Failed to update task status: {}", task_id))
    }
}

/// Cancel a task
pub async fn cancel_task(task_id: &str) -> HippoxBoolResult {
    let mut pool = TASK_POOL.write().await;
    let success = pool.cancel_task(task_id);
    if success {
        HippoxResult::ok(true)
    } else {
        HippoxResult::system_error(format!("Failed to cancel task: {}", task_id))
    }
}

/// Pause a task
pub async fn pause_task(task_id: &str) -> HippoxBoolResult {
    let mut pool = TASK_POOL.write().await;
    let success = pool.pause_task(task_id);
    if success {
        HippoxResult::ok(true)
    } else {
        HippoxResult::system_error(format!("Failed to pause task: {}", task_id))
    }
}

/// Resume a paused task
pub async fn resume_task(task_id: &str) -> HippoxBoolResult {
    let result = {
        let mut pool = TASK_POOL.write().await;
        pool.resume_task(task_id)
    };
    if result {
        let pool = TASK_POOL.read().await;
        if let Some(task) = pool.get_task(task_id) {
            if let Some(executable) = task.get_executable() {
                if let Some(callback) = executable.get_workflow_callback() {
                    callback.on_workflow_resumed(task_id, 0, 0).await;
                }
            }
        }
        HippoxResult::ok(true)
    } else {
        HippoxResult::system_error(format!("Failed to resume task: {}", task_id))
    }
}

/// Retry a failed task
pub async fn retry_task(task_id: &str) -> HippoxBoolResult {
    let mut pool = TASK_POOL.write().await;
    let success = pool.retry_task(task_id);
    if success {
        HippoxResult::ok(true)
    } else {
        HippoxResult::system_error(format!("Failed to retry task: {}", task_id))
    }
}

/// Get all tasks
pub async fn get_all_tasks(limit: Option<usize>) -> HippoxBatchResult {
    let pool = TASK_POOL.read().await;
    let tasks = pool.get_all_tasks(limit);
    // Convert tasks to string IDs or full representation?
    // Using task IDs as strings for batch result
    let task_ids: Vec<String> = tasks.into_iter().map(|t| t.id).collect();
    HippoxResult::ok(task_ids)
}

/// Get all tasks with full details
pub async fn get_all_tasks_detailed(limit: Option<usize>) -> HippoxResult<Vec<Task>> {
    let pool = TASK_POOL.read().await;
    let tasks = pool.get_all_tasks(limit);
    HippoxResult::ok(tasks)
}

/// Set maximum concurrent tasks
pub async fn set_max_concurrent(max: usize) -> HippoxVoidResult {
    let mut pool = TASK_POOL.write().await;
    pool.set_max_concurrent(max);
    HippoxResult::ok(())
}

/// Get running task count
pub async fn running_count() -> HippoxResult<usize> {
    let pool = TASK_POOL.read().await;
    HippoxResult::ok(pool.running_count())
}

/// Get pending task count
pub async fn pending_count() -> HippoxResult<usize> {
    let pool = TASK_POOL.read().await;
    HippoxResult::ok(pool.pending_count())
}

/// Shutdown the task pool
pub async fn shutdown_task_pool() -> HippoxVoidResult {
    let mut pool = TASK_POOL.write().await;
    pool.shutdown();
    HippoxResult::ok(())
}

/// Wait for a task to complete and return its result
///
/// This function blocks until the specified task completes, then returns
/// the final output (including token usage).
///
/// # Arguments
/// * `task_id` - The task ID returned from `create_task()` or `create_task_with_executable()`
///
/// # Returns
/// The final output of the task as a HippoxStringResult with token usage
///
/// # Example
/// ```
/// let task_id = tasks::create_task_with_executable(...).await?;
/// let result = tasks::wait_task(&task_id).await?;
/// println!("Result: {}", result);
/// ```
pub async fn wait_task(task_id: &str) -> HippoxStringResult {
    // Poll task status until terminal state
    loop {
        let status = match get_task_status(task_id).await {
            HippoxResult { data: Some(s), .. } => s,
            HippoxResult { error: Some(e), .. } => {
                return HippoxResult::system_error(e);
            }
            _ => {
                return HippoxResult::system_error(format!("Task not found: {}", task_id));
            }
        };
        match status {
            TaskStatus::Completed => {
                // Get the task and extract result
                match get_task(task_id).await {
                    HippoxResult {
                        data: Some(task), ..
                    } => {
                        let output = task.final_output.unwrap_or_default();
                        return HippoxResult::ok_with_tokens(
                            output,
                            task.input_token_count,
                            task.output_token_count,
                        );
                    }
                    HippoxResult { error: Some(e), .. } => {
                        return HippoxResult::system_error(format!(
                            "Task completed but data retrieval failed: {}",
                            e
                        ));
                    }
                    _ => {
                        return HippoxResult::system_error(format!(
                            "Task completed but data not found: {}",
                            task_id
                        ));
                    }
                }
            }
            TaskStatus::Failed => match get_task(task_id).await {
                HippoxResult {
                    data: Some(task), ..
                } => {
                    let error = task.error.unwrap_or_else(|| "Unknown error".to_string());
                    return HippoxResult::system_error(format!("Task failed: {}", error));
                }
                _ => {
                    return HippoxResult::system_error(format!("Task failed: {}", task_id));
                }
            },
            TaskStatus::Cancelled => {
                return HippoxResult::system_error(format!("Task was cancelled: {}", task_id));
            }
            TaskStatus::Timeout => {
                return HippoxResult::system_error(format!("Task timed out: {}", task_id));
            }
            TaskStatus::Pending | TaskStatus::Running | TaskStatus::Paused => {
                // Wait before polling again (with backoff)
                sleep(Duration::from_millis(100)).await;
                continue;
            }
        }
    }
}
