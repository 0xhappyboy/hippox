//! Signal bus module for task and step control
//!
//! This module provides two global signal buses for controlling task execution:
//! - Task signal bus: controls entire tasks (pause, stop)
//! - Task Step signal bus: controls individual steps within a task

use hippox_atomic_skills::SkillSignal;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Signal state enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskSignalStatus {
    Pause,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSignal {
    status: TaskSignalStatus,
    msg: Option<String>,
}

/// Task signal bus: task_id -> Signal
pub(crate) static TASK_SIGNAL_BUS: Lazy<RwLock<HashMap<String, TaskSignal>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Task Step signal bus: task_id -> (step_index -> Signal)
pub(crate) static TASK_STEP_SIGNAL_BUS: Lazy<
    RwLock<HashMap<String, HashMap<String, SkillSignal>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));
