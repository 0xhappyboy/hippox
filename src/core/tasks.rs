//! Internal task implementations for Hippox core

use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use crate::pipeline::{SystemPipeline, needs_format_conversion};
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::tasks::{ExecutableTask, TaskStateUpdater};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor};
use crate::{Pipeline, t};

#[derive(Debug)]
pub(crate) struct NaturalLanguageTask {
    input: String,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
    categories: Option<Vec<String>>,
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
            categories: None,
        }
    }

    pub(crate) fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = Some(categories);
        self
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
        let categories = self.categories.clone();
        let overall_start = Instant::now();
        let pipeline = SystemPipeline::new();
        Box::pin(async move {
            let mut executor_with_callback = workflow_executor.clone();
            if let Some(ref cb) = callback {
                executor_with_callback = executor_with_callback.with_callback(cb.clone());
            }
            executor_with_callback = executor_with_callback.with_task_id(task_id.clone());
            let result = if let Some(cats) = &categories {
                executor_with_callback
                    .execute_with_categories(&scheduler, &input, cats)
                    .await
            } else {
                executor_with_callback.execute(&scheduler, &input).await
            };
            let total_duration = overall_start.elapsed().as_millis() as u64;
            let total_steps = 0;
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
            // Stage Two: Format conversion
            let final_output = if !raw_json.is_empty() && needs_format_conversion(&input) {
                let stage_two_result = pipeline.stage_two(&scheduler, &input, &raw_json).await;
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
