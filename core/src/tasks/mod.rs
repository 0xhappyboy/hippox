//! Task management module for Hippox core
//!
//! This module provides a GLOBAL static task pool (independent of Hippox instances)
//! with automatic background execution engine that starts at program load.

mod api;
mod engine;
mod executor;
mod types;

pub use api::*;
pub use executor::ExecutableTask;
pub use executor::TaskStateUpdater;
pub use types::*;

/// Get a TaskStateUpdater for a specific task ID
pub async fn get_state_updater(task_id: &str) -> Option<TaskStateUpdater> {
    let pool = TASK_POOL.read().await;
    if pool.has_task(task_id) {
        Some(TaskStateUpdater::new(
            task_id.to_string(),
            TASK_POOL.clone(),
        ))
    } else {
        None
    }
}
