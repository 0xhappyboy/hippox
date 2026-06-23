//! Internal task implementations for Hippox core

use hippox_drivers::DriverCallback;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use crate::driver_scheduler::DriverScheduler;
use crate::pipeline::{Pipeline, SystemPipeline, WorkflowExecResult, needs_format_conversion};
use crate::t;
use crate::tasks::{ExecutableTask, TaskStateUpdater};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor};

#[derive(Debug)]
pub(crate) struct NaturalLanguageTask {
    input: String,
    workflow_executor: WorkflowExecutor,
    scheduler: DriverScheduler,
    workflow_callback: Option<Arc<dyn WorkflowCallback>>,
    driver_callback: Option<Arc<dyn DriverCallback>>,
    disabled_drivers: Option<Vec<String>>,
}

impl NaturalLanguageTask {
    pub(crate) fn new(
        input: String,
        workflow_executor: WorkflowExecutor,
        scheduler: DriverScheduler,
        workflow_callback: Option<Arc<dyn WorkflowCallback>>,
        driver_callback: Option<Arc<dyn DriverCallback>>,
        disabled_drivers: Option<Vec<&str>>,
    ) -> Self {
        Self {
            input,
            workflow_executor,
            scheduler,
            workflow_callback,
            driver_callback,
            disabled_drivers: disabled_drivers.map(|v| v.into_iter().map(String::from).collect()),
        }
    }

    pub(crate) fn disabled_drivers(&self) -> Option<&[String]> {
        self.disabled_drivers.as_deref()
    }
}

impl ExecutableTask for NaturalLanguageTask {
    fn execute(
        &self,
        state_updater: TaskStateUpdater,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let workflow_callback = self.workflow_callback.clone();
        let driver_callback = self.driver_callback.clone();
        let input = self.input.clone();
        let mut workflow_executor = self.workflow_executor.clone();
        let scheduler = self.scheduler.clone();
        let task_id = state_updater.task_id().to_string();
        let overall_start = Instant::now();
        let pipeline = SystemPipeline::new();
        let disabled_drivers = self.disabled_drivers.clone();

        Box::pin(async move {
            let intent_result = match pipeline.intent_analysis(&scheduler, &input, &task_id).await {
                Ok(result) => result,
                Err(e) => {
                    tracing::warn!("Intent analysis failed: {}, using raw input", e);
                    crate::pipeline::IntentAnalysisResult {
                        categories: vec![],
                        clean_intent: input.clone(),
                    }
                }
            };
            let clean_intent = &intent_result.clean_intent;
            let categories = &intent_result.categories;
            let mut executor_with_callback = workflow_executor.clone();
            if let Some(ref cb) = workflow_callback {
                executor_with_callback = executor_with_callback.with_workflow_callback(cb.clone());
            }
            if let Some(cb) = driver_callback {
                executor_with_callback = executor_with_callback.with_driver_callback(cb);
            }
            executor_with_callback = executor_with_callback.with_task_id(task_id.clone());
            let result = pipeline
                .execute_workflow(
                    &scheduler,
                    &executor_with_callback,
                    clean_intent,
                    categories,
                    disabled_drivers.as_deref(),
                )
                .await;
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
            let final_output = if needs_format_conversion(&input) {
                let format_result = pipeline
                    .response_formatting(&scheduler, &input, &raw_json, &task_id)
                    .await;
                format_result.final_output
            } else {
                display_output
            };
            match result {
                WorkflowExecutionResult::Completed(_)
                | WorkflowExecutionResult::CompletedWithRaw { .. } => {
                    state_updater.update_workflow_complete(&final_output).await;
                    if let Some(ref cb) = workflow_callback {
                        cb.on_workflow_complete(
                            &task_id,
                            &final_output,
                            total_duration,
                            total_steps,
                        )
                        .await;
                    }
                }
                WorkflowExecutionResult::Paused { .. } => {
                    info!("Task {} was paused", task_id);
                }
                WorkflowExecutionResult::Cancelled { .. } => {
                    info!("Task {} was cancelled", task_id);
                }
                WorkflowExecutionResult::Failed { error, .. } => {
                    state_updater.update_workflow_failed(&error).await;
                    if let Some(ref cb) = workflow_callback {
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

    fn get_workflow_callback(&self) -> Option<Arc<dyn WorkflowCallback>> {
        self.workflow_callback.clone()
    }

    fn get_driver_callback(&self) -> Option<Arc<dyn DriverCallback>> {
        self.driver_callback.clone()
    }
}
