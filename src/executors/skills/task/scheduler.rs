use anyhow::Result;
use chrono::{Datelike, Duration as ChronoDuration, Local, TimeZone};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

use crate::executors::types::{Skill, SkillParameter};

type SchedulerMap = Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>;

static SCHEDULER_TASKS: once_cell::sync::Lazy<SchedulerMap> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug)]
pub struct ScheduleTaskSkill;

#[async_trait::async_trait]
impl Skill for ScheduleTaskSkill {
    fn name(&self) -> &str {
        "schedule_task"
    }

    fn description(&self) -> &str {
        "Schedule a command to run at specified time or interval"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to schedule a task, set up a reminder, or run a command periodically"
    }

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

    fn example_output(&self) -> String {
        "Task 'daily_report' scheduled successfully".to_string()
    }

    fn category(&self) -> &str {
        "system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
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

#[derive(Debug)]
pub struct UnscheduleTaskSkill;

#[async_trait::async_trait]
impl Skill for UnscheduleTaskSkill {
    fn name(&self) -> &str {
        "unschedule_task"
    }

    fn description(&self) -> &str {
        "Remove a scheduled task"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to cancel a previously scheduled task"
    }

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

    fn example_call(&self) -> Value {
        json!({
            "action": "unschedule_task",
            "parameters": {
                "task_id": "daily_backup"
            }
        })
    }

    fn example_output(&self) -> String {
        "Task 'daily_backup' removed successfully".to_string()
    }

    fn category(&self) -> &str {
        "system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
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

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: task_id"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListScheduledTasksSkill;

#[async_trait::async_trait]
impl Skill for ListScheduledTasksSkill {
    fn name(&self) -> &str {
        "list_scheduled_tasks"
    }

    fn description(&self) -> &str {
        "List all scheduled tasks"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to see all active scheduled tasks"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "list_scheduled_tasks",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "Scheduled tasks:\n- daily_backup\n- hourly_cleanup".to_string()
    }

    fn category(&self) -> &str {
        "system"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let tasks = SCHEDULER_TASKS.lock().unwrap();
        if tasks.is_empty() {
            Ok("No scheduled tasks".to_string())
        } else {
            let task_list: Vec<String> = tasks.keys().cloned().collect();
            Ok(format!("Scheduled tasks:\n- {}", task_list.join("\n- ")))
        }
    }
}

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
