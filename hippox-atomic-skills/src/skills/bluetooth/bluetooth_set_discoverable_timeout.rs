//! Bluetooth set discoverable timeout skill - set how long device stays discoverable

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct BluetoothSetDiscoverableTimeoutSkill;

#[async_trait::async_trait]
impl Skill for BluetoothSetDiscoverableTimeoutSkill {
    fn name(&self) -> &str {
        "bluetooth_set_discoverable_timeout"
    }

    fn description(&self) -> &str {
        "Set how long the Bluetooth adapter remains discoverable (0 = unlimited)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control the discoverable duration. Set to 0 for unlimited, or a positive number for limited time."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Discoverable timeout in seconds (0 = unlimited)".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(60.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_set_discoverable_timeout",
            "parameters": {
                "timeout_secs": 60
            }
        })
    }

    fn example_output(&self) -> String {
        "Discoverable timeout set to 60 seconds".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'timeout_secs' parameter"))?;
        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["discoverable-timeout", &timeout.to_string()])
                .output()?;
        }
        if timeout == 0 {
            Ok("Discoverable timeout set to unlimited".to_string())
        } else {
            Ok(format!("Discoverable timeout set to {} seconds", timeout))
        }
    }
}
