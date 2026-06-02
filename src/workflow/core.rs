//! Core WorkflowExecutor implementation

use super::batch::execute_batch;
use super::chain::execute_chain;
use super::plan_and_execute::execute_plan_and_execute;
use super::prompt;
use super::react::execute_react;
use super::types::*;
use crate::executors::Executor;
use crate::skill_scheduler::SkillScheduler;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct WorkflowExecutor {
    pub(crate) mode: WorkflowMode,
    pub(crate) executor: Executor,
    pub(crate) max_iterations: usize,
    pub(crate) callback: Option<Arc<dyn WorkflowCallback>>,
    pub(crate) task_id: Option<String>,
}

impl WorkflowExecutor {
    pub fn new(mode: WorkflowMode) -> Self {
        Self {
            mode,
            executor: Executor::new(),
            max_iterations: 10,
            callback: None,
            task_id: None,
        }
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
        skills_registry: &str,
        instances_registry: &str,
    ) -> WorkflowExecutionResult {
        match self.mode {
            WorkflowMode::ReAct => {
                execute_react(self, scheduler, input, skills_registry, instances_registry).await
            }
            WorkflowMode::Batch => {
                execute_batch(self, scheduler, input, skills_registry, instances_registry).await
            }
            WorkflowMode::Chain => {
                execute_chain(self, scheduler, input, skills_registry, instances_registry).await
            }
            WorkflowMode::PlanAndExecute => {
                execute_plan_and_execute(
                    self,
                    scheduler,
                    input,
                    skills_registry,
                    instances_registry,
                )
                .await
            }
        }
    }

    pub async fn execute_skill_md(
        &self,
        scheduler: &SkillScheduler,
        skill_file: &crate::skill_loader::SkillFile,
        params: Option<&std::collections::HashMap<String, serde_json::Value>>,
        skills_registry: &str,
        instances_registry: &str,
    ) -> WorkflowExecutionResult {
        let mut instructions = skill_file.instructions.clone();
        if let Some(params) = params {
            for (key, value) in params {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                instructions = instructions.replace(&placeholder, &replacement);
            }
        }
        let enhanced_input = format!(
            "{}\n\n## Available Atomic Skills\n{}\n\n## Available Instances\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
            instructions, skills_registry, instances_registry
        );
        match self.mode {
            WorkflowMode::ReAct => {
                execute_react(
                    self,
                    scheduler,
                    &enhanced_input,
                    skills_registry,
                    instances_registry,
                )
                .await
            }
            WorkflowMode::Batch => {
                execute_batch(
                    self,
                    scheduler,
                    &enhanced_input,
                    skills_registry,
                    instances_registry,
                )
                .await
            }
            WorkflowMode::Chain => {
                execute_chain(
                    self,
                    scheduler,
                    &enhanced_input,
                    skills_registry,
                    instances_registry,
                )
                .await
            }
            WorkflowMode::PlanAndExecute => {
                execute_plan_and_execute(
                    self,
                    scheduler,
                    &enhanced_input,
                    skills_registry,
                    instances_registry,
                )
                .await
            }
        }
    }

    pub fn build_react_prompt(skills_registry: &str, instances_registry: &str) -> String {
        prompt::build_react_prompt(skills_registry, instances_registry)
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
