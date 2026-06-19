// mouse_control/mouse_control_press.rs
//! Mouse press skill

use super::common::{MouseButton, mouse_press};
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
pub struct MouseControlPressDriver;

#[async_trait::async_trait]
impl Driver for MouseControlPressDriver {
    fn name(&self) -> &str {
        "mouse_control_press"
    }

    fn description(&self) -> &str {
        "Press and hold a mouse button (without releasing)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to hold down a mouse button. Use 'mouse_control_release' to release it."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "button".to_string(),
                param_type: "string".to_string(),
                description: "Mouse button: 'left', 'right', or 'middle'".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("left".to_string())),
                enum_values: Some(vec![
                    "left".to_string(),
                    "right".to_string(),
                    "middle".to_string(),
                ]),
            },
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to press at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to press at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_press",
            "parameters": {
                "button": "left",
                "x": 500,
                "y": 300
            }
        })
    }

    fn example_output(&self) -> String {
        "Mouse button left pressed".to_string()
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
        let button_str = parameters
            .get("button")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'button' parameter"))?;

        let button = match button_str {
            "left" => MouseButton::Left,
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => anyhow::bail!("Unknown button: {}", button_str),
        };

        let x = parameters
            .get("x")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let y = parameters
            .get("y")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        let (press_x, press_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = super::common::get_mouse_position()?;
            (pos.x, pos.y)
        };

        mouse_press(button, press_x, press_y)?;

        Ok(format!("Mouse button {} pressed", button_str))
    }
}
