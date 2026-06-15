use anyhow::Result;
use chrono::{Duration, Local, Utc};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct DateTimeSkill;

#[async_trait::async_trait]
impl Skill for DateTimeSkill {
    fn name(&self) -> &str {
        "datetime"
    }

    fn description(&self) -> &str {
        "Get current date and time, or perform timezone conversions"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks for the current time, date, timestamp, or wants to convert between timezones"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Operation type: now, utc, timestamp, or convert".to_string(),
                required: false,
                default: Some(Value::String("now".to_string())),
                example: Some(Value::String("now".to_string())),
                enum_values: Some(vec![
                    "now".to_string(),
                    "utc".to_string(),
                    "timestamp".to_string(),
                    "convert".to_string(),
                ]),
            },
            SkillParameter {
                name: "timezone".to_string(),
                param_type: "string".to_string(),
                description: "Timezone like 'Asia/Shanghai', 'America/New_York', 'UTC' (for convert operation)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Asia/Shanghai".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format like '%Y-%m-%d %H:%M:%S'".to_string(),
                required: false,
                default: Some(Value::String("%Y-%m-%d %H:%M:%S".to_string())),
                example: Some(Value::String("%Y-%m-%d %H:%M:%S".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "datetime",
            "parameters": {
                "operation": "now"
            }
        })
    }

    fn example_output(&self) -> String {
        "2024-01-15 14:30:25".to_string()
    }

    fn category(&self) -> &str {
        "time"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("now");
        let timezone = parameters.get("timezone").and_then(|v| v.as_str());
        let format_str = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("%Y-%m-%d %H:%M:%S");
        match operation {
            "utc" => {
                let now = Utc::now();
                Ok(now.format(format_str).to_string())
            }
            "timestamp" => {
                let now = Utc::now();
                Ok(now.timestamp().to_string())
            }
            "convert" => {
                if let Some(tz) = timezone {
                    convert_timezone(tz, format_str)
                } else {
                    Ok("Missing timezone for conversion".to_string())
                }
            }
            _ => {
                let now = Local::now();
                Ok(now.format(format_str).to_string())
            }
        }
    }

    fn validate(&self, _parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}

fn convert_timezone(timezone: &str, format_str: &str) -> Result<String> {
    let offset = match timezone.to_lowercase().as_str() {
        "utc" | "gmt" => 0,
        "asia/shanghai" | "asia/beijing" | "cst" => 8,
        "asia/tokyo" | "jst" => 9,
        "america/new_york" | "est" => -5,
        "america/los_angeles" | "pst" => -8,
        "europe/london" | "bst" | "gmt" => 0,
        "europe/paris" | "cet" => 1,
        "asia/dubai" | "gst" => 4,
        "asia/singapore" | "sgt" => 8,
        "australia/sydney" | "aest" => 11,
        _ => {
            if let Some(offset_val) = parse_offset(timezone) {
                offset_val
            } else {
                return Ok(format!("Unknown timezone: {}", timezone));
            }
        }
    };
    let now = Utc::now();
    let dt = now + Duration::hours(offset.into());
    Ok(dt.format(format_str).to_string())
}

fn parse_offset(tz: &str) -> Option<i32> {
    let tz_lower = tz.to_lowercase();
    if tz_lower.starts_with("utc") || tz_lower.starts_with("gmt") {
        let rest = &tz_lower[3..];
        if let Ok(offset) = rest.parse::<i32>() {
            return Some(offset);
        }
        if rest.starts_with('+') {
            if let Ok(offset) = rest[1..].parse::<i32>() {
                return Some(offset);
            }
        }
        if rest.starts_with('-') {
            if let Ok(offset) = rest[1..].parse::<i32>() {
                return Some(-offset);
            }
        }
    }
    None
}
