//! Internal task implementations for Hippox core

use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use crate::tasks::{ExecutableTask, TaskStateUpdater};
use crate::workflow::{WorkflowCallback, WorkflowExecutor};

/// Task for natural language processing
#[derive(Debug)]
pub(crate) struct NaturalLanguageTask {
    input: String,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
    skills_registry: String,
    instances_registry: String,
}

impl NaturalLanguageTask {
    pub(crate) fn new(
        input: String,
        workflow_executor: WorkflowExecutor,
        scheduler: SkillScheduler,
        skills_registry: String,
        instances_registry: String,
    ) -> Self {
        Self {
            input,
            workflow_executor,
            scheduler,
            skills_registry,
            instances_registry,
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
        let skills_registry = self.skills_registry.clone();
        let instances_registry = self.instances_registry.clone();
        let task_id = state_updater.task_id().to_string();
        let overall_start = Instant::now();
        Box::pin(async move {
            let mut executor_with_callback = workflow_executor.clone();
            if let Some(ref cb) = callback {
                executor_with_callback = executor_with_callback.with_callback(cb.clone());
            }
            executor_with_callback = executor_with_callback.with_task_id(task_id.clone());
            let result = executor_with_callback
                .execute(&scheduler, &input, &skills_registry, &instances_registry)
                .await;
            let total_duration = overall_start.elapsed().as_millis() as u64;
            let total_steps = 0;
            state_updater.update_workflow_complete(&result).await;
            if let Some(ref cb) = callback {
                cb.on_workflow_complete(&task_id, &result, total_duration, total_steps)
                    .await;
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

/// Task for SKILL.md file execution
#[derive(Debug)]
pub(crate) struct SkillMdTask {
    path: String,
    params: Option<HashMap<String, Value>>,
    workflow_executor: WorkflowExecutor,
    scheduler: SkillScheduler,
    skills_registry: String,
    instances_registry: String,
}

impl SkillMdTask {
    pub(crate) fn new(
        path: String,
        params: Option<HashMap<String, Value>>,
        workflow_executor: WorkflowExecutor,
        scheduler: SkillScheduler,
        skills_registry: String,
        instances_registry: String,
    ) -> Self {
        Self {
            path,
            params,
            workflow_executor,
            scheduler,
            skills_registry,
            instances_registry,
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
        let skills_registry = self.skills_registry.clone();
        let instances_registry = self.instances_registry.clone();
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
            let result = executor_with_callback
                .execute_skill_md(
                    &scheduler,
                    &skill_file,
                    params.as_ref(),
                    &skills_registry,
                    &instances_registry,
                )
                .await;
            let total_duration = overall_start.elapsed().as_millis() as u64;
            let total_steps = 0;
            state_updater.update_workflow_complete(&result).await;
            if let Some(ref cb) = callback {
                cb.on_workflow_complete(&task_id, &result, total_duration, total_steps)
                    .await;
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
