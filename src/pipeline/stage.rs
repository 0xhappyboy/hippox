//! Pipeline stage definitions

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    SkillScheduler, WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor, WorkflowMode,
    prompts::build_format_conversion_prompt,
};

/// Pipeline execution stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    /// Stage One: Core workflow execution
    StageOne,
    /// Stage Two: Format conversion
    StageTwo,
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

/// Execute Stage One - core workflow execution
///
/// This stage runs the workflow and returns standard JSON output.
/// It does NOT consider user format requirements.
pub async fn execute_stage_one(
    mode: WorkflowMode,
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    callback: Option<Arc<dyn WorkflowCallback>>,
) -> StageOneResult {
    // Execute workflow (format requirements are ignored at this stage)
    let result = executor.execute(scheduler, input).await;
    let json_output = match result {
        WorkflowExecutionResult::Completed(output) => output,
        WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => raw_json,
        WorkflowExecutionResult::Paused { partial_output, .. } => partial_output,
        WorkflowExecutionResult::Cancelled { .. } => String::new(),
        WorkflowExecutionResult::Failed { error, .. } => error,
    };
    StageOneResult {
        json_output,
        original_input: input.to_string(),
    }
}

/// Execute Stage Two - format conversion based on user's structure requirements
///
/// This stage takes the Stage One JSON output and converts it to
/// the format requested by the user (if any).
pub async fn execute_stage_two(
    scheduler: &SkillScheduler,
    original_input: &str,
    json_output: &str,
) -> StageTwoResult {
    // If JSON output is empty, just return it
    if json_output.is_empty() {
        return StageTwoResult {
            final_output: json_output.to_string(),
            was_converted: false,
        };
    }
    // Build conversion prompt
    let prompt = build_format_conversion_prompt(original_input, json_output);
    // Call LLM for conversion
    let final_output = match scheduler.get_llm().generate(&prompt).await {
        Ok(resp) => resp,
        Err(e) => {
            // On error, return original JSON
            json_output.to_string()
        }
    };
    // Check if conversion actually changed anything (clone to avoid move)
    let was_converted = final_output != json_output;
    StageTwoResult {
        final_output,
        was_converted,
    }
}
