use crate::{
    DriverScheduler, FormatResult, IntentAnalysisResult, IntentParseResult, Pipeline, WorkflowCallback, WorkflowExecResult, WorkflowExecutor,
    WorkflowMode,
    prompts::{build_format_conversion_prompt, build_intent_parser_prompt},
};
use async_trait::async_trait;
use std::sync::Arc;
/// Default implementation of Pipeline trait
#[derive(Debug, Clone, Default)]
pub(crate) struct SystemPipeline;
impl SystemPipeline {
    pub fn new() -> Self {
        Self
    }
    /// Internal: Parse raw input to extract clean intent, categories, and format
    ///
    /// `model` - Optional model id override forwarded to the LLM call.
    async fn parse_intent(&self, scheduler: &DriverScheduler, raw_input: &str, task_id: &str, model: Option<&str>) -> IntentParseResult {
        let prompt = build_intent_parser_prompt(raw_input);
        // Forward the caller-provided model id.
        let response = scheduler.generate_with_task(&prompt, task_id, model).await;
        match response {
            Ok(resp) => {
                let json_str = crate::workflow::WorkflowExecutor::extract_json(&resp);
                match serde_json::from_str::<IntentParseResult>(&json_str) {
                    Ok(result) => result,
                    Err(e) => IntentParseResult::fallback(raw_input),
                }
            }
            Err(e) => IntentParseResult::fallback(raw_input),
        }
    }
    /// Execute the workflow, forwarding the model id.
    ///
    /// `model` - Optional model id override forwarded to every LLM call.
    pub async fn execute_workflow(
        &self,
        scheduler: &DriverScheduler,
        executor: &WorkflowExecutor,
        clean_intent: &str,
        categories: &[String],
        disabled_drivers: Option<&[String]>,
        model: Option<&str>,
    ) -> crate::workflow::WorkflowExecutionResult {
        if categories.is_empty() {
            // Fallback chat for empty categories, forwarding the model id.
            match scheduler.fallback_chat(clean_intent, model).await {
                Ok(output) => crate::workflow::WorkflowExecutionResult::CompletedWithRaw {
                    display: output.clone(),
                    raw_json: serde_json::json!({
                        "mode": "chat",
                        "result": output
                    })
                    .to_string(),
                },
                Err(e) => crate::workflow::WorkflowExecutionResult::Failed { error: format!("Fallback chat failed: {}", e), completed_steps: 0 },
            }
        } else {
            executor.execute_with_categories(scheduler, clean_intent, categories, disabled_drivers, model).await
        }
    }
}
#[async_trait]
impl Pipeline for SystemPipeline {
    /// Step 1: Analyze user intent
    async fn intent_analysis(
        &self,
        scheduler: &DriverScheduler,
        raw_input: &str,
        task_id: &str,
        model: Option<&str>,
    ) -> anyhow::Result<IntentAnalysisResult> {
        let parsed = self.parse_intent(scheduler, raw_input, task_id, model).await;
        Ok(IntentAnalysisResult { categories: parsed.skill_categories, clean_intent: parsed.clean_intent })
    }
    /// Step 2: Core workflow execution
    async fn workflow_execution(
        &self,
        _mode: WorkflowMode,
        executor: &WorkflowExecutor,
        scheduler: &DriverScheduler,
        input: &str,
        disabled_drivers: Option<&[String]>,
        model: Option<&str>,
    ) -> WorkflowExecResult {
        let result = self.execute_workflow(scheduler, executor, input, &[], disabled_drivers, model).await;
        let json_output = match result {
            crate::workflow::WorkflowExecutionResult::Completed(output) => output,
            crate::workflow::WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => raw_json,
            crate::workflow::WorkflowExecutionResult::Paused { partial_output, .. } => partial_output,
            crate::workflow::WorkflowExecutionResult::Cancelled { .. } => String::new(),
            crate::workflow::WorkflowExecutionResult::Failed { error, .. } => error,
        };
        WorkflowExecResult { json_output, original_input: input.to_string() }
    }
    /// Step 3: Without format specification
    async fn response_formatting(
        &self,
        scheduler: &DriverScheduler,
        original_input: &str,
        json_output: &str,
        task_id: &str,
        model: Option<&str>,
    ) -> FormatResult {
        if json_output.is_empty() {
            return FormatResult { final_output: json_output.to_string(), was_converted: false };
        }
        let prompt = build_format_conversion_prompt(original_input, json_output);
        // Forward the caller-provided model id.
        let final_output = match scheduler.generate_with_task(&prompt, task_id, model).await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!("Response formatting failed: {}, returning original JSON", e);
                json_output.to_string()
            }
        };
        let was_converted = final_output != json_output;
        FormatResult { final_output, was_converted }
    }
}
