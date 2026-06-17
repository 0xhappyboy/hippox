use serde::{Deserialize, Serialize};

use crate::{SkillScheduler, WorkflowCallback, WorkflowExecutor, WorkflowMode};
use std::sync::Arc;

/// Intent analysis result from Step 1
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntentAnalysisResult {
    pub categories: Vec<String>,
    pub clean_intent: String,
}

/// Workflow execution result from Step 2  
#[derive(Debug, Clone)]
pub struct WorkflowExecResult {
    /// The standard JSON output from workflow
    pub json_output: String,
    /// Original user input
    pub original_input: String,
}

/// Format result from Step 3
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// Final output after format conversion
    pub final_output: String,
    /// Whether conversion was performed
    pub was_converted: bool,
}

/// Pipeline trait - defines the three steps of execution
#[async_trait::async_trait]
pub trait Pipeline: Send + Sync {
    /// Step 1: Analyze user intent into skill categories
    async fn intent_analysis(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
        task_id: &str,
    ) -> anyhow::Result<IntentAnalysisResult>;

    /// Step 2: Core workflow execution
    async fn workflow_execution(
        &self,
        mode: WorkflowMode,
        executor: &WorkflowExecutor,
        scheduler: &SkillScheduler,
        input: &str,
    ) -> WorkflowExecResult;

    /// Step 3: Format conversion based on user's structure requirements
    async fn response_formatting(
        &self,
        scheduler: &SkillScheduler,
        original_input: &str,
        json_output: &str,
        task_id: &str,
    ) -> FormatResult;
}
