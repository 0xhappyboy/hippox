//! Workflow execution module for Hippox core
//!
//! This module provides different workflow execution modes:
//! - ReAct: Traditional think-act-observe loop
//! - Batch: Parallel execution of independent skills
//! - PlanAndExecute: One-time planning with dependency resolution
//! - Chain: Simple sequential execution with variable passing

pub(crate) mod batch;
pub(crate) mod chain;
pub(crate) mod core;
pub(crate) mod plan_and_execute;
pub(crate) mod prompt;
pub(crate) mod react;
pub mod types;
pub(crate) mod utils;

pub(crate) use batch::*;
pub(crate) use chain::*;
pub(crate) use core::*;
pub(crate) use plan_and_execute::*;
pub(crate) use prompt::*;
pub(crate) use react::*;
pub use types::*;
pub(crate) use utils::*;
