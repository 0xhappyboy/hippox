//! WiFi analyze quality skill - analyze WiFi quality and recommend channel

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{get_wifi_status, scan_wifi_networks};
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiAnalyzeQualitySkill;

#[async_trait::async_trait]
impl Skill for WifiAnalyzeQualitySkill {
    fn name(&self) -> &str {
        "wifi_analyze_quality"
    }

    fn description(&self) -> &str {
        "Analyze WiFi quality and recommend the best channel to use"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to diagnose WiFi interference and get recommendations for improving connection quality."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_analyze_quality"
        })
    }

    fn example_output(&self) -> String {
        "Quality Analysis:\n- Score: 75/100\n- Current Channel: 6\n- Recommended Channel: 11\n- Recommendations: Switch to channel 11 to avoid interference from 3 nearby networks".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let status = get_wifi_status()?;
        let networks = scan_wifi_networks()?;
        let current_channel = status.channel.unwrap_or(6);
        // Count networks per channel
        let mut channel_counts: HashMap<u32, u32> = HashMap::new();
        for network in &networks {
            if let Some(channel) = network.channel {
                *channel_counts.entry(channel).or_insert(0) += 1;
            }
        }
        // Find best channel (least congested)
        let mut best_channel = current_channel;
        let mut min_count = u32::MAX;
        for channel in [1, 6, 11, 3, 8, 2, 4, 5, 7, 9, 10] {
            let count = *channel_counts.entry(channel).or_insert(0);
            if count < min_count {
                min_count = count;
                best_channel = channel;
            }
        }
        let signal = status.signal_strength.unwrap_or(0);
        let score = if signal > 80 {
            90
        } else if signal > 60 {
            70
        } else if signal > 40 {
            50
        } else {
            30
        };
        let mut recommendations = Vec::new();
        if best_channel != current_channel {
            recommendations.push(format!(
                "Switch to channel {} to avoid interference from {} nearby networks",
                best_channel, min_count
            ));
        }
        if signal < 50 {
            recommendations
                .push("Move closer to the router for better signal strength".to_string());
        }
        let mut result = format!(
            "Quality Analysis:\n- Score: {}/100\n- Current Channel: {}\n- Recommended Channel: {}\n- Interfering Networks: {}\n",
            score, current_channel, best_channel, min_count
        );
        if !recommendations.is_empty() {
            result.push_str("- Recommendations:\n");
            for rec in recommendations {
                result.push_str(&format!("  * {}\n", rec));
            }
        }
        Ok(result)
    }
}
