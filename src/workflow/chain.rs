//! Chain mode workflow execution

use crate::executors::{Executor, SkillCall};
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

use super::core::WorkflowExecutor;
use super::prompt;
use super::types::*;
use super::utils::{VARIABLE_REGEX, format_step_results};

/// Parse chain response from LLM
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

/// Resolve variables deep with cached regex
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

/// Execute chain mode workflow
pub async fn execute_chain(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    skills_registry: &str,
    instances_registry: &str,
) -> String {
    let chain_prompt = prompt::build_chain_prompt(skills_registry, instances_registry, input);
    let llm_response = match scheduler.get_llm().generate(&chain_prompt).await {
        Ok(resp) => resp,
        Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
    };
    let chain = match parse_chain_response(&llm_response) {
        Ok(chain) => chain,
        Err(e) => return format!("Failed to parse chain: {}", e),
    };
    let mut context = HashMap::new();
    context.insert("user_input".to_string(), Value::String(input.to_string()));
    let mut results = Vec::new();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    for (idx, step) in chain.steps.iter().enumerate() {
        let step_name = step.action.clone();
        if let Some(cb) = executor.get_callback() {
            if let Some(ref tid) = task_id {
                cb.on_step_start(tid, &step_name, idx).await;
            }
        }
        let mut resolved_params = HashMap::new();
        for (key, value) in &step.parameters {
            let resolved = resolve_variables_deep(value, &context);
            resolved_params.insert(key.clone(), resolved);
        }
        let call = SkillCall {
            action: step.action.clone(),
            parameters: resolved_params,
        };
        match executor.get_executor().execute(&call).await {
            Ok(output) => {
                if let Some(cb) = executor.get_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_success(tid, &step_name, idx, &output).await;
                    }
                }
                if let Some(output_as) = &step.output_as {
                    if let Ok(num) = output.parse::<f64>() {
                        context.insert(output_as.clone(), json!(num));
                    } else {
                        context.insert(output_as.clone(), Value::String(output.clone()));
                    }
                }
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: output.clone(),
                    status: ExecutionStatus::Success,
                });
            }
            Err(e) => {
                let error_msg = e.to_string();
                if let Some(cb) = executor.get_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_failure(tid, &step_name, idx, &error_msg).await;
                    }
                }
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: error_msg.clone(),
                    status: ExecutionStatus::Failure,
                });
                if let Some(cb) = executor.get_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_workflow_failed(tid, &error_msg).await;
                    }
                }
                return format!("Skill '{}' failed: {}", step.action, error_msg);
            }
        }
    }

    let final_output = format_step_results(&results);
    if let Some(cb) = executor.get_callback() {
        if let Some(ref tid) = task_id {
            let has_failure = results.iter().any(|r| r.status == ExecutionStatus::Failure);
            if has_failure {
                cb.on_workflow_failed(tid, &final_output).await;
            } else {
                cb.on_workflow_complete(tid, &final_output).await;
            }
        }
    }
    final_output
}
