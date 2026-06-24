//! Task event bus module
//!
//! This module provides a global broadcast channel for task events.
//! Any component can subscribe to receive task lifecycle events
//! without needing to inject callbacks.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use super::core::{StepStatus, TaskStatus};

/// Task lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLifecycleEvent {
    pub task_id: String,
    pub event_type: TaskEventType,
    pub data: serde_json::Value,
}

/// Event types - reuse existing status enums where possible
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskEventType {
    /// Task was created
    Created,
    /// Task status changed (reuses TaskStatus)
    StatusChanged { status: TaskStatus },
    /// Step status changed (reuses StepStatus)
    StepChanged {
        step_index: usize,
        step_name: String,
        status: StepStatus,
        output: Option<String>,
        error: Option<String>,
        duration_ms: Option<u64>,
    },
    /// Task progress update
    Progress { progress: u8, message: String },
}

impl TaskLifecycleEvent {
    pub fn created(task_id: String) -> Self {
        Self {
            task_id,
            event_type: TaskEventType::Created,
            data: serde_json::Value::Null,
        }
    }

    pub fn status_changed(task_id: String, status: TaskStatus) -> Self {
        Self {
            task_id,
            event_type: TaskEventType::StatusChanged { status },
            data: serde_json::Value::Null,
        }
    }

    pub fn step_changed(
        task_id: String,
        step_index: usize,
        step_name: String,
        status: StepStatus,
        output: Option<String>,
        error: Option<String>,
        duration_ms: Option<u64>,
    ) -> Self {
        Self {
            task_id,
            event_type: TaskEventType::StepChanged {
                step_index,
                step_name,
                status,
                output,
                error,
                duration_ms,
            },
            data: serde_json::Value::Null,
        }
    }

    pub fn progress(task_id: String, progress: u8, message: String) -> Self {
        Self {
            task_id,
            event_type: TaskEventType::Progress { progress, message },
            data: serde_json::Value::Null,
        }
    }

    /// Check if this event indicates a terminal state
    pub fn is_terminal(&self) -> bool {
        match &self.event_type {
            TaskEventType::StatusChanged { status } => matches!(
                status,
                TaskStatus::Completed
                    | TaskStatus::Failed
                    | TaskStatus::Cancelled
                    | TaskStatus::Timeout
            ),
            _ => false,
        }
    }
}

/// Global task event bus
pub static TASK_POOL_EVENT_BUS: Lazy<Arc<broadcast::Sender<TaskLifecycleEvent>>> =
    Lazy::new(|| {
        let (tx, _) = broadcast::channel(256);
        Arc::new(tx)
    });

/// Task event subscriber
pub struct TaskEventSubscriber {
    receiver: broadcast::Receiver<TaskLifecycleEvent>,
}

impl TaskEventSubscriber {
    /// Create a new subscriber
    pub fn new() -> Self {
        Self {
            receiver: TASK_POOL_EVENT_BUS.subscribe(),
        }
    }

    /// Receive the next event (blocks until event is available)
    pub async fn recv(&mut self) -> Option<TaskLifecycleEvent> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => return Some(event),
                Err(broadcast::error::RecvError::Closed) => return None,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[EventBus] Lagged, skipped {} events", n);
                    // Continue loop to try receiving next event
                    continue;
                }
            }
        }
    }

    /// Receive only events for a specific task
    pub async fn recv_for_task(&mut self, target_task_id: &str) -> Option<TaskLifecycleEvent> {
        while let Some(event) = self.recv().await {
            if event.task_id == target_task_id {
                return Some(event);
            }
        }
        None
    }

    /// Wait for a specific task to reach a terminal state
    pub async fn wait_for_terminal(&mut self, target_task_id: &str) -> Option<TaskLifecycleEvent> {
        while let Some(event) = self.recv().await {
            if event.task_id == target_task_id && event.is_terminal() {
                return Some(event);
            }
        }
        None
    }

    /// Wait for a specific task to reach a specific status
    pub async fn wait_for_status(
        &mut self,
        target_task_id: &str,
        target_status: TaskStatus,
    ) -> Option<TaskLifecycleEvent> {
        while let Some(event) = self.recv().await {
            if event.task_id != target_task_id {
                continue;
            }
            if let TaskEventType::StatusChanged { status } = &event.event_type {
                if *status == target_status {
                    return Some(event);
                }
            }
        }
        None
    }
}

impl Default for TaskEventSubscriber {
    fn default() -> Self {
        Self::new()
    }
}

/// Publish a task pool event
pub fn publish_task_pool_event(event: TaskLifecycleEvent) {
    let _ = TASK_POOL_EVENT_BUS.send(event);
}

/// Create a subscriber for all events
pub fn subscribe() -> TaskEventSubscriber {
    TaskEventSubscriber::new()
}
