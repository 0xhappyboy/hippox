//! Type definitions for workflow execution

use crate::executors::SkillCall;
use async_trait::async_trait;
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

/// Step interruption info for callback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepInterruptionInfo {
    /// Whether the step was interrupted
    pub interrupted: bool,
    /// Reason for interruption: "cancelled" or "paused"
    pub reason: String,
    /// Current step index when interrupted
    pub step_index: usize,
    /// Step name when interrupted
    pub step_name: String,
    /// Checkpoint data if available
    pub checkpoint: Option<String>,
}

/// Workflow execution callback trait
///
/// Implement this trait to receive real-time updates about workflow execution.
/// This is useful for UI updates, logging, or progress reporting.
#[async_trait]
pub trait WorkflowCallback: Send + Sync + Debug {
    /// Called when a step (skill execution) starts
    /// - parameters: The parameters passed to the skill
    async fn on_step_start(
        &self,
        task_id: &str,
        step_name: &str,
        step_index: usize,
        parameters: Option<&HashMap<String, Value>>,
    );

    /// Called when a step completes successfully
    /// - output: The output from the skill execution
    /// - duration_ms: How long the step took to execute
    async fn on_step_success(
        &self,
        task_id: &str,
        step_name: &str,
        step_index: usize,
        output: &str,
        duration_ms: u64,
    );

    /// Called when a step fails
    /// - error: The error message
    /// - duration_ms: How long the step took before failing
    async fn on_step_failure(
        &self,
        task_id: &str,
        step_name: &str,
        step_index: usize,
        error: &str,
        duration_ms: u64,
    );

    /// Called when a step is interrupted (cancelled or paused)
    /// - info: Interruption information including reason and checkpoint
    async fn on_step_interrupted(&self, task_id: &str, info: &StepInterruptionInfo);

    /// Called when the entire workflow completes successfully
    /// - final_output: The final result of the workflow
    /// - total_duration_ms: Total time from start to completion
    /// - total_steps: Total number of steps executed
    async fn on_workflow_complete(
        &self,
        task_id: &str,
        final_output: &str,
        total_duration_ms: u64,
        total_steps: usize,
    );

    /// Called when the workflow fails
    /// - error: The error message
    /// - total_duration_ms: Total time from start to failure
    /// - total_steps: Number of steps executed before failure
    async fn on_workflow_failed(
        &self,
        task_id: &str,
        error: &str,
        total_duration_ms: u64,
        total_steps: usize,
    );

    /// Called when the workflow is cancelled
    /// - total_duration_ms: Total time from start to cancellation
    /// - total_steps: Number of steps executed before cancellation
    async fn on_workflow_cancelled(
        &self,
        task_id: &str,
        total_duration_ms: u64,
        total_steps: usize,
    );

    /// Called when the workflow is paused
    /// - checkpoint: The saved checkpoint data for resume
    /// - total_duration_ms: Total time from start to pause
    /// - total_steps: Number of steps executed before pause
    async fn on_workflow_paused(
        &self,
        task_id: &str,
        checkpoint: Option<&str>,
        total_duration_ms: u64,
        total_steps: usize,
    );
}

/// Helper function to truncate output for display
pub fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...", &output[..max_len])
    }
}

/// Context variable for workflow execution
#[derive(Debug, Clone)]
pub struct Workflow {
    /// Variable store for passing data between steps
    pub variables: HashMap<String, Value>,
    /// Step results for debugging
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

/// Execution status for a workflow step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

/// Step result for multi-step execution
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

/// Checkpoint data for workflow resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCheckpoint {
    /// Last completed step index
    pub last_completed_step: usize,
    /// Variables context
    pub variables: HashMap<String, Value>,
    /// Step results completed so far
    pub completed_results: Vec<StepResult>,
    /// Current workflow mode
    pub mode: WorkflowMode,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
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
