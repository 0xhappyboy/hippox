// mouse_control/mouse_control_get_cursor.rs
//! Mouse get cursor type skill

use super::common::get_cursor_type;
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
pub struct MouseControlGetCursorDriver;

#[async_trait::async_trait]
impl Driver for MouseControlGetCursorDriver {
    fn name(&self) -> &str {
        "mouse_control_get_cursor"
    }

    fn description(&self) -> &str {
        "Get the current cursor type (arrow, hand, wait, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to determine what kind of cursor is currently displayed, which can indicate UI state."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_get_cursor"
        })
    }

    fn example_output(&self) -> String {
        "Cursor type: arrow".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Mouse
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let cursor_type = get_cursor_type()?;
        Ok(format!("Cursor type: {}", cursor_type))
    }
}
