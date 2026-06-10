//! Internal task implementations for Hippox core

use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use crate::pipeline::needs_format_conversion;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::{execute_stage_two, t};
use crate::tasks::{ExecutableTask, TaskStateUpdater};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor};

#[derive(Debug)]
pub(crate) struct NaturalLanguageTask {
    input: String,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
}

impl NaturalLanguageTask {
    pub(crate) fn new(
        input: String,
        workflow_executor: WorkflowExecutor,
        scheduler: SkillScheduler,
    ) -> Self {
        Self {
            input,
            workflow_executor,
            scheduler,
        }
    }
}

impl ExecutableTask for NaturalLanguageTask {
    fn execute(
        &self,
        state_updater: TaskStateUpdater,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let input = self.input.clone();
        let workflow_executor = self.workflow_executor.clone();
        let scheduler = self.scheduler.clone();
        let task_id = state_updater.task_id().to_string();
        let overall_start = Instant::now();

        Box::pin(async move {
            let mut executor_with_callback = workflow_executor.clone();
            if let Some(ref cb) = callback {
                executor_with_callback = executor_with_callback.with_callback(cb.clone());
            }
            executor_with_callback = executor_with_callback.with_task_id(task_id.clone());
            // Stage One: Core workflow execution
            let result = executor_with_callback.execute(&scheduler, &input).await;
            let total_duration = overall_start.elapsed().as_millis() as u64;
            let total_steps = 0;
            // Extract raw JSON from result (for stage two conversion)
            let (display_output, raw_json) = match &result {
                WorkflowExecutionResult::Completed(output) => (output.clone(), output.clone()),
                WorkflowExecutionResult::CompletedWithRaw { display, raw_json } => {
                    (display.clone(), raw_json.clone())
                }
                WorkflowExecutionResult::Paused { partial_output, .. } => {
                    (partial_output.clone(), String::new())
                }
                WorkflowExecutionResult::Cancelled { .. } => (String::new(), String::new()),
                WorkflowExecutionResult::Failed { error, .. } => (error.clone(), String::new()),
            };
            // Stage Two: Format conversion (only if user has format requirement)
            let final_output = if !raw_json.is_empty() && needs_format_conversion(&input) {
                let stage_two_result = execute_stage_two(&scheduler, &input, &raw_json).await;
                stage_two_result.final_output
            } else {
                display_output
            };
            match result {
                WorkflowExecutionResult::Completed(_)
                | WorkflowExecutionResult::CompletedWithRaw { .. } => {
                    state_updater.update_workflow_complete(&final_output).await;
                    if let Some(ref cb) = callback {
                        cb.on_workflow_complete(
                            &task_id,
                            &final_output,
                            total_duration,
                            total_steps,
                        )
                        .await;
                    }
                }
                WorkflowExecutionResult::Paused { partial_output, .. } => {
                    info!("Task {} was paused", task_id);
                    if !partial_output.is_empty() {
                        // Optionally save partial output
                    }
                }
                WorkflowExecutionResult::Cancelled { .. } => {
                    info!("Task {} was cancelled", task_id);
                }
                WorkflowExecutionResult::Failed { error, .. } => {
                    state_updater.update_workflow_failed(&error).await;
                    if let Some(ref cb) = callback {
                        cb.on_workflow_failed(&task_id, &error, total_duration, total_steps)
                            .await;
                    }
                }
            }
        })
    }

    fn task_type(&self) -> &str {
        "natural_language"
    }

    fn input(&self) -> &str {
        &self.input
    }
}

#[derive(Debug)]
pub(crate) struct SkillMdTask {
    path: String,
    params: Option<HashMap<String, Value>>,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
}

impl SkillMdTask {
    pub(crate) fn new(
        path: String,
        params: Option<HashMap<String, Value>>,
        workflow_executor: WorkflowExecutor,
        scheduler: SkillScheduler,
    ) -> Self {
        Self {
            path,
            params,
            workflow_executor,
            scheduler,
        }
    }
}

impl ExecutableTask for SkillMdTask {
    fn execute(
        &self,
        state_updater: TaskStateUpdater,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let path = self.path.clone();
        let params = self.params.clone();
        let workflow_executor = self.workflow_executor.clone();
        let scheduler = self.scheduler.clone();
        let task_id = state_updater.task_id().to_string();
        let overall_start = Instant::now();

        Box::pin(async move {
            let skill_file = match SkillLoader::load_from_path(&path) {
                Ok(Some(file)) => file,
                Ok(None) => {
                    let error_msg = format!("{}: {}", t!("error.skill_not_found"), path);
                    let total_duration = overall_start.elapsed().as_millis() as u64;
                    state_updater.update_workflow_failed(&error_msg).await;
                    if let Some(ref cb) = callback {
                        cb.on_workflow_failed(&task_id, &error_msg, total_duration, 0)
                            .await;
                    }
                    return;
                }
                Err(e) => {
                    let error_msg = format!("{}: {}", t!("error.load_skill_failed"), e);
                    let total_duration = overall_start.elapsed().as_millis() as u64;
                    state_updater.update_workflow_failed(&error_msg).await;
                    if let Some(ref cb) = callback {
                        cb.on_workflow_failed(&task_id, &error_msg, total_duration, 0)
                            .await;
                    }
                    return;
                }
            };

            let mut executor_with_callback = workflow_executor.clone();
            if let Some(ref cb) = callback {
                executor_with_callback = executor_with_callback.with_callback(cb.clone());
            }
            executor_with_callback = executor_with_callback.with_task_id(task_id.clone());

            // Stage One: Core workflow execution
            let result = executor_with_callback
                .execute_skill_md(&scheduler, &skill_file, params.as_ref())
                .await;

            let total_duration = overall_start.elapsed().as_millis() as u64;
            let total_steps = 0;

            // Extract raw JSON from result (for stage two conversion)
            let (display_output, raw_json) = match &result {
                WorkflowExecutionResult::Completed(output) => (output.clone(), output.clone()),
                WorkflowExecutionResult::CompletedWithRaw { display, raw_json } => {
                    (display.clone(), raw_json.clone())
                }
                WorkflowExecutionResult::Paused { partial_output, .. } => {
                    (partial_output.clone(), String::new())
                }
                WorkflowExecutionResult::Cancelled { .. } => (String::new(), String::new()),
                WorkflowExecutionResult::Failed { error, .. } => (error.clone(), String::new()),
            };
            // Stage Two: Format conversion (only if user has format requirement)
            let final_output = if !raw_json.is_empty() && needs_format_conversion(&path) {
                let stage_two_result = execute_stage_two(&scheduler, &path, &raw_json).await;
                stage_two_result.final_output
            } else {
                display_output
            };
            match result {
                WorkflowExecutionResult::Completed(_)
                | WorkflowExecutionResult::CompletedWithRaw { .. } => {
                    state_updater.update_workflow_complete(&final_output).await;
                    if let Some(ref cb) = callback {
                        cb.on_workflow_complete(
                            &task_id,
                            &final_output,
                            total_duration,
                            total_steps,
                        )
                        .await;
                    }
                }
                WorkflowExecutionResult::Paused { partial_output, .. } => {
                    info!("SKILL.md task {} was paused", task_id);
                    if !partial_output.is_empty() {
                        // Optionally save partial output
                    }
                }
                WorkflowExecutionResult::Cancelled { .. } => {
                    info!("SKILL.md task {} was cancelled", task_id);
                }
                WorkflowExecutionResult::Failed { error, .. } => {
                    state_updater.update_workflow_failed(&error).await;
                    if let Some(ref cb) = callback {
                        cb.on_workflow_failed(&task_id, &error, total_duration, total_steps)
                            .await;
                    }
                }
            }
        })
    }

    fn task_type(&self) -> &str {
        "skill_md"
    }

    fn input(&self) -> &str {
        &self.path
    }
}
