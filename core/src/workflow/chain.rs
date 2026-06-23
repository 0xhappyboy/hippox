//! Chain mode workflow execution
//!
//! This mode executes drivers sequentially in the order defined by the LLM.
//! Each step is independent and does not depend on previous step outputs.
//! Failures in one step do not prevent subsequent steps from executing.
//!
//! # Characteristics
//! - Drivers are executed one after another (sequential, not parallel)
//! - Each step has independent retry (3 attempts) and timeout (60s) protection
//! - Steps do not depend on each other's results (no variable passing)
//! - Best for: Ordered operations where order matters but results are independent
//!
//! # Execution Flow
//! 1. LLM generates a chain plan with ordered steps
//! 2. Each step is executed in sequence
//! 3. Step failures are recorded but execution continues
//! 4. All results are aggregated and returned
//!
//! # Note
//! Unlike PlanAndExecute, Chain mode does NOT support variable passing
//! between steps. Each step operates independently with only the user input
//! available as context.

use crate::prompts::build_chain_prompt;
use crate::t;
use crate::{DriverScheduler, TASK_STEP_SIGNAL_BUS, check_task_interruption};
use hippox_drivers::{DriverCall, DriverCallback, DriverContext, Executor};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::retry::*;
use super::types::*;
use super::utils::{VARIABLE_REGEX, format_step_results};

/// Parse the LLM response into a ChainPlan.
///
/// The expected response format is a JSON object with:
/// - `mode`: Must be "chain"
/// - `steps`: Array of step definitions, each with action, parameters, and optional output_as
///
/// # Arguments
/// * `response` - The raw LLM response string
///
/// # Returns
/// A ChainPlan containing the parsed steps
pub fn parse_chain_response(response: &str) -> anyhow::Result<ChainPlan> {
    let json_str = WorkflowExecutor::extract_json(response);
    let value: Value = serde_json::from_str(&json_str)?;

    #[derive(serde::Deserialize)]
    struct ChainStep {
        action: String,
        parameters: HashMap<String, Value>,
        output_as: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct ChainResponse {
        mode: String,
        steps: Vec<ChainStep>,
    }

    let chain: ChainResponse = serde_json::from_value(value)?;
    if chain.mode != "chain" {
        anyhow::bail!("Invalid chain mode: expected 'chain', got '{}'", chain.mode);
    }
    let steps = chain
        .steps
        .into_iter()
        .map(|s| ChainStepDef {
            action: s.action,
            parameters: s.parameters,
            output_as: s.output_as,
        })
        .collect();
    Ok(ChainPlan { steps })
}

/// Resolve variable placeholders in a value using context.
///
/// Supports `{{variable_name}}` syntax for variable substitution.
/// This is used to resolve parameters before driver execution.
///
/// # Arguments
/// * `value` - The value containing potential variable placeholders
/// * `context` - The context map containing variable values
///
/// # Returns
/// The resolved value with variables substituted
fn resolve_variables_deep(value: &Value, context: &HashMap<String, Value>) -> Value {
    if let Some(s) = value.as_str() {
        if s.contains("{{") && s.contains("}}") {
            let mut result = s.to_string();
            for cap in VARIABLE_REGEX.captures_iter(s) {
                let var_name = &cap[1];
                if let Some(val) = context.get(var_name) {
                    let replacement = if let Some(num) = val.as_f64() {
                        num.to_string()
                    } else if let Some(s) = val.as_str() {
                        s.to_string()
                    } else {
                        val.to_string()
                    };
                    result = result.replace(&format!("{{{{{}}}}}", var_name), &replacement);
                }
            }
            return Value::String(result);
        }
        return Value::String(s.to_string());
    }
    if let Some(s) = value.as_str() {
        if s.starts_with("{{") && s.ends_with("}}") {
            let var_name = &s[2..s.len() - 2];
            if let Some(val) = context.get(var_name) {
                return val.clone();
            }
            return Value::String(s.to_string());
        }
        return Value::String(s.to_string());
    }
    if let Some(obj) = value.as_object() {
        let mut new_obj = serde_json::Map::new();
        for (k, v) in obj {
            new_obj.insert(k.clone(), resolve_variables_deep(v, context));
        }
        return Value::Object(new_obj);
    }
    if let Some(arr) = value.as_array() {
        let new_arr: Vec<Value> = arr
            .iter()
            .map(|v| resolve_variables_deep(v, context))
            .collect();
        return Value::Array(new_arr);
    }
    value.clone()
}

/// Execute a single chain step with retry and timeout protection.
///
/// This function handles the complete lifecycle of a single chain step:
/// - Executes the driver with timeout protection
/// - Automatically retries on failure or timeout (up to max_retries)
/// - Triggers appropriate callbacks for each attempt
/// - Returns Ok(output) on success, Err(error) after all retries exhausted
///
/// # Arguments
/// * `executor` - The driver executor
/// * `call` - The driver call to execute
/// * `step_name` - Name of the driver (for logging and callbacks)
/// * `idx` - Index of this step in the chain
/// * `task_id` - Optional task ID for task tracking
/// * `workflow_callback` - Optional workflow callback for progress notifications
/// * `driver_callback_arc` - Optional driver callback for driver-level events
/// * `max_retries` - Maximum number of retry attempts
/// * `timeout_secs` - Timeout in seconds for each execution attempt
///
/// # Returns
/// Ok(String) on success, Err(anyhow::Error) after all retries are exhausted
async fn execute_chain_step_with_retry(
    executor: &Executor,
    call: DriverCall,
    step_name: String,
    idx: usize,
    task_id: Option<String>,
    workflow_callback: &Option<Arc<dyn WorkflowCallback>>,
    driver_callback_arc: Option<Arc<dyn DriverCallback>>,
    max_retries: usize,
    timeout_secs: u64,
) -> anyhow::Result<String> {
    let mut retry_context = RetryContext::new(max_retries, DEFAULT_MAX_CONSECUTIVE_FAILURES);
    let mut last_error = None;
    loop {
        let driver_context = DriverContext {
            task_id: task_id.clone(),
            driver_index: Some(idx),
            driver_name: Some(step_name.clone()),
            extra: HashMap::new(),
            signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
        };
        let result = execute_driver_with_timeout(
            executor,
            &call,
            driver_callback_arc.clone(),
            Some(&driver_context),
            timeout_secs,
        )
        .await;
        match result {
            DriverExecutionResult::Success(output) => {
                return Ok(output);
            }
            DriverExecutionResult::Timeout(ref error_msg)
            | DriverExecutionResult::Failure(ref error_msg) => {
                let is_timeout = result.is_timeout();

                if let Some(cb) = workflow_callback {
                    if let Some(ref tid) = task_id {
                        if is_timeout {
                            cb.on_step_timeout(tid, &step_name, idx, error_msg, 0).await;
                        } else {
                            cb.on_step_failure(tid, &step_name, idx, error_msg, 0).await;
                        }
                    }
                }
                last_error = Some(error_msg.clone());
                if retry_context.can_retry(&step_name) {
                    continue;
                } else {
                    return Err(anyhow::anyhow!(
                        "Driver '{}' failed after {} retries: {}",
                        step_name,
                        max_retries,
                        last_error.unwrap_or_default()
                    ));
                }
            }
        }
    }
}

/// Execute a chain workflow with category filtering.
///
/// This is the main entry point for chain mode execution. It:
/// 1. Generates a chain plan using LLM with filtered drivers
/// 2. Executes each step sequentially
/// 3. Records all results (success and failure)
/// 4. Continues execution even if individual steps fail
///
/// # Arguments
/// * `executor` - The workflow executor
/// * `scheduler` - The driver scheduler for LLM interactions
/// * `input` - User input text
/// * `categories` - Driver categories to filter by
///
/// # Returns
/// A WorkflowExecutionResult containing all step results
pub async fn execute_chain_with_categories(
    executor: &WorkflowExecutor,
    scheduler: &DriverScheduler,
    input: &str,
    categories: &[String],
    disabled_drivers: Option<&[String]>,
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    let filtered_drivers =
        crate::prompts::generate_drivers_registry_by_categories(categories, disabled_drivers);
    let chain_prompt = crate::prompts::build_chain_prompt_with_categories(&filtered_drivers, input);

    let llm_response = match scheduler
        .generate_with_task(&chain_prompt, &task_id.clone().unwrap())
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return WorkflowExecutionResult::Failed {
                error: format!("{}: {}", t!("error.llm_error"), e),
                completed_steps: 0,
            };
        }
    };

    let chain = match parse_chain_response(&llm_response) {
        Ok(chain) => chain,
        Err(e) => {
            return WorkflowExecutionResult::Failed {
                error: format!("Failed to parse chain: {}", e),
                completed_steps: 0,
            };
        }
    };

    let mut context = HashMap::new();
    context.insert("user_input".to_string(), Value::String(input.to_string()));
    let mut results = Vec::new();

    let driver_callback_arc: Option<Arc<dyn DriverCallback>> = executor.get_driver_callback();
    let timeout_secs = get_timeout_secs(executor);
    let max_retries = DEFAULT_MAX_RETRIES_PER_SKILL;

    for (idx, step) in chain.steps.iter().enumerate() {
        // Check interruption before each step
        if let Err(result) = check_task_interruption(
            task_id.as_deref(),
            executor.get_workflow_callback(),
            idx,
            &step.action,
            None,
        )
        .await
        {
            return result;
        }

        let step_name = step.action.clone();
        let step_start = Instant::now();

        // Resolve parameters
        let mut resolved_params = HashMap::new();
        for (key, value) in &step.parameters {
            let resolved = resolve_variables_deep(value, &context);
            resolved_params.insert(key.clone(), resolved);
        }
        let call = DriverCall {
            action: step.action.clone(),
            parameters: resolved_params,
        };
        // Execute with retry
        match execute_chain_step_with_retry(
            executor.get_executor(),
            call,
            step_name.clone(),
            idx,
            task_id.clone(),
            executor.get_workflow_callback(),
            driver_callback_arc.clone(),
            max_retries,
            timeout_secs,
        )
        .await
        {
            Ok(output) => {
                // Save output to context for variable passing
                if let Some(output_as) = &step.output_as {
                    if let Ok(num) = output.parse::<f64>() {
                        context.insert(output_as.clone(), json!(num));
                    } else {
                        context.insert(output_as.clone(), Value::String(output.clone()));
                    }
                }
                let duration = step_start.elapsed().as_millis() as u64;
                if let Some(cb) = executor.get_workflow_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_success(tid, &step_name, idx, &output, duration)
                            .await;
                    }
                }
                results.push(StepResult {
                    driver: step.action.clone(),
                    parameters: step.parameters.clone(),
                    output: output.clone(),
                    status: ExecutionStatus::Success,
                });
            }
            Err(e) => {
                let error_msg = e.to_string();
                results.push(StepResult {
                    driver: step.action.clone(),
                    parameters: step.parameters.clone(),
                    output: error_msg.clone(),
                    status: ExecutionStatus::Failure,
                });
            }
        }
    }
    let final_display = format_step_results(&results);
    let raw_json = serde_json::json!({
        "mode": "chain",
        "steps": results.iter().map(|r| {
            serde_json::json!({
                "driver": r.driver,
                "output": r.output,
                "status": match r.status {
                    ExecutionStatus::Success => "success",
                    ExecutionStatus::Failure => "failure",
                }
            })
        }).collect::<Vec<_>>()
    })
    .to_string();
    WorkflowExecutionResult::CompletedWithRaw {
        display: final_display,
        raw_json,
    }
}
