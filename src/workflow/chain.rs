//! Chain mode workflow execution

use crate::executors::{Executor, SkillCall};
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

use super::types::*;
use super::core::WorkflowExecutor;

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

/// Resolve variables deep
fn resolve_variables_deep(value: &Value, context: &HashMap<String, Value>) -> Value {
    if let Some(s) = value.as_str() {
        if s.contains("{{") && s.contains("}}") {
            let mut result = s.to_string();
            let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
            for cap in re.captures_iter(s) {
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

/// Format step results
fn format_step_results(results: &[StepResult]) -> String {
    if results.is_empty() {
        return t!("skill.no_steps_executed").to_string();
    }
    if results.len() == 1 {
        return results[0].output.clone();
    }
    let success_count = results
        .iter()
        .filter(|r| r.status == ExecutionStatus::Success)
        .count();
    let failure_count = results.len() - success_count;
    let mut output = format!(
        "{} (SUCCESS {} / FAILURE {}):\n\n",
        t!("skill.executed_steps", results.len()),
        success_count,
        failure_count
    );
    for (i, result) in results.iter().enumerate() {
        let marker = match result.status {
            ExecutionStatus::Success => "SUCCESS",
            ExecutionStatus::Failure => "FAILURE",
        };
        output.push_str(&format!("{} {}: {}\n", marker, i + 1, result.output));
    }
    output
}

/// Execute chain mode workflow
pub async fn execute_chain(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    skills_registry: &str,
    instances_registry: &str,
    is_first_message: bool,
) -> String {
    let chain_prompt = format!(
        r#"You are Hippox, a reliable AI runtime and skills orchestration engine with autonomous decision-making.

## CRITICAL: Variable Reference Format
When referencing a previous output, use this EXACT format:
{{{{variable_name}}}}

Example: if output_as is "step1", reference as {{{{step1}}}}

## Available Atomic Skills
{}

## Available Instances
{}

## Response Format
{{"mode": "chain", "steps": [
  {{"action": "calculator", "parameters": {{"expression": "5 * 3"}}, "output_as": "result1"}},
  {{"action": "calculator", "parameters": {{"expression": "{{{{result1}}}} + 10"}}, "output_as": "result2"}}
]}}

## User Input
{}

Respond with ONLY the JSON.
"#,
        skills_registry, instances_registry, input
    );

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

    for (idx, step) in chain.steps.iter().enumerate() {
        let step_name = step.action.clone();
        if let Some(cb) = executor.get_callback() {
            cb.on_step_start(&step_name, idx).await;
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
                    cb.on_step_success(&step_name, idx, &output).await;
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
                    cb.on_step_failure(&step_name, idx, &error_msg).await;
                }
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: error_msg.clone(),
                    status: ExecutionStatus::Failure,
                });
                if let Some(cb) = executor.get_callback() {
                    cb.on_workflow_failed(&error_msg).await;
                }
                return format!("Skill '{}' failed: {}", step.action, error_msg);
            }
        }
    }

    let final_output = format_step_results(&results);
    if let Some(cb) = executor.get_callback() {
        let has_failure = results.iter().any(|r| r.status == ExecutionStatus::Failure);
        if has_failure {
            cb.on_workflow_failed(&final_output).await;
        } else {
            cb.on_workflow_complete(&final_output).await;
        }
    }
    final_output
}