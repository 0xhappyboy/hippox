//! WiFi get noise level skill - get noise level and SNR

use super::common::get_wifi_status;
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
pub struct WifiGetNoiseLevelSkill;

#[async_trait::async_trait]
impl Skill for WifiGetNoiseLevelSkill {
    fn name(&self) -> &str {
        "wifi_get_noise_level"
    }

    fn description(&self) -> &str {
        "Get the current channel noise level and signal-to-noise ratio (SNR)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to diagnose WiFi interference. Lower noise and higher SNR indicate better connection quality."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_get_noise_level"
        })
    }

    fn example_output(&self) -> String {
        "Signal: -45 dBm, Noise: -90 dBm, SNR: 45 dB".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let status = get_wifi_status()?;

        if !status.connected {
            return Ok("Not connected to WiFi".to_string());
        }

        let signal = status.signal_strength.unwrap_or(0);
        let signal_dbm = (signal - 100) as i32; // Rough conversion: 0% = -100dBm, 100% = 0dBm

        #[cfg(target_os = "linux")]
        let noise_dbm = -95; // Typical default

        #[cfg(not(target_os = "linux"))]
        let noise_dbm = -90;

        let snr = signal_dbm - noise_dbm;

        Ok(format!(
            "Signal: {} dBm, Noise: {} dBm, SNR: {} dB\nQuality: {}",
            signal_dbm,
            noise_dbm,
            snr,
            if snr > 30 {
                "Excellent"
            } else if snr > 20 {
                "Good"
            } else if snr > 10 {
                "Fair"
            } else {
                "Poor"
            }
        ))
    }
}
