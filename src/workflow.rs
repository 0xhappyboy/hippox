//! Workflow execution module for Hippox core
//!
//! This module provides different workflow execution modes:
//! - ReAct: Traditional think-act-observe loop
//! - Batch: Parallel execution of independent skills
//! - PlanAndExecute: One-time planning with dependency resolution
//! - Chain: Simple sequential execution with variable passing

use crate::executors::{Executor, SkillCall};
use crate::memory::ConversationMemory;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use async_trait::async_trait;
use futures::future::join_all;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Workflow execution mode enumeration
///
/// Defines the strategy for processing user requests and executing skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowMode {
    /// ReAct mode: Think → Act → Observe loop
    ///
    /// Each skill execution is followed by LLM decision for next step.
    /// Best for: Open-ended tasks, dynamic decision making, error recovery
    /// LLM calls: 1 per skill + 1 for final response
    ReAct,

    /// Batch mode: Execute multiple independent skills in parallel
    ///
    /// Skills must have no dependencies on each other's results.
    /// Best for: Independent operations, bulk processing
    /// LLM calls: 1 (generates batch plan)
    Batch,

    /// Chain mode: Sequential execution with variable passing
    ///
    /// Each skill's output can be passed as input to the next skill.
    /// Best for: Linear pipelines, data transformation chains
    /// LLM calls: 1 (generates chain)
    Chain,

    /// Plan-and-Execute mode: One-time planning with full workflow
    ///
    /// Supports conditionals, variable references, and error handling.
    /// Best for: Complex workflows, conditional logic, deterministic tasks
    /// LLM calls: 1 (generates plan) + optional for dynamic decisions
    PlanAndExecute,
}

impl Default for WorkflowMode {
    fn default() -> Self {
        WorkflowMode::ReAct
    }
}

impl std::fmt::Display for WorkflowMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowMode::ReAct => write!(f, "ReAct"),
            WorkflowMode::Batch => write!(f, "Batch"),
            WorkflowMode::Chain => write!(f, "Chain"),
            WorkflowMode::PlanAndExecute => write!(f, "PlanAndExecute"),
        }
    }
}

/// Workflow execution callback trait
///
/// Implement this trait to receive real-time updates about workflow execution.
/// This is useful for UI updates, logging, or progress reporting.
#[async_trait]
pub trait WorkflowCallback: Send + Sync + Debug {
    /// Called when a step (skill execution) starts
    async fn on_step_start(&self, step_name: &str, step_index: usize);
    /// Called when a step completes successfully
    async fn on_step_success(&self, step_name: &str, step_index: usize, output: &str);
    /// Called when a step fails
    async fn on_step_failure(&self, step_name: &str, step_index: usize, error: &str);
    /// Called when the entire workflow completes successfully
    async fn on_workflow_complete(&self, final_output: &str);
    /// Called when the workflow fails
    async fn on_workflow_failed(&self, error: &str);
}

/// Context variable for workflow execution
#[derive(Debug, Clone)]
pub struct Workflow {
    /// Variable store for passing data between steps
    variables: HashMap<String, Value>,
    /// Step results for debugging
    step_results: Vec<WorkflowStepResult>,
}

impl Workflow {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            step_results: Vec::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn add_step_result(&mut self, result: WorkflowStepResult) {
        self.step_results.push(result);
    }

    pub fn get_step_results(&self) -> &[WorkflowStepResult] {
        &self.step_results
    }
}

impl Default for Workflow {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single workflow step
#[derive(Debug, Clone)]
pub struct WorkflowStepResult {
    pub step_id: String,
    pub skill: String,
    pub input: HashMap<String, Value>,
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Workflow step definition for Plan-and-Execute mode
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WorkflowStep {
    /// Unique identifier for this step (for variable referencing)
    pub id: String,
    /// Skill to execute
    pub action: String,
    /// Parameters with potential variable references
    pub parameters: HashMap<String, ValueRef>,
    /// Condition for execution (optional)
    #[serde(default)]
    pub condition: Option<Condition>,
    /// Variable name to store output (optional)
    #[serde(default)]
    pub output_as: Option<String>,
    /// Error handler (optional)
    #[serde(default)]
    pub on_error: Option<ErrorHandler>,
}

/// Variable reference that can be a literal or reference to previous output
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum ValueRef {
    /// Literal JSON value
    Literal(Value),
    /// Reference to a variable: {"$ref": "variable_name"}
    Reference(Reference),
    /// Expression (future extension)
    Expression(Expression),
}

/// Reference to a variable or step output
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub path: String,
}

/// Expression for dynamic evaluation (placeholder for future)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Expression {
    #[serde(rename = "$expr")]
    pub expr: String,
}

/// Conditional execution predicate
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    /// Operation: eq, ne, gt, lt, contains, etc.
    pub op: String,
    /// Left operand
    pub left: ValueRef,
    /// Right operand
    pub right: ValueRef,
}

/// Error handling strategy
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ErrorHandler {
    /// Action: fail, skip, retry, fallback
    pub action: String,
    /// Fallback value or step
    #[serde(default)]
    pub fallback: Option<ValueRef>,
    /// Maximum retries
    #[serde(default)]
    pub max_retries: Option<u32>,
}

/// Complete workflow plan
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WorkflowPlan {
    pub name: Option<String>,
    pub steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
}

/// Response from LLM for Plan-and-Execute mode
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PlanInstruction {
    pub mode: String,
    pub plan: Option<WorkflowPlan>,
    pub message: Option<String>,
}

/// Workflow executor that handles different execution modes
#[derive(Debug, Clone)]
pub struct WorkflowExecutor {
    mode: WorkflowMode,
    executor: Executor,
    max_iterations: usize,
    callback: Option<Arc<dyn WorkflowCallback>>,
}

impl WorkflowExecutor {
    pub fn new(mode: WorkflowMode) -> Self {
        Self {
            mode,
            executor: Executor::new(),
            max_iterations: 10,
            callback: None,
        }
    }

    pub fn with_callback(mut self, callback: Arc<dyn WorkflowCallback>) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Deep recursive resolution of variable references
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
            // Processing the {{variable}} format
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
                new_obj.insert(k.clone(), Self::resolve_variables_deep(v, context));
            }
            return Value::Object(new_obj);
        }
        if let Some(arr) = value.as_array() {
            let new_arr: Vec<Value> = arr
                .iter()
                .map(|v| Self::resolve_variables_deep(v, context))
                .collect();
            return Value::Array(new_arr);
        }
        value.clone()
    }

    /// Execute a workflow plan with variable resolution and conditions
    async fn execute_workflow_plan(&self, plan: &WorkflowPlan) -> anyhow::Result<String> {
        let mut context = Workflow::new();
        for (key, value) in &plan.parameters {
            context.set_variable(key, value.clone());
        }
        let mut string_context = HashMap::new();
        for step in &plan.steps {
            // Check condition
            if let Some(condition) = &step.condition {
                if !self.evaluate_condition(condition, &context) {
                    info!("Step {} condition not met, skipping", step.id);
                    continue;
                }
            }
            let mut resolved_params = HashMap::new();
            for (key, value_ref) in &step.parameters {
                let resolved = self.resolve_value_ref(value_ref, &context);
                let final_resolved = Self::resolve_variables_deep(&resolved, &string_context);
                resolved_params.insert(key.clone(), final_resolved);
            }
            let result = self
                .execute_step_with_retry(&step.action, resolved_params, step)
                .await;
            match result {
                Ok(output) => {
                    if let Some(output_as) = &step.output_as {
                        context.set_variable(output_as, Value::String(output.clone()));
                        string_context.insert(output_as.clone(), Value::String(output.clone()));
                    }
                    context.add_step_result(WorkflowStepResult {
                        step_id: step.id.clone(),
                        skill: step.action.clone(),
                        input: step
                            .parameters
                            .iter()
                            .map(|(k, v)| (k.clone(), self.value_ref_to_value(v)))
                            .collect(),
                        output: output.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    if let Some(error_handler) = &step.on_error {
                        match error_handler.action.as_str() {
                            "skip" => {
                                context.add_step_result(WorkflowStepResult {
                                    step_id: step.id.clone(),
                                    skill: step.action.clone(),
                                    input: step
                                        .parameters
                                        .iter()
                                        .map(|(k, v)| (k.clone(), self.value_ref_to_value(v)))
                                        .collect(),
                                    output: String::new(),
                                    success: false,
                                    error: Some(e.to_string()),
                                });
                                continue;
                            }
                            "fail" => {
                                return Err(anyhow::anyhow!("Step {} failed: {}", step.id, e));
                            }
                            _ => {
                                return Err(anyhow::anyhow!("Step {} failed: {}", step.id, e));
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!("Step {} failed: {}", step.id, e));
                    }
                }
            }
        }
        if let Some(last_result) = context.get_step_results().last() {
            Ok(last_result.output.clone())
        } else {
            Ok(t!("skill.no_steps_executed").to_string())
        }
    }

    async fn execute_step_with_retry(
        &self,
        skill_name: &str,
        parameters: HashMap<String, Value>,
        step: &WorkflowStep,
    ) -> anyhow::Result<String> {
        let max_retries = step
            .on_error
            .as_ref()
            .and_then(|e| e.max_retries)
            .unwrap_or(1);
        let mut last_error = None;
        for attempt in 0..max_retries {
            let call = SkillCall {
                action: skill_name.to_string(),
                parameters: parameters.clone(),
            };
            match self.executor.execute(&call).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            (100 * (attempt + 1)).into(),
                        ))
                        .await;
                    }
                }
            }
        }
        Err(anyhow::anyhow!(
            last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"))
        ))
    }

    fn evaluate_condition(&self, condition: &Condition, context: &Workflow) -> bool {
        let left = self.resolve_value_ref(&condition.left, context);
        let right = self.resolve_value_ref(&condition.right, context);
        match condition.op.as_str() {
            "eq" => left == right,
            "ne" => left != right,
            "gt" => {
                if let (Some(left_num), Some(right_num)) = (left.as_u64(), right.as_u64()) {
                    left_num > right_num
                } else if let (Some(left_num), Some(right_num)) = (left.as_f64(), right.as_f64()) {
                    left_num > right_num
                } else {
                    false
                }
            }
            "lt" => {
                if let (Some(left_num), Some(right_num)) = (left.as_u64(), right.as_u64()) {
                    left_num < right_num
                } else if let (Some(left_num), Some(right_num)) = (left.as_f64(), right.as_f64()) {
                    left_num < right_num
                } else {
                    false
                }
            }
            "contains" => left
                .as_str()
                .map(|s| s.contains(right.as_str().unwrap_or("")))
                .unwrap_or(false),
            _ => false,
        }
    }

    fn resolve_value_ref(&self, value_ref: &ValueRef, context: &Workflow) -> Value {
        match value_ref {
            ValueRef::Literal(value) => value.clone(),
            ValueRef::Reference(ref_reference) => {
                let path = &ref_reference.path;
                if let Some(value) = context.get_variable(path) {
                    value.clone()
                } else if path == "user_input" {
                    // Special case for user input
                    Value::Null
                } else {
                    Value::Null
                }
            }
            ValueRef::Expression(expr) => Value::String(expr.expr.clone()),
        }
    }

    fn resolve_variables(value: &Value, context: &HashMap<String, Value>) -> Value {
        if let Some(s) = value.as_str() {
            if s.starts_with("{{") && s.ends_with("}}") {
                let var_name = &s[2..s.len() - 2];
                if let Some(val) = context.get(var_name) {
                    return val.clone();
                }
            }
            if let Ok(json_val) = serde_json::from_str::<Value>(s) {
                if let Some(ref_obj) = json_val.as_object() {
                    if let Some(var_name) = ref_obj.get("$ref").and_then(|v| v.as_str()) {
                        if let Some(val) = context.get(var_name) {
                            return val.clone();
                        }
                    }
                }
            }
        }
        value.clone()
    }

    async fn execute_batch_plan(&self, steps: &[SkillCall]) -> Vec<StepResult> {
        if steps.is_empty() {
            return Vec::new();
        }
        let callback = self.callback.clone();
        let futures = steps.iter().enumerate().map(|(idx, step)| {
            let step = step.clone();
            let executor = self.executor.clone();
            let callback = callback.clone();
            tokio::spawn(async move {
                let step_name = step.action.clone();
                if let Some(cb) = &callback {
                    cb.on_step_start(&step_name, idx).await;
                }
                match executor.execute(&step).await {
                    Ok(output) => {
                        if let Some(cb) = &callback {
                            cb.on_step_success(&step_name, idx, &output).await;
                        }
                        Some(StepResult {
                            skill: step.action.clone(),
                            parameters: step.parameters.clone(),
                            output,
                            status: ExecutionStatus::Success,
                        })
                    }
                    Err(e) => {
                        let error_msg = format!("Failed: {}", e);
                        if let Some(cb) = &callback {
                            cb.on_step_failure(&step_name, idx, &error_msg).await;
                        }
                        Some(StepResult {
                            skill: step.action.clone(),
                            parameters: step.parameters.clone(),
                            output: error_msg,
                            status: ExecutionStatus::Failure,
                        })
                    }
                }
            })
        });
        let results = join_all(futures).await;
        results
            .into_iter()
            .filter_map(|r| r.ok().flatten())
            .collect()
    }

    fn format_step_results(&self, results: &[StepResult]) -> String {
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

    pub fn parse_react_response(response: &str) -> anyhow::Result<ReactInstruction> {
        let json_str = Self::extract_json(response);
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

    pub fn parse_chain_response(response: &str) -> anyhow::Result<ChainPlan> {
        let json_str = Self::extract_json(response);
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

    pub fn parse_plan_response(response: &str) -> anyhow::Result<PlanInstruction> {
        let json_str = Self::extract_json(response);
        let instruction: PlanInstruction = serde_json::from_str(&json_str)?;
        Ok(instruction)
    }

    pub fn extract_json(text: &str) -> String {
        if let Some(start) = text.find("```json") {
            let after_start = &text[start + 7..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        if let Some(start) = text.find("```") {
            let after_start = &text[start + 3..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                return text[start..=end].to_string();
            }
        }
        text.to_string()
    }

    pub fn get_mode(&self) -> WorkflowMode {
        self.mode
    }

    fn value_ref_to_value(&self, value_ref: &ValueRef) -> Value {
        match value_ref {
            ValueRef::Literal(value) => value.clone(),
            ValueRef::Reference(ref_reference) => {
                Value::String(format!("$ref:{}", ref_reference.path))
            }
            ValueRef::Expression(expr) => Value::String(format!("$expr:{}", expr.expr)),
        }
    }

    /// Execute workflow with pre-built registries (optimized version)
    ///
    /// # Arguments
    /// * `skills_dir` - Optional skills directory path. Only needed for SKILL.md loading.
    ///                  Pass `None` if not executing SKILL.md files.
    pub async fn execute(
        &self,
        scheduler: &SkillScheduler,
        memory: &ConversationMemory,
        input: &str,
        session_id: &str,
        skills_registry: &str,
        instances_registry: &str,
        is_first_message: bool,
    ) -> String {
        match self.mode {
            WorkflowMode::ReAct => {
                self.execute_react(
                    scheduler,
                    memory,
                    input,
                    session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
            WorkflowMode::Batch => {
                self.execute_batch(
                    scheduler,
                    input,
                    session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
            WorkflowMode::Chain => {
                self.execute_chain(
                    scheduler,
                    input,
                    session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
            WorkflowMode::PlanAndExecute => {
                self.execute_plan_and_execute(
                    scheduler,
                    input,
                    session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
        }
    }

    /// Execute a SKILL.md workflow file
    pub async fn execute_skill_md(
        &self,
        scheduler: &SkillScheduler,
        memory: &ConversationMemory,
        skill_file: &crate::skill_loader::SkillFile,
        params: Option<&HashMap<String, Value>>,
        skills_registry: &str,
        instances_registry: &str,
        is_first_message: bool,
    ) -> String {
        let mut instructions = skill_file.instructions.clone();
        // Substitute parameters
        if let Some(params) = params {
            for (key, value) in params {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                instructions = instructions.replace(&placeholder, &replacement);
            }
        }
        let enhanced_input = format!(
            "{}\n\n## Available Atomic Skills\n{}\n\n## Available Instances\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
            instructions, skills_registry, instances_registry
        );
        let session_id = format!("skill_md_{}", skill_file.name);
        match self.mode {
            WorkflowMode::ReAct => {
                self.execute_react(
                    scheduler,
                    memory,
                    &enhanced_input,
                    &session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
            WorkflowMode::Batch => {
                self.execute_batch(
                    scheduler,
                    &enhanced_input,
                    &session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
            WorkflowMode::Chain => {
                self.execute_chain(
                    scheduler,
                    &enhanced_input,
                    &session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
            WorkflowMode::PlanAndExecute => {
                self.execute_plan_and_execute(
                    scheduler,
                    &enhanced_input,
                    &session_id,
                    skills_registry,
                    instances_registry,
                    is_first_message,
                )
                .await
            }
        }
    }

    /// ReAct mode with registry (optimized)
    async fn execute_react(
        &self,
        scheduler: &SkillScheduler,
        memory: &ConversationMemory,
        input: &str,
        session_id: &str,
        skills_registry: &str,
        instances_registry: &str,
        is_first_message: bool,
    ) -> String {
        let input_trimmed = input.trim();
        if input_trimmed == "clear" {
            memory.clear_session(session_id);
            return t!("app.conversation_cleared").to_string();
        }
        if input_trimmed == "exit" || input_trimmed == "quit" {
            return "goodbye".to_string();
        }
        if input_trimmed.is_empty() {
            return String::new();
        }

        let history = memory.get_history(session_id);
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut final_response = None;
        let mut iteration = 0;

        while iteration < self.max_iterations {
            iteration += 1;
            let execution_summary = if step_results.is_empty() {
                String::new()
            } else {
                let mut summary = format!("\n## {}\n", t!("skill.previous_executed_steps"));
                for (i, result) in step_results.iter().enumerate() {
                    summary.push_str(&format!("{}. {}\n", i + 1, result.to_string()));
                }
                summary
            };

            // Build prompt with cached registries
            let system_prompt = Self::build_react_prompt(skills_registry, instances_registry);
            let user_prompt = if is_first_message && history.is_empty() && step_results.is_empty() {
                // First message: include registries (already in system_prompt)
                format!(
                    "{}\n\n## {}\n{}\n\n## {}\n\n## {}\n",
                    system_prompt,
                    t!("prompt.original_request"),
                    input_trimmed,
                    t!("prompt.your_response"),
                    t!("prompt.first_message_hint")
                )
            } else {
                format!(
                    "{}\n\n## {}\n{}\n\n## {}\n{}\n\n## {}\n",
                    system_prompt,
                    t!("prompt.original_request"),
                    input_trimmed,
                    t!("prompt.conversation_history"),
                    history,
                    t!("prompt.your_response")
                )
            };

            let llm_response = match scheduler.get_llm().generate(&user_prompt).await {
                Ok(resp) => resp,
                Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
            };

            let instruction = match Self::parse_react_response(&llm_response) {
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
                    if let Some(cb) = &self.callback {
                        cb.on_step_start(&step_name, step_index).await;
                    }
                    match self.executor.execute(&call).await {
                        Ok(output) => {
                            if let Some(cb) = &self.callback {
                                cb.on_step_success(&step_name, step_index, &output).await;
                            }
                            step_results.push(StepResult {
                                skill: call.action.clone(),
                                parameters: call.parameters.clone(),
                                output: output.clone(),
                                status: ExecutionStatus::Success,
                            });
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            if let Some(cb) = &self.callback {
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
                    let results = self.execute_batch_plan(&steps).await;
                    for result in results {
                        step_results.push(result);
                    }
                    let summary = self.format_step_results(&step_results);
                    final_response = Some(summary);
                    break;
                }
            }
        }

        if iteration >= self.max_iterations {
            final_response = Some(t!("error.max_iterations_reached").to_string());
        }

        let final_response = final_response.unwrap_or_else(|| {
            if step_results.is_empty() {
                t!("skill.no_actions_executed").to_string()
            } else {
                self.format_step_results(&step_results)
            }
        });

        if let Some(cb) = &self.callback {
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

        memory.add_exchange(session_id, input, &final_response);
        final_response
    }

    /// Build ReAct prompt with pre-built registries
    pub fn build_react_prompt(skills_registry: &str, instances_registry: &str) -> String {
        format!(
            r#"You are Hippox, a reliable AI runtime and skills orchestration engine with autonomous decision-making.

## Available Atomic Skills (JSON Registry)
{}

## Available Instances (Configured Services)
{}

## Response Format

You can respond in one of three ways:

### 1. Execute a single skill
{{"action": "skill_name", "parameters": {{"param1": "value1"}}}}

### 2. Execute multiple skills in sequence (no dependencies)
{{
  "mode": "batch",
  "steps": [
    {{"action": "skill1", "parameters": {{}}}},
    {{"action": "skill2", "parameters": {{}}}}
  ]
}}

### 3. Finish and return final answer
{{"action": "done", "message": "Your final answer here"}}

## Rules

- If the task requires conditional logic (e.g., "if rain then send email"), use mode "single" and execute one skill at a time
- After each skill execution, you will receive the result and can decide the next step
- Use "batch" mode only when skills have no dependencies on each other's results
- Use "done" when you have completed the task or no skill is needed
- When calling database skills, choose the appropriate instance_id from the Available Instances list based on user's intent

## Previous Execution Results (if any)
"#,
            skills_registry, instances_registry
        )
    }

    // Batch mode with registry
    async fn execute_batch(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
        session_id: &str,
        skills_registry: &str,
        instances_registry: &str,
        is_first_message: bool,
    ) -> String {
        let batch_prompt = format!(
            r#"You are Hippox, a reliable AI runtime and skills orchestration engine with autonomous decision-making.

## Available Atomic Skills (JSON Registry)
{}

## Available Instances
{}

## Task
Execute multiple skills in batch mode. Skills should have NO dependencies on each other.

## Response Format
{{
  "mode": "batch",
  "steps": [
    {{"action": "skill1", "parameters": {{}}}},
    {{"action": "skill2", "parameters": {{}}}}
  ]
}}

If no skills are needed, respond with:
{{"action": "done", "message": "Your answer"}}

## User Input
{}

Respond with ONLY the JSON.
"#,
            skills_registry, instances_registry, input
        );

        let llm_response = match scheduler.get_llm().generate(&batch_prompt).await {
            Ok(resp) => resp,
            Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
        };

        let instruction = match Self::parse_react_response(&llm_response) {
            Ok(instr) => instr,
            Err(_) => return llm_response,
        };

        match instruction {
            ReactInstruction::Done(message) => message,
            ReactInstruction::Batch(steps) => {
                let results = self.execute_batch_plan(&steps).await;
                self.format_step_results(&results)
            }
            ReactInstruction::Single(_) => t!("error.batch_mode_invalid").to_string(),
        }
    }

    // Chain mode with registry
    async fn execute_chain(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
        session_id: &str,
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

        let chain = match Self::parse_chain_response(&llm_response) {
            Ok(chain) => chain,
            Err(e) => return format!("Failed to parse chain: {}", e),
        };

        let mut context = HashMap::new();
        context.insert("user_input".to_string(), Value::String(input.to_string()));
        let mut results = Vec::new();

        for (idx, step) in chain.steps.iter().enumerate() {
            let step_name = step.action.clone();
            if let Some(cb) = &self.callback {
                cb.on_step_start(&step_name, idx).await;
            }
            let mut resolved_params = HashMap::new();
            for (key, value) in &step.parameters {
                let resolved = Self::resolve_variables_deep(value, &context);
                resolved_params.insert(key.clone(), resolved);
            }
            let call = SkillCall {
                action: step.action.clone(),
                parameters: resolved_params,
            };
            match self.executor.execute(&call).await {
                Ok(output) => {
                    if let Some(cb) = &self.callback {
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
                    if let Some(cb) = &self.callback {
                        cb.on_step_failure(&step_name, idx, &error_msg).await;
                    }
                    results.push(StepResult {
                        skill: step.action.clone(),
                        parameters: call.parameters,
                        output: error_msg.clone(),
                        status: ExecutionStatus::Failure,
                    });
                    if let Some(cb) = &self.callback {
                        cb.on_workflow_failed(&error_msg).await;
                    }
                    return format!("Skill '{}' failed: {}", step.action, error_msg);
                }
            }
        }

        let final_output = self.format_step_results(&results);
        if let Some(cb) = &self.callback {
            let has_failure = results.iter().any(|r| r.status == ExecutionStatus::Failure);
            if has_failure {
                cb.on_workflow_failed(&final_output).await;
            } else {
                cb.on_workflow_complete(&final_output).await;
            }
        }
        final_output
    }

    // Plan-and-Execute mode with registry
    async fn execute_plan_and_execute(
        &self,
        scheduler: &SkillScheduler,
        input: &str,
        session_id: &str,
        skills_registry: &str,
        instances_registry: &str,
        is_first_message: bool,
    ) -> String {
        let plan_prompt = format!(
            r#"You are Hippox, a reliable AI runtime and skills orchestration engine with autonomous decision-making.

## Available Atomic Skills (JSON Registry)
{}

## Available Instances
{}

## Task
Create a complete execution plan that handles dependencies and conditions.

## IMPORTANT: Response Format
You MUST return a JSON object with exactly this structure:

{{"mode": "plan", "plan": {{"steps": [
  {{"id": "step1", "action": "skill_name", "parameters": {{"param": "value"}}, "output_as": "result1"}},
  {{"id": "step2", "action": "skill_name", "parameters": {{"input": "{{{{result1}}}}"}}, "condition": {{"op": "contains", "left": "{{{{result1}}}}", "right": "error"}}}}
]}}}}

## Condition Operators
- eq: equal
- ne: not equal
- gt: greater than
- lt: less than
- contains: string contains

## User Input
{}

If no skills are needed:
{{"mode": "done", "message": "Your answer"}}

Respond with ONLY the JSON. No markdown, no extra text.
"#,
            skills_registry, instances_registry, input
        );

        let llm_response = match scheduler.get_llm().generate(&plan_prompt).await {
            Ok(resp) => resp,
            Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
        };

        let instruction = match Self::parse_plan_response(&llm_response) {
            Ok(instr) => instr,
            Err(e) => return format!("Failed to parse plan: {}", e),
        };

        match instruction {
            PlanInstruction {
                mode,
                plan,
                message,
            } => {
                if mode == "done" {
                    return message.unwrap_or_else(|| t!("skill.no_actions_executed").to_string());
                }

                if let Some(plan) = plan {
                    match self.execute_workflow_plan(&plan).await {
                        Ok(result) => result,
                        Err(e) => format!("Workflow failed: {}", e),
                    }
                } else {
                    t!("skill.no_actions_executed").to_string()
                }
            }
        }
    }
}

/// Execution status for a workflow step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

/// Step result for multi-step execution
#[derive(Debug, Clone)]
pub struct StepResult {
    pub skill: String,
    pub parameters: HashMap<String, Value>,
    pub output: String,
    pub status: ExecutionStatus,
}

impl StepResult {
    pub fn to_string(&self) -> String {
        let status_str = match self.status {
            ExecutionStatus::Success => "SUCCESS",
            ExecutionStatus::Failure => "FAILURE",
        };
        format!(
            "{} Executed skill '{}' with parameters {:?}\nResult: {}",
            status_str, self.skill, self.parameters, self.output
        )
    }
}

/// Internal instruction enum for ReAct mode parsing
#[derive(Debug)]
pub enum ReactInstruction {
    Done(String),
    Single(SkillCall),
    Batch(Vec<SkillCall>),
}

/// Chain plan definition
#[derive(Debug)]
pub struct ChainPlan {
    pub steps: Vec<ChainStepDef>,
}

#[derive(Debug)]
pub struct ChainStepDef {
    pub action: String,
    pub parameters: HashMap<String, Value>,
    pub output_as: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_mode_default() {
        let mode = WorkflowMode::default();
        assert_eq!(mode, WorkflowMode::ReAct);
    }

    #[test]
    fn test_workflow_mode_display() {
        assert_eq!(format!("{}", WorkflowMode::ReAct), "ReAct");
        assert_eq!(format!("{}", WorkflowMode::Batch), "Batch");
        assert_eq!(format!("{}", WorkflowMode::Chain), "Chain");
        assert_eq!(
            format!("{}", WorkflowMode::PlanAndExecute),
            "PlanAndExecute"
        );
    }

    #[test]
    fn test_extract_json_from_markdown() {
        let text = "```json\n{\"action\": \"test\"}\n```";
        let json = WorkflowExecutor::extract_json(text);
        assert_eq!(json, "{\"action\": \"test\"}");
    }

    #[test]
    fn test_extract_json_from_plain() {
        let text = "Some text {\"action\": \"test\"} and more";
        let json = WorkflowExecutor::extract_json(text);
        assert_eq!(json, "{\"action\": \"test\"}");
    }

    #[test]
    fn test_workflow_context() {
        let mut context = Workflow::new();
        context.set_variable("test", Value::String("value".to_string()));
        assert_eq!(
            context.get_variable("test"),
            Some(&Value::String("value".to_string()))
        );
    }

    #[test]
    fn test_condition_evaluation() {
        let executor = WorkflowExecutor::new(WorkflowMode::PlanAndExecute);
        let mut context = Workflow::new();
        context.set_variable("result", Value::Number(serde_json::Number::from(42)));
        let condition = Condition {
            op: "eq".to_string(),
            left: ValueRef::Reference(Reference {
                path: "result".to_string(),
            }),
            right: ValueRef::Literal(Value::Number(serde_json::Number::from(42))),
        };
        assert!(executor.evaluate_condition(&condition, &context));
    }
}

/// test plan and execute workflow
#[cfg(test)]
mod test_plan_and_execute_workflow {
    use super::*;
    use crate::executors::types::{Skill, SkillParameter};
    use serde_json::json;

    // mock level 1 Skill - calculate square
    #[derive(Debug)]
    struct MockCalculatorSkill;
    #[async_trait::async_trait]
    impl Skill for MockCalculatorSkill {
        fn name(&self) -> &str {
            "calculator"
        }
        fn description(&self) -> &str {
            "Calculate the square of the input number"
        }
        fn category(&self) -> &str {
            "math"
        }
        fn parameters(&self) -> Vec<SkillParameter> {
            vec![SkillParameter {
                name: "input".to_string(),
                param_type: "string".to_string(),
                description: "Input number".to_string(),
                required: true,
                default: None,
                example: Some(json!("5")),
                enum_values: None,
            }]
        }
        async fn execute(&self, params: &HashMap<String, Value>) -> anyhow::Result<String> {
            let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("0");
            let num: i32 = input.parse().unwrap_or(0);
            Ok(format!("Square result: {}", num * num))
        }
    }

    // mock level 2 Skill - multiply by 3
    #[derive(Debug)]
    struct MockMultiplierSkill;
    #[async_trait::async_trait]
    impl Skill for MockMultiplierSkill {
        fn name(&self) -> &str {
            "multiplier"
        }
        fn description(&self) -> &str {
            "Multiply the input by 3"
        }
        fn category(&self) -> &str {
            "math"
        }
        fn parameters(&self) -> Vec<SkillParameter> {
            vec![SkillParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Input number".to_string(),
                required: true,
                default: None,
                example: Some(json!("25")),
                enum_values: None,
            }]
        }
        async fn execute(&self, params: &HashMap<String, Value>) -> anyhow::Result<String> {
            let input = params.get("value").and_then(|v| v.as_str()).unwrap_or("0");
            let num: i32 = input.parse().unwrap_or(0);
            Ok(format!("Multiply by 3 result: {}", num * 3))
        }
    }

    // mock level 3 Skill - add 10
    #[derive(Debug)]
    struct MockAdderSkill;
    #[async_trait::async_trait]
    impl Skill for MockAdderSkill {
        fn name(&self) -> &str {
            "adder"
        }
        fn description(&self) -> &str {
            "Add 10 to the input"
        }
        fn category(&self) -> &str {
            "math"
        }
        fn parameters(&self) -> Vec<SkillParameter> {
            vec![SkillParameter {
                name: "number".to_string(),
                param_type: "string".to_string(),
                description: "Input number".to_string(),
                required: true,
                default: None,
                example: Some(json!("75")),
                enum_values: None,
            }]
        }
        async fn execute(&self, params: &HashMap<String, Value>) -> anyhow::Result<String> {
            let input = params.get("number").and_then(|v| v.as_str()).unwrap_or("0");
            let num: i32 = input.parse().unwrap_or(0);
            Ok(format!("Add 10 result: {}", num + 10))
        }
    }

    // mock level 4 Skill - format output
    #[derive(Debug)]
    struct MockFormatterSkill;
    #[async_trait::async_trait]
    impl Skill for MockFormatterSkill {
        fn name(&self) -> &str {
            "formatter"
        }
        fn description(&self) -> &str {
            "Format the final output"
        }
        fn category(&self) -> &str {
            "document"
        }
        fn parameters(&self) -> Vec<SkillParameter> {
            vec![SkillParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Input content".to_string(),
                required: true,
                default: None,
                example: Some(json!("Add 10 result: 85")),
                enum_values: None,
            }]
        }
        async fn execute(&self, params: &HashMap<String, Value>) -> anyhow::Result<String> {
            let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
            Ok(format!("[Final Answer] {}", content))
        }
    }

    // Register all mock skills to the global registry
    fn register_mock_skills() {
        use crate::executors::registry::register_skill;
        use std::sync::Arc;
        let _ = register_skill("calculator".to_string(), Arc::new(MockCalculatorSkill));
        let _ = register_skill("multiplier".to_string(), Arc::new(MockMultiplierSkill));
        let _ = register_skill("adder".to_string(), Arc::new(MockAdderSkill));
        let _ = register_skill("formatter".to_string(), Arc::new(MockFormatterSkill));
    }

    #[tokio::test]
    async fn test_4layer_nesting_workflow() {
        register_mock_skills();
        let executor = WorkflowExecutor::new(WorkflowMode::PlanAndExecute);
        let mut parameters = HashMap::new();
        parameters.insert(
            "user_input".to_string(),
            Value::Number(serde_json::Number::from(5)),
        );
        let steps = vec![
            WorkflowStep {
                id: "step1".to_string(),
                action: "calculator".to_string(),
                parameters: {
                    let mut map = HashMap::new();
                    map.insert(
                        "input".to_string(),
                        ValueRef::Reference(Reference {
                            path: "user_input".to_string(),
                        }),
                    );
                    map
                },
                condition: None,
                output_as: Some("result1".to_string()),
                on_error: None,
            },
            WorkflowStep {
                id: "step2".to_string(),
                action: "multiplier".to_string(),
                parameters: {
                    let mut map = HashMap::new();
                    map.insert(
                        "value".to_string(),
                        ValueRef::Reference(Reference {
                            path: "result1".to_string(),
                        }),
                    );
                    map
                },
                condition: None,
                output_as: Some("result2".to_string()),
                on_error: None,
            },
            WorkflowStep {
                id: "step3".to_string(),
                action: "adder".to_string(),
                parameters: {
                    let mut map = HashMap::new();
                    map.insert(
                        "number".to_string(),
                        ValueRef::Reference(Reference {
                            path: "result2".to_string(),
                        }),
                    );
                    map
                },
                condition: None,
                output_as: Some("result3".to_string()),
                on_error: None,
            },
            WorkflowStep {
                id: "step4".to_string(),
                action: "formatter".to_string(),
                parameters: {
                    let mut map = HashMap::new();
                    map.insert(
                        "content".to_string(),
                        ValueRef::Reference(Reference {
                            path: "result3".to_string(),
                        }),
                    );
                    map
                },
                condition: None,
                output_as: Some("final".to_string()),
                on_error: None,
            },
        ];
        let plan = WorkflowPlan {
            name: Some("4layer_test".to_string()),
            steps,
            parameters,
        };
        println!(
            "Nested skill JSON structure:\n{}",
            serde_json::to_string_pretty(&plan).unwrap()
        );
        let result = executor.execute_workflow_plan(&plan).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        println!("4-layer nesting execution result: {}", output);
        assert!(output.contains("[Final Answer]"));
    }
}

/// test react workflow mode
#[cfg(test)]
mod test_react_workflow {
    use super::*;
    use crate::executors::types::{Skill, SkillParameter};
    use serde_json::json;
    use std::sync::Arc;

    // Mock skill for testing - returns a simple greeting
    #[derive(Debug)]
    struct MockGreetingSkill;
    #[async_trait::async_trait]
    impl Skill for MockGreetingSkill {
        fn name(&self) -> &str {
            "greeting"
        }
        fn description(&self) -> &str {
            "Return a greeting message"
        }
        fn category(&self) -> &str {
            "message"
        }
        fn parameters(&self) -> Vec<SkillParameter> {
            vec![SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Name to greet".to_string(),
                required: false,
                default: Some(json!("World")),
                example: Some(json!("Alice")),
                enum_values: None,
            }]
        }
        async fn execute(&self, params: &HashMap<String, Value>) -> anyhow::Result<String> {
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");
            Ok(format!("Hello, {}!", name))
        }
    }

    // Mock skill for testing - performs basic calculation
    #[derive(Debug)]
    struct MockCalculationSkill;
    #[async_trait::async_trait]
    impl Skill for MockCalculationSkill {
        fn name(&self) -> &str {
            "calculate"
        }
        fn description(&self) -> &str {
            "Perform basic arithmetic operations"
        }
        fn category(&self) -> &str {
            "math"
        }
        fn parameters(&self) -> Vec<SkillParameter> {
            vec![
                SkillParameter {
                    name: "a".to_string(),
                    param_type: "string".to_string(),
                    description: "First number".to_string(),
                    required: true,
                    default: None,
                    example: Some(json!("10")),
                    enum_values: None,
                },
                SkillParameter {
                    name: "b".to_string(),
                    param_type: "string".to_string(),
                    description: "Second number".to_string(),
                    required: true,
                    default: None,
                    example: Some(json!("5")),
                    enum_values: None,
                },
                SkillParameter {
                    name: "operation".to_string(),
                    param_type: "string".to_string(),
                    description: "Operation: add, subtract, multiply, divide".to_string(),
                    required: true,
                    default: None,
                    example: Some(json!("add")),
                    enum_values: Some(vec![
                        "add".to_string(),
                        "subtract".to_string(),
                        "multiply".to_string(),
                        "divide".to_string(),
                    ]),
                },
            ]
        }

        async fn execute(&self, params: &HashMap<String, Value>) -> anyhow::Result<String> {
            let a: i32 = params
                .get("a")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let b: i32 = params
                .get("b")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let op = params
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("add");
            let result = match op {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b != 0 {
                        a / b
                    } else {
                        return Ok("Error: Division by zero".to_string());
                    }
                }
                _ => 0,
            };
            Ok(format!("Result: {}", result))
        }
    }

    // Register mock skills for ReAct testing
    fn register_react_mock_skills() {
        use crate::executors::registry::register_skill;
        let _ = register_skill("greeting".to_string(), Arc::new(MockGreetingSkill));
        let _ = register_skill("calculate".to_string(), Arc::new(MockCalculationSkill));
    }

    #[tokio::test]
    async fn test_react_mode_with_single_skill() {
        register_react_mock_skills();
        let executor = WorkflowExecutor::new(WorkflowMode::ReAct);
        let memory = ConversationMemory::new();
        let skills_dir = PathBuf::from(".");
        assert_eq!(executor.get_mode(), WorkflowMode::ReAct);
    }

    #[tokio::test]
    async fn test_react_mode_executor_creation() {
        let executor = WorkflowExecutor::new(WorkflowMode::ReAct);
        assert_eq!(executor.get_mode(), WorkflowMode::ReAct);
        assert_eq!(executor.max_iterations, 10);
        let executor_with_custom_iter =
            WorkflowExecutor::new(WorkflowMode::ReAct).with_max_iterations(5);
        assert_eq!(executor_with_custom_iter.max_iterations, 5);
    }

    #[tokio::test]
    async fn test_react_mode_batch_instruction_parsing() {
        let response = r#"{
            "mode": "batch",
            "steps": [
                {"action": "greeting", "parameters": {"name": "Alice"}},
                {"action": "calculate", "parameters": {"a": "10", "b": "5", "operation": "add"}}
            ]
        }"#;
        let instruction = WorkflowExecutor::parse_react_response(response);
        assert!(instruction.is_ok());
        match instruction.unwrap() {
            ReactInstruction::Batch(steps) => {
                assert_eq!(steps.len(), 2);
                assert_eq!(steps[0].action, "greeting");
                assert_eq!(steps[1].action, "calculate");
            }
            _ => panic!("Expected Batch instruction"),
        }
    }

    #[tokio::test]
    async fn test_react_mode_done_instruction_parsing() {
        let response = r#"{"action": "done", "message": "Task completed successfully"}"#;
        let instruction = WorkflowExecutor::parse_react_response(response);
        assert!(instruction.is_ok());
        match instruction.unwrap() {
            ReactInstruction::Done(message) => {
                assert_eq!(message, "Task completed successfully");
            }
            _ => panic!("Expected Done instruction"),
        }
    }

    #[tokio::test]
    async fn test_react_mode_single_instruction_parsing() {
        let response = r#"{"action": "greeting", "parameters": {"name": "Bob"}}"#;
        let instruction = WorkflowExecutor::parse_react_response(response);
        assert!(instruction.is_ok());
        match instruction.unwrap() {
            ReactInstruction::Single(call) => {
                assert_eq!(call.action, "greeting");
                assert_eq!(
                    call.parameters.get("name").and_then(|v| v.as_str()),
                    Some("Bob")
                );
            }
            _ => panic!("Expected Single instruction"),
        }
    }

    #[test]
    fn test_react_mode_extract_json_from_response() {
        let response = "Here is the JSON: ```json\n{\"action\": \"test\"}\n```";
        let json = WorkflowExecutor::extract_json(response);
        assert_eq!(json, "{\"action\": \"test\"}");
        let response = "Response: {\"action\": \"calculate\", \"parameters\": {\"a\": \"1\"}}";
        let json = WorkflowExecutor::extract_json(response);
        assert!(json.contains("calculate"));
    }

    #[tokio::test]
    async fn test_react_mode_clear_command() {
        let memory = ConversationMemory::new();
        let session_id = "test_session";
        // Add some conversation
        memory.add_exchange(session_id, "Hello", "Hi there!");
        assert!(!memory.get_history(session_id).is_empty());
        let executor = WorkflowExecutor::new(WorkflowMode::ReAct);
        let skills_dir = PathBuf::from(".");
        let input = "clear";
        let input_trimmed = input.trim();
        assert_eq!(input_trimmed, "clear");
    }
}
