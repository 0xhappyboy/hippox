use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::{Disks, Networks, System, Users};

/// A skill for getting system time.
#[derive(Debug)]
pub struct OsGetTimeSkill;

#[async_trait::async_trait]
impl Skill for OsGetTimeSkill {
    fn name(&self) -> &str {
        "os_get_time"
    }

    fn description(&self) -> &str {
        "Get current system time and timezone"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check the current date, time, and timezone"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "format".to_string(),
            param_type: "string".to_string(),
            description: "Output format: full, date, time, timestamp".to_string(),
            required: false,
            default: Some(json!("full")),
            example: Some(json!("date")),
            enum_values: Some(vec![
                "full".to_string(),
                "date".to_string(),
                "time".to_string(),
                "timestamp".to_string(),
            ]),
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_time"
        })
    }

    fn example_output(&self) -> String {
        "Current time: 2024-01-15 14:30:45 (UTC+8)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Time
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("full");
        let now = chrono::Local::now();
        let tz = now.offset();
        match format {
            "date" => Ok(format!("Current date: {}", now.format("%Y-%m-%d"))),
            "time" => Ok(format!("Current time: {}", now.format("%H:%M:%S"))),
            "timestamp" => Ok(format!("Unix timestamp: {}", now.timestamp())),
            _ => Ok(format!(
                "Current time: {} ({})",
                now.format("%Y-%m-%d %H:%M:%S"),
                tz
            )),
        }
    }
}

/// A skill for setting system time (requires admin).
#[derive(Debug)]
pub struct OsSetTimeSkill;

#[async_trait::async_trait]
impl Skill for OsSetTimeSkill {
    fn name(&self) -> &str {
        "os_set_time"
    }

    fn description(&self) -> &str {
        "Set system time (requires administrator privileges)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to adjust system time"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "datetime".to_string(),
            param_type: "string".to_string(),
            description: "New datetime in format 'YYYY-MM-DD HH:MM:SS'".to_string(),
            required: true,
            default: None,
            example: Some(json!("2024-01-15 12:00:00")),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_set_time",
            "parameters": {
                "datetime": "2024-01-15 10:00:00"
            }
        })
    }

    fn example_output(&self) -> String {
        "System time updated successfully".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Time
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let datetime = parameters
            .get("datetime")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: datetime"))?;
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async(
                "powershell",
                &["-Command", &format!("Set-Date -Date '{}'", datetime)],
                None,
            )
            .await?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            exec_async("sudo", &["date", "-s", datetime], None).await?;
        }
        Ok("System time updated successfully".to_string())
    }
}
