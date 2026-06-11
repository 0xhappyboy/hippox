use serde::{Deserialize, Serialize};

use crate::{SkillScheduler, WorkflowCallback, WorkflowExecutor, WorkflowMode};
use std::sync::Arc;

/// Classification result from Stage Zero
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassificationResult {
    pub categories: Vec<String>,
}

/// Result after Stage One execution
#[derive(Debug, Clone)]
pub struct StageOneResult {
    /// The standard JSON output from workflow
    pub json_output: String,
    /// Original user input
    pub original_input: String,
}

/// Result after Stage Two execution
#[derive(Debug, Clone)]
pub struct StageTwoResult {
    /// Final output after format conversion
    pub final_output: String,
    /// Whether conversion was performed
    pub was_converted: bool,
}

/// Pipeline trait - defines the three stages of execution
#[async_trait::async_trait]
pub trait Pipeline: Send + Sync {
    /// Stage Zero: Classify user intent into skill categories
    async fn stage_zero(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
    ) -> anyhow::Result<ClassificationResult>;

    /// Stage One: Core workflow execution
    async fn stage_one(
        &self,
        mode: WorkflowMode,
        executor: &WorkflowExecutor,
        scheduler: &SkillScheduler,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> StageOneResult;

    /// Stage Two: Format conversion based on user's structure requirements
    async fn stage_two(
        &self,
        scheduler: &SkillScheduler,
        original_input: &str,
        json_output: &str,
    ) -> StageTwoResult;
}
