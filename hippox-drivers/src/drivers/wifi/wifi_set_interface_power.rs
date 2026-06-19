//! WiFi set interface power skill - set power saving mode

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
pub struct WifiSetInterfacePowerDriver;

#[async_trait::async_trait]
impl Driver for WifiSetInterfacePowerDriver {
    fn name(&self) -> &str {
        "wifi_set_interface_power"
    }

    fn description(&self) -> &str {
        "Set wireless interface power saving mode (performance or power saving)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to balance between performance and battery life. 'performance' mode keeps WiFi at full power, 'powersave' reduces power consumption."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "mode".to_string(),
                param_type: "string".to_string(),
                description: "Power mode: 'performance' or 'powersave'".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("performance".to_string())),
                enum_values: Some(vec!["performance".to_string(), "powersave".to_string()]),
            },
            DriverParameter {
                name: "interface".to_string(),
                param_type: "string".to_string(),
                description: "Interface name (default: auto-detect)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("wlan0".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_set_interface_power",
            "parameters": {
                "mode": "performance"
            }
        })
    }

    fn example_output(&self) -> String {
        "WiFi interface power mode set to: performance".to_string()
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
        let mode = parameters
            .get("mode")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mode' parameter"))?;

        let interface = parameters
            .get("interface")
            .and_then(|v| v.as_str())
            .unwrap_or("wlan0");

        #[cfg(target_os = "linux")]
        {
            let power_value = match mode {
                "performance" => "off",
                "powersave" => "on",
                _ => anyhow::bail!("Invalid mode: {}", mode),
            };

            Command::new("iwconfig")
                .args([interface, "power", power_value])
                .output()?;
        }

        #[cfg(target_os = "windows")]
        {
            // Windows power settings via powercfg
            let scheme = match mode {
                "performance" => "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c", // High performance
                "powersave" => "a1841308-3541-4fab-bc81-f71556f20b4a",   // Power saver
                _ => anyhow::bail!("Invalid mode: {}", mode),
            };
            Command::new("powercfg")
                .args(["/setactive", scheme])
                .output()?;
        }

        Ok(format!("WiFi interface power mode set to: {}", mode))
    }
}
