//! WiFi ping gateway skill - test connection to gateway

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::{get_default_gateway, ping_gateway};

#[derive(Debug)]
pub struct WifiPingGatewaySkill;

#[async_trait::async_trait]
impl Skill for WifiPingGatewaySkill {
    fn name(&self) -> &str {
        "wifi_ping_gateway"
    }

    fn description(&self) -> &str {
        "Ping the default gateway to test WiFi connection quality"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to test the connection to your router. High latency or packet loss indicates WiFi issues."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of ping packets to send (default: 4)".to_string(),
                required: false,
                default: Some(Value::Number(4.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_ping_gateway",
            "parameters": {
                "count": 4
            }
        })
    }

    fn example_output(&self) -> String {
        "Gateway: 192.168.1.1, Ping: 2.5ms (0% loss)".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let gateway = get_default_gateway()?;
        let (success, avg_time) = ping_gateway(&gateway)?;
        
        if success {
            Ok(format!("Gateway: {}, Ping: {}ms", gateway, avg_time))
        } else {
            Ok(format!("Gateway: {}, Ping failed (timeout or unreachable)", gateway))
        }
    }
}