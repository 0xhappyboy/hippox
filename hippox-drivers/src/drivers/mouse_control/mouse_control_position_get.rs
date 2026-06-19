// mouse_control/mouse_control_position_get.rs
//! Mouse position get skill

use super::common::get_mouse_position;
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
pub struct MouseControlPositionGetDriver;

#[async_trait::async_trait]
impl Driver for MouseControlPositionGetDriver {
    fn name(&self) -> &str {
        "mouse_control_position_get"
    }

    fn description(&self) -> &str {
        "Get the current mouse cursor position"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to read the current coordinates of the mouse cursor on screen"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_position_get"
        })
    }

    fn example_output(&self) -> String {
        "Mouse position: x=500, y=300".to_string()
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
        let pos = get_mouse_position()?;
        Ok(format!("Mouse position: x={}, y={}", pos.x, pos.y))
    }
}
