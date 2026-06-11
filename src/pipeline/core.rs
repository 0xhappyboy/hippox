use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    FormatResult, IntentAnalysisResult, IntentParseResult, Pipeline, SkillScheduler,
    WorkflowCallback, WorkflowExecResult, WorkflowExecutor, WorkflowMode,
    prompts::{build_format_conversion_prompt, build_intent_parser_prompt},
};

/// Default implementation of Pipeline trait
#[derive(Debug, Clone, Default)]
pub(crate) struct SystemPipeline;

impl SystemPipeline {
    pub fn new() -> Self {
        Self
    }

    /// Internal: Parse raw input to extract clean intent, categories, and format
    async fn parse_intent(&self, scheduler: &SkillScheduler, raw_input: &str) -> IntentParseResult {
        let prompt = build_intent_parser_prompt(raw_input);
        match scheduler.get_llm().generate(&prompt).await {
            Ok(response) => {
                let json_str = crate::workflow::WorkflowExecutor::extract_json(&response);
                match serde_json::from_str::<IntentParseResult>(&json_str) {
                    Ok(result) => result,
                    Err(e) => IntentParseResult::fallback(raw_input),
                }
            }
            Err(e) => IntentParseResult::fallback(raw_input),
        }
    }
}

#[async_trait]
impl Pipeline for SystemPipeline {
    /// Step 1: Analyze user intent
    async fn intent_analysis(
        &self,
        scheduler: &SkillScheduler,
        raw_input: &str,
    ) -> anyhow::Result<IntentAnalysisResult> {
        let parsed = self.parse_intent(scheduler, raw_input).await;
        Ok(IntentAnalysisResult {
            categories: parsed.skill_categories,
            clean_intent: parsed.clean_intent,
        })
    }

    /// Step 2: Core workflow execution
    async fn workflow_execution(
        &self,
        mode: WorkflowMode,
        executor: &WorkflowExecutor,
        scheduler: &SkillScheduler,
        input: &str,
        callback: Option<Arc<dyn WorkflowCallback>>,
    ) -> WorkflowExecResult {
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
        WorkflowExecResult {
            json_output,
            original_input: input.to_string(),
        }
    }

    /// Step 3: Without format specification
    async fn response_formatting(
        &self,
        scheduler: &SkillScheduler,
        original_input: &str,
        json_output: &str,
    ) -> FormatResult {
        if json_output.is_empty() {
            return FormatResult {
                final_output: json_output.to_string(),
                was_converted: false,
            };
        }
        let prompt = build_format_conversion_prompt(original_input, json_output);
        let final_output = match scheduler.get_llm().generate(&prompt).await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!("Response formatting failed: {}, returning original JSON", e);
                json_output.to_string()
            }
        };
        let was_converted = final_output != json_output;
        FormatResult {
            final_output,
            was_converted,
        }
    }
}
