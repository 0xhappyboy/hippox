//! WiFi set interface power skill - set power saving mode
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for setting WiFi interface power mode
#[derive(Debug)]
pub struct WifiSetInterfacePowerDriver;
#[async_trait::async_trait]
impl Driver for WifiSetInterfacePowerDriver {
    fn name(&self) -> &str {
        return "wifi_set_interface_power";
    }
    fn description(&self) -> &str {
        return "Set wireless interface power saving mode (performance or power saving)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to balance between performance and battery life. 'performance' mode keeps WiFi at full power, 'powersave' reduces power consumption.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_set_interface_power",
            "parameters": {
                "mode": "performance"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "WiFi interface power mode set to: performance".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_set_interface_power driver");
        let mode = parameters.get("mode").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mode' parameter");
            return DriverError::missing_parameter("mode");
        })?;
        let interface = parameters.get("interface").and_then(|v| v.as_str()).unwrap_or("wlan0");
        #[cfg(target_os = "linux")]
        {
            let power_value = match mode {
                "performance" => "off",
                "powersave" => "on",
                _ => {
                    debug!("Invalid mode: {}", mode);
                    return Err(DriverError::invalid_enum_value("mode", mode, vec!["performance".to_string(), "powersave".to_string()]));
                }
            };
            Command::new("iwconfig").args([interface, "power", power_value]).output().map_err(|e| {
                debug!("Failed to set power mode: {}", e);
                return DriverError::execution(format!("Failed to set power mode: {}", e));
            })?;
        }
        #[cfg(target_os = "windows")]
        {
            let scheme = match mode {
                "performance" => "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c", // High performance
                "powersave" => "a1841308-3541-4fab-bc81-f71556f20b4a",   // Power saver
                _ => {
                    debug!("Invalid mode: {}", mode);
                    return Err(DriverError::invalid_enum_value("mode", mode, vec!["performance".to_string(), "powersave".to_string()]));
                }
            };
            Command::new("powercfg").args(["/setactive", scheme]).output().map_err(|e| {
                debug!("Failed to set power scheme: {}", e);
                return DriverError::execution(format!("Failed to set power scheme: {}", e));
            })?;
        }
        info!("WiFi interface power mode set to: {}", mode);
        return Ok(format!("WiFi interface power mode set to: {}", mode));
    }
}
