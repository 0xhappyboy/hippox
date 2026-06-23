//! Core WorkflowExecutor implementation

use hippox_drivers::{DriverCallback, Executor};

use super::types::*;
use crate::driver_scheduler::DriverScheduler;
use crate::prompts::{build_driver_md_prompt, build_react_prompt};
use crate::{
    execute_batch_with_categories, execute_chain_with_categories,
    execute_plan_and_execute_with_categories, execute_react_with_categories,
};
use std::sync::Arc;

const MAX_NUMBER_OF_REACT: usize = 10;

#[derive(Debug, Clone)]
pub(crate) struct WorkflowExecutor {
    pub(crate) mode: WorkflowMode,
    pub(crate) executor: Executor,
    pub(crate) max_iterations: usize,
    pub(crate) task_id: Option<String>,
    pub(crate) workflow_callback: Option<Arc<dyn WorkflowCallback>>,
    pub(crate) driver_callback: Option<Arc<dyn DriverCallback>>,
}

impl WorkflowExecutor {
    pub fn new(mode: WorkflowMode) -> Self {
        Self {
            mode,
            executor: Executor::new(),
            max_iterations: MAX_NUMBER_OF_REACT,
            task_id: None,
            workflow_callback: None,
            driver_callback: None,
        }
    }

    pub fn with_driver_callback(mut self, driver_callback: Arc<dyn DriverCallback>) -> Self {
        self.driver_callback = Some(driver_callback);
        self
    }

    pub fn get_driver_callback(&self) -> Option<Arc<dyn DriverCallback>> {
        self.driver_callback.clone()
    }

    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn get_task_id(&self) -> Option<&str> {
        self.task_id.as_deref()
    }

    pub fn with_workflow_callback(mut self, callback: Arc<dyn WorkflowCallback>) -> Self {
        self.workflow_callback = Some(callback);
        self
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn get_mode(&self) -> WorkflowMode {
        self.mode
    }

    pub fn get_executor(&self) -> &Executor {
        &self.executor
    }

    pub fn get_workflow_callback(&self) -> &Option<Arc<dyn WorkflowCallback>> {
        &self.workflow_callback
    }

    pub async fn execute_with_categories(
        &self,
        scheduler: &DriverScheduler,
        input: &str,
        categories: &[String],
        disabled_drivers: Option<&[String]>,
    ) -> WorkflowExecutionResult {
        match self.mode {
            WorkflowMode::ReAct => {
                execute_react_with_categories(self, scheduler, input, categories, disabled_drivers)
                    .await
            }
            WorkflowMode::Batch => {
                execute_batch_with_categories(self, scheduler, input, categories, disabled_drivers)
                    .await
            }
            WorkflowMode::Chain => {
                execute_chain_with_categories(self, scheduler, input, categories, disabled_drivers)
                    .await
            }
            WorkflowMode::PlanAndExecute => {
                execute_plan_and_execute_with_categories(
                    self,
                    scheduler,
                    input,
                    categories,
                    disabled_drivers,
                )
                .await
            }
        }
    }

    pub fn extract_json(text: &str) -> String {
        if let Some(start) = text.find("```json") {
            let after_start = &text[start + 7..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        if let Some(start) = text.find("```") {
            let after_start = &text[start + 3..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                return text[start..=end].to_string();
            }
        }
        text.to_string()
    }
}
