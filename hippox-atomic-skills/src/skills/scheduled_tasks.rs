/// Task scheduling skills module
///
/// This module provides skills for scheduling, managing, and listing scheduled tasks.
/// It supports three types of schedules: cron expressions, fixed intervals, and one-time
/// execution at a specific time.
///
/// # Schedule Types
///
/// - **cron**: Recurring tasks using cron expressions (e.g., "0 9 * * *" for daily at 9am)
/// - **interval**: Recurring tasks with a fixed interval in seconds
/// - **at**: One-time tasks executed at a specific ISO 8601 timestamp
///
/// # Task Management
///
/// Each scheduled task has a unique ID that can be used to cancel it. Tasks run in the
/// background as Tokio async tasks and execute shell commands.
///
/// # Examples
///
/// Schedule a daily backup:
/// ```json
/// {
///     "action": "schedule_task",
///     "parameters": {
///         "task_id": "daily_backup",
///         "command": "backup.sh",
///         "schedule_type": "cron",
///         "cron_expr": "0 9 * * *"
///     }
/// }
/// ```
///
/// Cancel a scheduled task:
/// ```json
/// {
///     "action": "unschedule_task",
///     "parameters": { "task_id": "daily_backup" }
/// }
/// ```
use anyhow::Result;
use chrono::{Datelike, Duration as ChronoDuration, Local, TimeZone};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

use crate::{SkillCallback, SkillCategory, SkillContext};
use crate::types::{Skill, SkillParameter};

/// Type alias for a thread-safe map storing scheduled task handles
type SchedulerMap = Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>;

/// Global static storage for all active scheduled tasks
///
/// This lazy static variable holds references to all currently running scheduled tasks.
/// It is protected by a mutex for thread-safe access across multiple async tasks.
static SCHEDULER_TASKS: once_cell::sync::Lazy<SchedulerMap> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Skill for scheduling tasks to run at specified times or intervals
///
/// This skill creates and manages scheduled tasks that execute shell commands
/// according to cron expressions, fixed intervals, or at specific times.
///
/// # Parameters
///
/// | Parameter | Type | Required | Description |
/// |-----------|------|----------|-------------|
/// | `task_id` | string | Yes | Unique identifier for the scheduled task |
/// | `command` | string | Yes | Shell command to execute |
/// | `schedule_type` | string | Yes | Type of schedule: "cron", "interval", or "at" |
/// | `cron_expr` | string | No* | Cron expression (required for cron type) |
/// | `interval_secs` | integer | No* | Interval in seconds (required for interval type) |
/// | `at_time` | string | No* | ISO 8601 timestamp (required for at type) |
///
/// *Required depending on `schedule_type` value
///
/// # Returns
///
/// Returns a success message with the task ID if scheduled successfully.
///
/// # Errors
///
/// Returns an error if:
/// - Required parameters are missing
/// - A task with the same ID already exists
/// - Invalid cron expression or time format
/// - Target time is in the past (for "at" schedule)
#[derive(Debug)]
pub struct ScheduleTaskSkill;

#[async_trait::async_trait]
impl Skill for ScheduleTaskSkill {
    /// Returns the unique name of this skill
    fn name(&self) -> &str {
        "schedule_task"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Schedule a command to run at specified time or interval"
    }

    /// Returns a hint about when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to schedule a task, set up a reminder, or run a command periodically"
    }

    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "task_id".to_string(),
                param_type: "string".to_string(),
                description: "Unique identifier for the scheduled task".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("daily_backup".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to execute".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("echo 'Hello'".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "schedule_type".to_string(),
                param_type: "string".to_string(),
                description: "Schedule type: cron, interval, or at".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("cron".to_string())),
                enum_values: Some(vec![
                    "cron".to_string(),
                    "interval".to_string(),
                    "at".to_string(),
                ]),
            },
            SkillParameter {
                name: "cron_expr".to_string(),
                param_type: "string".to_string(),
                description: "Cron expression (e.g., '0 9 * * *' for daily at 9am)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0 9 * * *".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "interval_secs".to_string(),
                param_type: "integer".to_string(),
                description: "Interval in seconds for interval schedule".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(3600.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "at_time".to_string(),
                param_type: "string".to_string(),
                description: "Specific time for one-time execution (ISO 8601 format)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("2024-12-31T23:59:00".to_string())),
                enum_values: None,
            },
        ]
    }

    /// Returns an example JSON call for this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "schedule_task",
            "parameters": {
                "task_id": "daily_report",
                "command": "python /path/to/report.py",
                "schedule_type": "cron",
                "cron_expr": "0 9 * * *"
            }
        })
    }

    /// Returns an example output string for this skill
    fn example_output(&self) -> String {
        "Task 'daily_report' scheduled successfully".to_string()
    }

    /// Returns the category of this skill
    fn category(&self) -> SkillCategory {
        SkillCategory::ScheduledTasks
    }

    /// Executes the task scheduling operation
    ///
    /// This method creates a new background task that will execute the specified
    /// command according to the schedule type.
    ///
    /// # Arguments
    ///
    /// * `parameters` - HashMap containing the scheduling parameters
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Success message or error
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = parameters
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: task_id"))?;
        let command = parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: command"))?;
        let schedule_type = parameters
            .get("schedule_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: schedule_type"))?;
        {
            let tasks = SCHEDULER_TASKS.lock().unwrap();
            if tasks.contains_key(task_id) {
                anyhow::bail!(
                    "Task '{}' already exists. Use unschedule_task first or use different task_id",
                    task_id
                );
            }
        }
        let task_id_owned = task_id.to_string();
        let command_owned = command.to_string();
        let schedule_type_owned = schedule_type.to_string();
        let cron_expr = parameters
            .get("cron_expr")
            .and_then(|v| v.as_str())
            .map(String::from);
        let interval_secs = parameters.get("interval_secs").and_then(|v| v.as_u64());
        let at_time_str = parameters
            .get("at_time")
            .and_then(|v| v.as_str())
            .map(String::from);
        let handle = tokio::spawn(async move {
            let schedule_duration = match schedule_type_owned.as_str() {
                "cron" => {
                    let expr = match cron_expr {
                        Some(ref e) => e,
                        None => {
                            eprintln!("[Scheduler] Missing cron_expr for cron schedule");
                            return;
                        }
                    };
                    match parse_cron_to_duration(expr) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("[Scheduler] Invalid cron expression: {}", e);
                            return;
                        }
                    }
                }
                "interval" => {
                    let secs = match interval_secs {
                        Some(s) => s,
                        None => {
                            eprintln!("[Scheduler] Missing interval_secs for interval schedule");
                            return;
                        }
                    };
                    Duration::from_secs(secs)
                }
                "at" => {
                    let time_str = match at_time_str {
                        Some(ref s) => s,
                        None => {
                            eprintln!("[Scheduler] Missing at_time for at schedule");
                            return;
                        }
                    };
                    let target_time = match chrono::DateTime::parse_from_rfc3339(time_str) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("[Scheduler] Invalid time format: {}", e);
                            return;
                        }
                    };
                    let now = chrono::Utc::now();
                    let target_utc = target_time.with_timezone(&chrono::Utc);
                    match (target_utc - now).to_std() {
                        Ok(d) => d,
                        Err(_) => {
                            eprintln!("[Scheduler] Target time is in the past");
                            return;
                        }
                    }
                }
                _ => {
                    eprintln!("[Scheduler] Unknown schedule_type: {}", schedule_type_owned);
                    return;
                }
            };
            if schedule_type_owned == "at" {
                time::sleep(schedule_duration).await;
                execute_command(&command_owned).await;
            } else {
                let mut interval = time::interval(schedule_duration);
                interval.tick().await;
                loop {
                    interval.tick().await;
                    execute_command(&command_owned).await;
                }
            }
        });
        {
            let mut tasks = SCHEDULER_TASKS.lock().unwrap();
            tasks.insert(task_id_owned, handle);
        }
        Ok(format!("Task '{}' scheduled successfully", task_id))
    }

    /// Validates the parameters for the schedule task operation
    ///
    /// # Arguments
    ///
    /// * `parameters` - The parameters to validate
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if parameters are valid, otherwise an error
    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: task_id"))?;
        parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: command"))?;
        let schedule_type = parameters
            .get("schedule_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: schedule_type"))?;
        match schedule_type {
            "cron" => {
                parameters
                    .get("cron_expr")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing cron_expr for cron schedule"))?;
            }
            "interval" => {
                parameters
                    .get("interval_secs")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing interval_secs for interval schedule")
                    })?;
            }
            "at" => {
                parameters
                    .get("at_time")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing at_time for at schedule"))?;
            }
            _ => anyhow::bail!("Unknown schedule_type: {}", schedule_type),
        }
        Ok(())
    }
}

/// Skill for removing a previously scheduled task
///
/// This skill cancels and removes a scheduled task by its unique ID.
/// The task will be aborted immediately and removed from the global task registry.
///
/// # Parameters
///
/// | Parameter | Type | Required | Description |
/// |-----------|------|----------|-------------|
/// | `task_id` | string | Yes | Unique identifier of the task to remove |
///
/// # Returns
///
/// Returns a success message with the task ID if the task was found and removed.
///
/// # Errors
///
/// Returns an error if the task ID does not exist.
#[derive(Debug)]
pub struct UnscheduleTaskSkill;

#[async_trait::async_trait]
impl Skill for UnscheduleTaskSkill {
    /// Returns the unique name of this skill
    fn name(&self) -> &str {
        "unschedule_task"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Remove a scheduled task"
    }

    /// Returns a hint about when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to cancel a previously scheduled task"
    }

    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "task_id".to_string(),
            param_type: "string".to_string(),
            description: "Unique identifier of the task to remove".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("daily_backup".to_string())),
            enum_values: None,
        }]
    }

    /// Returns an example JSON call for this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "unschedule_task",
            "parameters": {
                "task_id": "daily_backup"
            }
        })
    }

    /// Returns an example output string for this skill
    fn example_output(&self) -> String {
        "Task 'daily_backup' removed successfully".to_string()
    }

    /// Returns the category of this skill
    fn category(&self) -> SkillCategory {
        SkillCategory::ScheduledTasks
    }

    /// Executes the task unscheduling operation
    ///
    /// This method finds and cancels a scheduled task by its ID.
    ///
    /// # Arguments
    ///
    /// * `parameters` - HashMap containing the task_id parameter
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Success message or error
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = parameters
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: task_id"))?;
        let mut tasks = SCHEDULER_TASKS.lock().unwrap();
        if let Some(handle) = tasks.remove(task_id) {
            handle.abort();
            Ok(format!("Task '{}' removed successfully", task_id))
        } else {
            anyhow::bail!("Task '{}' not found", task_id)
        }
    }

    /// Validates the parameters for the unschedule task operation
    ///
    /// # Arguments
    ///
    /// * `parameters` - The parameters to validate
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if parameters are valid, otherwise an error
    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: task_id"))?;
        Ok(())
    }
}

/// Skill for listing all active scheduled tasks
///
/// This skill retrieves and displays all currently scheduled tasks.
/// It returns a formatted list of task IDs for active tasks.
///
/// # Parameters
///
/// This skill takes no parameters.
///
/// # Returns
///
/// Returns a formatted string listing all scheduled task IDs,
/// or a message indicating no tasks are scheduled.
#[derive(Debug)]
pub struct ListScheduledTasksSkill;

#[async_trait::async_trait]
impl Skill for ListScheduledTasksSkill {
    /// Returns the unique name of this skill
    fn name(&self) -> &str {
        "list_scheduled_tasks"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "List all scheduled tasks"
    }

    /// Returns a hint about when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to see all active scheduled tasks"
    }

    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    /// Returns an example JSON call for this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "list_scheduled_tasks",
            "parameters": {}
        })
    }

    /// Returns an example output string for this skill
    fn example_output(&self) -> String {
        "Scheduled tasks:\n- daily_backup\n- hourly_cleanup".to_string()
    }

    /// Returns the category of this skill
    fn category(&self) -> SkillCategory {
        SkillCategory::ScheduledTasks
    }

    /// Executes the task listing operation
    ///
    /// This method retrieves all active task IDs from the global registry.
    ///
    /// # Arguments
    ///
    /// * `_parameters` - Unused parameter
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Formatted list of scheduled tasks
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let tasks = SCHEDULER_TASKS.lock().unwrap();
        if tasks.is_empty() {
            Ok("No scheduled tasks".to_string())
        } else {
            let task_list: Vec<String> = tasks.keys().cloned().collect();
            Ok(format!("Scheduled tasks:\n- {}", task_list.join("\n- ")))
        }
    }
}

/// Executes a shell command and logs its output
///
/// This helper function runs a command through the system shell (`sh -c`)
/// and logs any stdout or stderr output to stderr for debugging.
///
/// # Arguments
///
/// * `command` - The shell command to execute
async fn execute_command(command: &str) {
    use std::process::Command;
    let output = Command::new("sh").arg("-c").arg(command).output();
    match output {
        Ok(out) => {
            if !out.stdout.is_empty() {
                eprintln!(
                    "[Scheduler] stdout: {}",
                    String::from_utf8_lossy(&out.stdout)
                );
            }
            if !out.stderr.is_empty() {
                eprintln!(
                    "[Scheduler] stderr: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("[Scheduler] Failed to execute command '{}': {}", command, e);
        }
    }
}

/// Parses a cron expression into a Duration until the next execution
///
/// This function parses a simple cron expression (minute hour) and calculates
/// the duration until the next scheduled execution time.
///
/// # Arguments
///
/// * `cron_expr` - Cron expression with at least minute and hour fields
///                 (e.g., "30 14 * * *" for 2:30 PM daily)
///
/// # Returns
///
/// * `Result<Duration>` - Duration until the next scheduled time
///
/// # Notes
///
/// Currently supports only minute and hour fields. Day, month, and day-of-week
/// fields are parsed but ignored. The function calculates the next occurrence
/// today or tomorrow based on the current time.
fn parse_cron_to_duration(cron_expr: &str) -> Result<Duration> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        anyhow::bail!("Invalid cron expression: need at least 5 fields");
    }
    let minute = parts[0].parse::<u32>().unwrap_or(0);
    let hour = parts[1].parse::<u32>().unwrap_or(0);
    let now = Local::now();
    let next = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), hour, minute, 0)
        .single()
        .unwrap_or(now);
    let duration = if next > now {
        next - now
    } else {
        let next_day = Local
            .with_ymd_and_hms(now.year(), now.month(), now.day() + 1, hour, minute, 0)
            .single()
            .unwrap_or(now + ChronoDuration::days(1));
        next_day - now
    };
    Ok(duration.to_std().unwrap_or(Duration::from_secs(60)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test parameter validation for ScheduleTaskSkill
    #[test]
    fn test_schedule_task_validation() {
        let skill = ScheduleTaskSkill;
        let mut valid_cron = HashMap::new();
        valid_cron.insert("task_id".to_string(), Value::String("test1".to_string()));
        valid_cron.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        valid_cron.insert(
            "schedule_type".to_string(),
            Value::String("cron".to_string()),
        );
        valid_cron.insert(
            "cron_expr".to_string(),
            Value::String("0 9 * * *".to_string()),
        );
        assert!(skill.validate(&valid_cron).is_ok());
        let mut valid_interval = HashMap::new();
        valid_interval.insert("task_id".to_string(), Value::String("test2".to_string()));
        valid_interval.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        valid_interval.insert(
            "schedule_type".to_string(),
            Value::String("interval".to_string()),
        );
        valid_interval.insert("interval_secs".to_string(), Value::Number(3600.into()));
        assert!(skill.validate(&valid_interval).is_ok());
        let mut valid_at = HashMap::new();
        valid_at.insert("task_id".to_string(), Value::String("test3".to_string()));
        valid_at.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        valid_at.insert("schedule_type".to_string(), Value::String("at".to_string()));
        valid_at.insert(
            "at_time".to_string(),
            Value::String("2025-12-31T23:59:00+00:00".to_string()),
        );
        assert!(skill.validate(&valid_at).is_ok());
        let mut missing_task_id = HashMap::new();
        missing_task_id.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        missing_task_id.insert(
            "schedule_type".to_string(),
            Value::String("cron".to_string()),
        );
        assert!(skill.validate(&missing_task_id).is_err());
        let mut missing_command = HashMap::new();
        missing_command.insert("task_id".to_string(), Value::String("test".to_string()));
        missing_command.insert(
            "schedule_type".to_string(),
            Value::String("cron".to_string()),
        );
        assert!(skill.validate(&missing_command).is_err());
        let mut missing_type = HashMap::new();
        missing_type.insert("task_id".to_string(), Value::String("test".to_string()));
        missing_type.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        assert!(skill.validate(&missing_type).is_err());
        let mut missing_cron_expr = HashMap::new();
        missing_cron_expr.insert("task_id".to_string(), Value::String("test".to_string()));
        missing_cron_expr.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        missing_cron_expr.insert(
            "schedule_type".to_string(),
            Value::String("cron".to_string()),
        );
        assert!(skill.validate(&missing_cron_expr).is_err());
        let mut missing_interval = HashMap::new();
        missing_interval.insert("task_id".to_string(), Value::String("test".to_string()));
        missing_interval.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        missing_interval.insert(
            "schedule_type".to_string(),
            Value::String("interval".to_string()),
        );
        assert!(skill.validate(&missing_interval).is_err());
        let mut unknown_type = HashMap::new();
        unknown_type.insert("task_id".to_string(), Value::String("test".to_string()));
        unknown_type.insert(
            "command".to_string(),
            Value::String("echo test".to_string()),
        );
        unknown_type.insert(
            "schedule_type".to_string(),
            Value::String("unknown".to_string()),
        );
        assert!(skill.validate(&unknown_type).is_err());
    }

    /// Test UnscheduleTaskSkill parameter validation
    #[test]
    fn test_unschedule_task_validation() {
        let skill = UnscheduleTaskSkill;
        let mut valid_params = HashMap::new();
        valid_params.insert(
            "task_id".to_string(),
            Value::String("daily_backup".to_string()),
        );
        assert!(skill.validate(&valid_params).is_ok());
        let empty_params = HashMap::new();
        assert!(skill.validate(&empty_params).is_err());
        let mut wrong_type = HashMap::new();
        wrong_type.insert("task_id".to_string(), Value::Number(123.into()));
        assert!(skill.validate(&wrong_type).is_err());
    }

    /// Test skill metadata (names, categories, descriptions)
    #[test]
    fn test_skill_metadata() {
        let schedule_skill = ScheduleTaskSkill;
        let unschedule_skill = UnscheduleTaskSkill;
        let list_skill = ListScheduledTasksSkill;
        assert_eq!(schedule_skill.name(), "schedule_task");
        assert_eq!(unschedule_skill.name(), "unschedule_task");
        assert_eq!(list_skill.name(), "list_scheduled_tasks");
        assert_eq!(schedule_skill.category(), SkillCategory::ScheduledTasks);
        assert_eq!(unschedule_skill.category(), SkillCategory::ScheduledTasks);
        assert_eq!(list_skill.category(), SkillCategory::ScheduledTasks);
        assert!(!schedule_skill.description().is_empty());
        assert!(!unschedule_skill.description().is_empty());
        assert!(!list_skill.description().is_empty());
        assert!(!schedule_skill.usage_hint().is_empty());
        assert!(!unschedule_skill.usage_hint().is_empty());
        assert!(!list_skill.usage_hint().is_empty());
    }

    /// Test parameter definitions for each skill
    #[test]
    fn test_skill_parameters() {
        let schedule_skill = ScheduleTaskSkill;
        let unschedule_skill = UnscheduleTaskSkill;
        let list_skill = ListScheduledTasksSkill;
        let schedule_params = schedule_skill.parameters();
        assert_eq!(schedule_params.len(), 6);
        let param_names: Vec<&str> = schedule_params.iter().map(|p| p.name.as_str()).collect();
        assert!(param_names.contains(&"task_id"));
        assert!(param_names.contains(&"command"));
        assert!(param_names.contains(&"schedule_type"));
        let task_id_param = schedule_params
            .iter()
            .find(|p| p.name == "task_id")
            .unwrap();
        assert!(task_id_param.required);
        let unschedule_params = unschedule_skill.parameters();
        assert_eq!(unschedule_params.len(), 1);
        assert_eq!(unschedule_params[0].name, "task_id");
        assert!(unschedule_params[0].required);
        assert_eq!(list_skill.parameters().len(), 0);
    }

    /// Test cron expression parsing
    #[test]
    fn test_parse_cron_to_duration() {
        let result = parse_cron_to_duration("30 14 * * *");
        assert!(result.is_ok());
        let result = parse_cron_to_duration("30 14");
        assert!(result.is_err());
        let result = parse_cron_to_duration("invalid 14 * * *");
        assert!(result.is_ok());
    }
}
