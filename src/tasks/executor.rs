//! Executor trait and state updater for tasks

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::workflow::WorkflowCallback;
use super::types::{TaskPool, TASK_POOL};

/// Task trait - each task must implement this to be executable
pub trait ExecutableTask: Send + Sync + Debug {
    /// Execute the task with the given state updater and external callback
    fn execute(
        &self,
        state_updater: TaskStateUpdater,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    /// Get the task type identifier
    fn task_type(&self) -> &str;

    /// Get the input for the task
    fn input(&self) -> &str;
}

/// Task state updater - updates internal task pool state
#[derive(Clone)]
pub struct TaskStateUpdater {
    task_id: String,
    task_pool: Arc<RwLock<TaskPool>>,
}

impl TaskStateUpdater {
    pub fn new(task_id: String, task_pool: Arc<RwLock<TaskPool>>) -> Self {
        Self { task_id, task_pool }
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Update step start in internal state
    pub async fn update_step_start(&self, step_name: &str, step_index: usize) {
        let mut pool = self.task_pool.write().await;
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

    /// Update step success in internal state
    pub async fn update_step_success(&self, step_name: &str, step_index: usize, output: &str) {
        let mut pool = self.task_pool.write().await;
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

    /// Update step failure in internal state
    pub async fn update_step_failure(&self, step_name: &str, step_index: usize, error: &str) {
        let mut pool = self.task_pool.write().await;
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

    /// Update workflow completion in internal state
    pub async fn update_workflow_complete(&self, final_output: &str) {
        let mut pool = self.task_pool.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            task.completed(final_output.to_string());
        }
    }

    /// Update workflow failure in internal state
    pub async fn update_workflow_failed(&self, error: &str) {
        let mut pool = self.task_pool.write().await;
        if let Some(task) = pool.get_task_mut(&self.task_id) {
            task.failed(error.to_string());
        }
    }
}