//! Display brightness get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::get_brightness;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct DisplayControlBrightnessGetDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlBrightnessGetDriver {
    fn name(&self) -> &str {
        "display_control_brightness_get"
    }

    fn description(&self) -> &str {
        "Get the current display brightness level (laptops only)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the screen brightness (0-100). Works on laptops, may not work on desktops."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_brightness_get"
        })
    }

    fn example_output(&self) -> String {
        "Display brightness: 75%".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Display
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let brightness = get_brightness()?;

        Ok(format!("Display brightness: {}%", brightness))
    }
}
