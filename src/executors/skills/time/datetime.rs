use anyhow::Result;
use chrono::{Local, Utc};
use serde_json::Value;
use std::collections::HashMap;

use crate::executors::types::Skill;

#[derive(Debug)]
pub struct DateTimeSkill;

#[async_trait::async_trait]
impl Skill for DateTimeSkill {
    fn name(&self) -> &str {
        "datetime"
    }

    fn description(&self) -> &str {
        "Get current date and time or convert timezones"
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
                // Default: local time
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
            // Try to parse as UTC offset like "UTC+8"
            if let Some(offset_val) = parse_offset(timezone) {
                offset_val
            } else {
                return Ok(format!("Unknown timezone: {}", timezone));
            }
        }
    };
    let now = Utc::now();
    let dt = now + chrono::Duration::hours(offset.into());
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
