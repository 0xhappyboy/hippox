//! Background execution engine for task processing

use std::sync::Arc;
use std::thread;
use tokio::sync::RwLock;

use super::executor::TaskStateUpdater;
use super::types::{TASK_NOTIFIER, TaskPool, TaskStatus};

pub fn start_execution_engine(pool: Arc<RwLock<TaskPool>>) {
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
        {
            let pool = task_pool.read().await;
            if pool.is_shutdown() {
                break;
            }
        }
        let task_id = {
            let mut pool = task_pool.write().await;
            pool.next_task()
        };
        if let Some(task_id) = task_id {
            // Check if task is still valid to run
            {
                let pool = task_pool.read().await;
                if let Some(task) = pool.get_task(&task_id) {
                    // Skip only Paused tasks (not ready to run)
                    if task.status == TaskStatus::Paused {
                        continue;
                    }
                    // Terminal states - clean up
                    if task.status == TaskStatus::Cancelled
                        || task.status == TaskStatus::Completed
                        || task.status == TaskStatus::Failed
                        || task.status == TaskStatus::Timeout
                    {
                        let mut pool = task_pool.write().await;
                        pool.complete_task(&task_id);
                        continue;
                    }
                    // For Running and Pending, continue to execute
                } else {
                    let mut pool = task_pool.write().await;
                    pool.complete_task(&task_id);
                    continue;
                }
            }
            // Get the executable task
            let executable = {
                let pool = task_pool.read().await;
                if let Some(task) = pool.get_task(&task_id) {
                    task.get_executable()
                } else {
                    None
                }
            };
            if let Some(executable_task) = executable {
                let state_updater = TaskStateUpdater::new(task_id.clone(), task_pool.clone());
                executable_task.execute(state_updater).await;
            } else {
                let mut pool = task_pool.write().await;
                if let Some(task) = pool.get_task_mut(&task_id) {
                    task.failed("No executable associated with task".to_string());
                }
                pool.complete_task(&task_id);
            }
        } else {
            TASK_NOTIFIER.notified().await;
        }
    }
}
