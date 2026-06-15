//! Type definitions for workflow execution

use async_trait::async_trait;
use hippox_atomic_skills::SkillCall;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Workflow execution mode enumeration
///
/// Defines the strategy for processing user requests and executing skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Workflow execution result
/// Workflow execution result
#[derive(Debug, Clone)]
pub enum WorkflowExecutionResult {
    Completed(String),
    /// Completed with separate raw JSON data for stage two conversion
    CompletedWithRaw {
        display: String,
        raw_json: String,
    },
    Paused {
        checkpoint: Option<String>,
        completed_steps: usize,
        partial_output: String,
    },
    Cancelled {
        completed_steps: usize,
    },
    Failed {
        error: String,
        completed_steps: usize,
    },
}

impl WorkflowExecutionResult {
    pub fn is_completed(&self) -> bool {
        matches!(
            self,
            WorkflowExecutionResult::Completed(_)
                | WorkflowExecutionResult::CompletedWithRaw { .. }
        )
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, WorkflowExecutionResult::Paused { .. })
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, WorkflowExecutionResult::Cancelled { .. })
    }

    /// Get the display output (for callbacks)
    pub fn display_output(&self) -> Option<&str> {
        match self {
            WorkflowExecutionResult::Completed(output) => Some(output),
            WorkflowExecutionResult::CompletedWithRaw { display, .. } => Some(display),
            WorkflowExecutionResult::Paused { partial_output, .. } => Some(partial_output),
            _ => None,
        }
    }

    /// Get the raw JSON output (for stage two conversion)
    pub fn raw_json(&self) -> Option<&str> {
        match self {
            WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => Some(raw_json),
            WorkflowExecutionResult::Completed(output) => Some(output),
            _ => None,
        }
    }

    pub fn final_output(&self) -> Option<&str> {
        self.raw_json()
    }
}

/// Step interruption info for callback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepInterruptionInfo {
    pub interrupted: bool,
    pub reason: String,
    pub step_index: usize,
    pub step_name: String,
    pub checkpoint: Option<String>,
}

/// Workflow execution callback trait
#[async_trait]
pub trait WorkflowCallback: Send + Sync + Debug {
    async fn on_step_start(
        &self,
        task_id: &str,
        step_name: &str,
        step_index: usize,
        parameters: Option<&HashMap<String, Value>>,
    );

    async fn on_step_success(
        &self,
        task_id: &str,
        step_name: &str,
        step_index: usize,
        output: &str,
        duration_ms: u64,
    );

    async fn on_step_failure(
        &self,
        task_id: &str,
        step_name: &str,
        step_index: usize,
        error: &str,
        duration_ms: u64,
    );

    async fn on_step_interrupted(&self, task_id: &str, info: &StepInterruptionInfo);

    async fn on_workflow_complete(
        &self,
        task_id: &str,
        final_output: &str,
        total_duration_ms: u64,
        total_steps: usize,
    );

    async fn on_workflow_failed(
        &self,
        task_id: &str,
        error: &str,
        total_duration_ms: u64,
        total_steps: usize,
    );

    async fn on_workflow_cancelled(
        &self,
        task_id: &str,
        total_duration_ms: u64,
        total_steps: usize,
    );

    async fn on_workflow_paused(
        &self,
        task_id: &str,
        checkpoint: Option<&str>,
        total_duration_ms: u64,
        total_steps: usize,
    );

    async fn on_workflow_resumed(&self, task_id: &str, total_duration_ms: u64, total_steps: usize) {
        let _ = (task_id, total_duration_ms, total_steps);
    }
}

pub fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...", &output[..max_len])
    }
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub variables: HashMap<String, Value>,
    pub step_results: Vec<WorkflowStepResult>,
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

#[derive(Debug, Clone)]
pub struct WorkflowStepResult {
    pub step_id: String,
    pub skill: String,
    pub input: HashMap<String, Value>,
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WorkflowStep {
    pub id: String,
    pub action: String,
    pub parameters: HashMap<String, ValueRef>,
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub output_as: Option<String>,
    #[serde(default)]
    pub on_error: Option<ErrorHandler>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum ValueRef {
    Literal(Value),
    Reference(Reference),
    Expression(Expression),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Expression {
    #[serde(rename = "$expr")]
    pub expr: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    pub op: String,
    pub left: ValueRef,
    pub right: ValueRef,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ErrorHandler {
    pub action: String,
    #[serde(default)]
    pub fallback: Option<ValueRef>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WorkflowPlan {
    pub name: Option<String>,
    pub steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PlanInstruction {
    pub mode: String,
    pub plan: Option<WorkflowPlan>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCheckpoint {
    pub last_completed_step: usize,
    pub variables: HashMap<String, Value>,
    pub completed_results: Vec<StepResult>,
    pub mode: WorkflowMode,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ReactInstruction {
    Done(String),
    Single(SkillCall),
    Batch(Vec<SkillCall>),
}

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
