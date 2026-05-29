//! ReAct mode workflow execution

use crate::executors::SkillCall;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use langhub::types::ChatMessage;
use serde_json::Value;
use std::collections::HashMap;

use super::batch::execute_batch_plan;
use super::core::WorkflowExecutor;
use super::types::*;

/// Parse ReAct response from LLM
pub fn parse_react_response(response: &str) -> anyhow::Result<ReactInstruction> {
    let json_str = WorkflowExecutor::extract_json(response);
    let value: Value = serde_json::from_str(&json_str)?;
    if let Some(message) = value.get("message").and_then(|v| v.as_str()) {
        if value.get("action").and_then(|v| v.as_str()) == Some("done") {
            return Ok(ReactInstruction::Done(message.to_string()));
        }
    }
    if let Some(mode) = value.get("mode").and_then(|v| v.as_str()) {
        if mode == "batch" {
            if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
                let mut skill_calls = Vec::new();
                for step in steps {
                    let call: SkillCall = serde_json::from_value(step.clone())?;
                    skill_calls.push(call);
                }
                return Ok(ReactInstruction::Batch(skill_calls));
            }
        }
    }
    if let Ok(call) = serde_json::from_value(value) {
        return Ok(ReactInstruction::Single(call));
    }
    anyhow::bail!("Unable to parse LLM response: {}", response)
}

/// Execute ReAct mode workflow
pub async fn execute_react(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    skills_registry: &str,
    instances_registry: &str,
    is_first_message: bool,
) -> String {
    let input_trimmed = input.trim();
    if input_trimmed == "clear" {
        return t!("app.conversation_cleared").to_string();
    }
    if input_trimmed == "exit" || input_trimmed == "quit" {
        return "goodbye".to_string();
    }
    if input_trimmed.is_empty() {
        return String::new();
    }
    let mut step_results: Vec<StepResult> = Vec::new();
    let mut final_response = None;
    let mut iteration = 0;
    let mut messages: Vec<ChatMessage> = Vec::new();
    let system_prompt = WorkflowExecutor::build_react_prompt(skills_registry, instances_registry);
    messages.push(ChatMessage::system(&system_prompt));
    messages.push(ChatMessage::user(input_trimmed));
    while iteration < executor.max_iterations {
        iteration += 1;
        let llm_response = match scheduler.get_llm().chat(messages.clone()).await {
            Ok(resp) => resp,
            Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
        };
        messages.push(ChatMessage::assistant(&llm_response));
        let instruction = match parse_react_response(&llm_response) {
            Ok(instr) => instr,
            Err(_) => return llm_response,
        };
        match instruction {
            ReactInstruction::Done(message) => {
                final_response = Some(message);
                break;
            }
            ReactInstruction::Single(call) => {
                let step_index = step_results.len();
                let step_name = call.action.clone();
                if let Some(cb) = executor.get_callback() {
                    cb.on_step_start(&step_name, step_index).await;
                }
                match executor.get_executor().execute(&call).await {
                    Ok(output) => {
                        if let Some(cb) = executor.get_callback() {
                            cb.on_step_success(&step_name, step_index, &output).await;
                        }
                        step_results.push(StepResult {
                            skill: call.action.clone(),
                            parameters: call.parameters.clone(),
                            output: output.clone(),
                            status: ExecutionStatus::Success,
                        });
                        messages.push(ChatMessage::user(&format!(
                            "Skill '{}' executed. Result: {}",
                            call.action, output
                        )));
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        if let Some(cb) = executor.get_callback() {
                            cb.on_step_failure(&step_name, step_index, &error_msg).await;
                        }
                        step_results.push(StepResult {
                            skill: call.action.clone(),
                            parameters: call.parameters.clone(),
                            output: error_msg.clone(),
                            status: ExecutionStatus::Failure,
                        });
                        final_response = Some(format!(
                            "{} '{}': {}",
                            t!("error.skill_failed"),
                            call.action,
                            error_msg
                        ));
                        break;
                    }
                }
            }
            ReactInstruction::Batch(steps) => {
                let results = execute_batch_plan(executor, &steps).await;
                let mut batch_output = String::new();
                for (i, result) in results.iter().enumerate() {
                    step_results.push(result.clone());
                    batch_output.push_str(&format!("Step {}: {}\n", i + 1, result.output));
                }
                messages.push(ChatMessage::user(&format!(
                    "Batch execution completed. Results:\n{}",
                    batch_output
                )));
                let summary = format_step_results(executor, &step_results);
                final_response = Some(summary);
                break;
            }
        }
    }
    if iteration >= executor.max_iterations {
        final_response = Some(t!("error.max_iterations_reached").to_string());
    }
    let final_response = final_response.unwrap_or_else(|| {
        if step_results.is_empty() {
            t!("skill.no_actions_executed").to_string()
        } else {
            format_step_results(executor, &step_results)
        }
    });
    if let Some(cb) = executor.get_callback() {
        let has_failure = step_results
            .iter()
            .any(|r| r.status == ExecutionStatus::Failure)
            || final_response.starts_with("Error:")
            || final_response.contains("failed");

        if has_failure {
            cb.on_workflow_failed(&final_response).await;
        } else {
            cb.on_workflow_complete(&final_response).await;
        }
    }
    final_response
}

/// Format step results
fn format_step_results(executor: &WorkflowExecutor, results: &[StepResult]) -> String {
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
