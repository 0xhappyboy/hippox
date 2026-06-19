//! Audio volume up skill

use super::common::volume_up;
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
pub struct AudioControlVolumeUpDriver;

#[async_trait::async_trait]
impl Driver for AudioControlVolumeUpDriver {
    fn name(&self) -> &str {
        "audio_control_volume_up"
    }

    fn description(&self) -> &str {
        "Increase system volume by a specified amount"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to increase the volume. Default delta is 10 if not specified."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "delta".to_string(),
            param_type: "integer".to_string(),
            description: "Amount to increase by (0-100)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(20.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_volume_up",
            "parameters": {
                "delta": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Volume increased by 10%".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Audio
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let delta = parameters
            .get("delta")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;

        volume_up(delta)?;

        Ok(format!("Volume increased by {}%", delta))
    }
}
