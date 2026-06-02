//! Background execution engine for task processing

use std::sync::Arc;
use std::thread;
use tokio::sync::RwLock;

use super::types::{TaskPool, TASK_NOTIFIER};
use super::executor::TaskStateUpdater;

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
            // Get the executable task and its callback
            let (executable, callback) = {
                let pool = task_pool.read().await;
                if let Some(task) = pool.get_task(&task_id) {
                    if let Some(executable) = task.get_executable() {
                        (Some(executable), task.callback.clone())
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            };
            if let Some(executable_task) = executable {
                let state_updater = TaskStateUpdater::new(task_id.clone(), task_pool.clone());
                executable_task.execute(state_updater, callback).await;
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