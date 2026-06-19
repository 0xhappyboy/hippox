//! Audio volume down skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::volume_down;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct AudioControlVolumeDownDriver;

#[async_trait::async_trait]
impl Driver for AudioControlVolumeDownDriver {
    fn name(&self) -> &str {
        "audio_control_volume_down"
    }

    fn description(&self) -> &str {
        "Decrease system volume by a specified amount"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to decrease the volume. Default delta is 10 if not specified."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "delta".to_string(),
            param_type: "integer".to_string(),
            description: "Amount to decrease by (0-100)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(20.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_volume_down",
            "parameters": {
                "delta": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Volume decreased by 10%".to_string()
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
        volume_down(delta)?;
        Ok(format!("Volume decreased by {}%", delta))
    }
}
