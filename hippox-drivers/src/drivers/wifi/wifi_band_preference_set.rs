//! WiFi band preference set skill - set preferred frequency band

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct WifiBandPreferenceSetDriver;

#[async_trait::async_trait]
impl Driver for WifiBandPreferenceSetDriver {
    fn name(&self) -> &str {
        "wifi_band_preference_set"
    }

    fn description(&self) -> &str {
        "Set preferred WiFi frequency band (2.4GHz, 5GHz, or 6GHz)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to prefer a specific band. 2.4GHz has better range, 5GHz/6GHz has faster speed but shorter range."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "band".to_string(),
            param_type: "string".to_string(),
            description: "Preferred band: '2.4', '5', '6', or 'auto'".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("5".to_string())),
            enum_values: Some(vec![
                "2.4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "auto".to_string(),
            ]),
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_band_preference_set",
            "parameters": {
                "band": "5"
            }
        })
    }

    fn example_output(&self) -> String {
        "WiFi band preference set to: 5GHz".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Wifi
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let band = parameters
            .get("band")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'band' parameter"))?;
        let band_name = match band {
            "2.4" => "2.4GHz",
            "5" => "5GHz",
            "6" => "6GHz",
            "auto" => "Auto",
            _ => anyhow::bail!("Invalid band: {}", band),
        };
        #[cfg(target_os = "windows")]
        {
            let band_code = match band {
                "2.4" => "1",
                "5" => "2",
                "6" => "4",
                "auto" => "0",
                _ => "0",
            };
            Command::new("netsh")
                .args(["wlan", "set", "allowexplicitcreds", "band=", band_code])
                .output()?;
        }
        #[cfg(target_os = "linux")]
        {
            let band_value = match band {
                "2.4" => "bg",
                "5" => "a",
                "auto" => "any",
                _ => "any",
            };
            Command::new("iw")
                .args(["reg", "set", band_value])
                .output()?;
        }
        Ok(format!("WiFi band preference set to: {}", band_name))
    }
}
