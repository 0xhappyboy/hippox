//! Window list skill

use super::common::list_windows;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowControlListDriver;

#[async_trait::async_trait]
impl Driver for WindowControlListDriver {
    fn name(&self) -> &str {
        "window_control_list"
    }

    fn description(&self) -> &str {
        "List all open windows"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what windows are currently open"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_list"
        })
    }

    fn example_output(&self) -> String {
        "Found 5 windows:\n1. 微信 (WeChat.exe, PID: 12345)\n2. Visual Studio Code (Code.exe, PID: 23456)\n3. Chrome (chrome.exe, PID: 34567)".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Window
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let windows = list_windows()?;

        if windows.is_empty() {
            return Ok("No windows found".to_string());
        }

        let mut result = format!("Found {} windows:\n", windows.len());
        for (i, window) in windows.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} ({} [{}], PID: {})\n",
                i + 1,
                window.title,
                window.process_name,
                if window.is_minimized {
                    "minimized"
                } else {
                    "visible"
                },
                window.pid
            ));
        }

        Ok(result)
    }
}
