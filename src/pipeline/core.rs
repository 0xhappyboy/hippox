use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    ClassificationResult, Pipeline, SkillScheduler, StageOneResult, StageTwoResult,
    WorkflowCallback, WorkflowExecutor, WorkflowMode,
    prompts::{build_classifier_prompt, build_format_conversion_prompt},
};

/// Default implementation of Pipeline trait
#[derive(Debug, Clone, Default)]
pub(crate) struct SystemPipeline;

impl SystemPipeline {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Pipeline for SystemPipeline {
    /// Stage Zero: Classify user intent into skill categories
    async fn stage_zero(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
    ) -> anyhow::Result<ClassificationResult> {
        let prompt = build_classifier_prompt(input);
        let response = scheduler.get_llm().generate(&prompt).await?;
        // Extract JSON from response
        let json_str = crate::workflow::WorkflowExecutor::extract_json(&response);
        match serde_json::from_str::<ClassificationResult>(&json_str) {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::warn!(
                    "Failed to parse classification result: {}, response: {}. Using empty categories.",
                    e,
                    response
                );
                Ok(ClassificationResult { categories: vec![] })
            }
        }
    }
    /// Stage One: Core workflow execution
    async fn stage_one(
        &self,
        mode: WorkflowMode,
        executor: &WorkflowExecutor,
        scheduler: &SkillScheduler,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> StageOneResult {
        // Execute workflow (format requirements are ignored at this stage)
        let result = executor.execute(scheduler, input).await;
        let json_output = match result {
            crate::workflow::WorkflowExecutionResult::Completed(output) => output,
            crate::workflow::WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => raw_json,
            crate::workflow::WorkflowExecutionResult::Paused { partial_output, .. } => {
                partial_output
            }
            crate::workflow::WorkflowExecutionResult::Cancelled { .. } => String::new(),
            crate::workflow::WorkflowExecutionResult::Failed { error, .. } => error,
        };
        StageOneResult {
            json_output,
            original_input: input.to_string(),
        }
    }
    /// Stage Two: Format conversion based on user's structure requirements
    async fn stage_two(
        &self,
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
                tracing::warn!(
                    "Stage Two conversion failed: {}, returning original JSON",
                    e
                );
                json_output.to_string()
            }
        };
        // Check if conversion actually changed anything
        let was_converted = final_output != json_output;
        StageTwoResult {
            final_output,
            was_converted,
        }
    }
}
