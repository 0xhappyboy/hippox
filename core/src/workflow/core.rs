//! Core WorkflowExecutor implementation

use hippox_atomic_skills::{Executor, SkillCallback};

use super::batch::execute_batch;
use super::chain::execute_chain;
use super::plan_and_execute::execute_plan_and_execute;
use super::react::execute_react;
use super::types::*;
use crate::prompts::{build_react_prompt, build_skill_md_prompt};
use crate::skill_scheduler::SkillScheduler;
use crate::{
    execute_batch_with_categories, execute_chain_with_categories,
    execute_plan_and_execute_with_categories, execute_react_with_categories,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct WorkflowExecutor {
    pub(crate) mode: WorkflowMode,
    pub(crate) executor: Executor,
    pub(crate) max_iterations: usize,
    pub(crate) task_id: Option<String>,
    pub(crate) callback: Option<Arc<dyn WorkflowCallback>>,
    pub(crate) skill_callback: Option<Arc<dyn SkillCallback>>,
}

impl WorkflowExecutor {
    pub fn new(mode: WorkflowMode) -> Self {
        Self {
            mode,
            executor: Executor::new(),
            max_iterations: 10,
            task_id: None,
            callback: None,
            skill_callback: None,
        }
    }

    pub fn with_skill_callback(mut self, skill_callback: Arc<dyn SkillCallback>) -> Self {
        self.skill_callback = Some(skill_callback);
        self
    }

    pub fn get_skill_callback(&self) -> Option<Arc<dyn SkillCallback>> {
        self.skill_callback.clone()
    }

    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn get_task_id(&self) -> Option<&str> {
        self.task_id.as_deref()
    }

    pub fn with_callback(mut self, callback: Arc<dyn WorkflowCallback>) -> Self {
        self.callback = Some(callback);
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

    pub fn get_callback(&self) -> &Option<Arc<dyn WorkflowCallback>> {
        &self.callback
    }

    pub async fn execute(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
    ) -> WorkflowExecutionResult {
        match self.mode {
            WorkflowMode::ReAct => execute_react(self, scheduler, input).await,
            WorkflowMode::Batch => execute_batch(self, scheduler, input).await,
            WorkflowMode::Chain => execute_chain(self, scheduler, input).await,
            WorkflowMode::PlanAndExecute => execute_plan_and_execute(self, scheduler, input).await,
        }
    }

    pub async fn execute_with_categories(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
        categories: &[String],
    ) -> WorkflowExecutionResult {
        match self.mode {
            WorkflowMode::ReAct => {
                execute_react_with_categories(self, scheduler, input, categories).await
            }
            WorkflowMode::Batch => {
                execute_batch_with_categories(self, scheduler, input, categories).await
            }
            WorkflowMode::Chain => {
                execute_chain_with_categories(self, scheduler, input, categories).await
            }
            WorkflowMode::PlanAndExecute => {
                execute_plan_and_execute_with_categories(self, scheduler, input, categories).await
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
