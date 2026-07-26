//! WiFi import config skill - import WiFi configuration from file
use super::common::connect_wifi;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use tracing::{debug, info};
/// Driver for importing WiFi configuration
#[derive(Debug)]
pub struct WifiImportConfigDriver;
#[async_trait::async_trait]
impl Driver for WifiImportConfigDriver {
    fn name(&self) -> &str {
        return "wifi_import_config";
    }
    fn description(&self) -> &str {
        return "Import WiFi configuration from a backup file to restore saved networks";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to restore WiFi settings from a previously exported backup file.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "file_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the configuration file to import".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("./wifi_backup.json".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "auto_connect".to_string(),
                param_type: "boolean".to_string(),
                description: "Automatically connect to the best network after import (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_import_config",
            "parameters": {
                "file_path": "./wifi_backup.json",
                "auto_connect": false
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Imported 3 WiFi networks from: ./wifi_backup.json".to_string();
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
        debug!("Executing wifi_import_config driver");
        let file_path = parameters.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'file_path' parameter");
            return DriverError::missing_parameter("file_path");
        })?;
        let auto_connect = parameters.get("auto_connect").and_then(|v| v.as_bool()).unwrap_or(false);
        // Read and parse the backup file
        let mut file = File::open(file_path).map_err(|e| {
            debug!("Failed to open file {}: {}", file_path, e);
            return DriverError::execution(format!("Failed to open file: {}", e));
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            debug!("Failed to read file: {}", e);
            return DriverError::execution(format!("Failed to read file: {}", e));
        })?;
        let import_data: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
            debug!("Failed to parse JSON: {}", e);
            return DriverError::execution(format!("Failed to parse backup file: {}", e));
        })?;
        let networks = import_data["networks"].as_array().ok_or_else(|| {
            debug!("Invalid backup file format: missing 'networks' array");
            return DriverError::execution("Invalid backup file format: missing 'networks' array");
        })?;
        let mut imported_count = 0;
        let mut best_network: Option<String> = None;
        let mut best_signal = -100;
        for network in networks {
            let ssid = network["ssid"].as_str().ok_or_else(|| {
                debug!("Network missing 'ssid' field");
                return DriverError::execution("Network missing 'ssid' field");
            })?;
            let password = network["password"].as_str();
            // Import the network
            connect_wifi(ssid, password).map_err(|e| {
                debug!("Failed to import network {}: {}", ssid, e);
                return DriverError::execution(format!("Failed to import network: {}", e));
            })?;
            imported_count += 1;
            // Check signal strength for auto-connect decision
            if auto_connect {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let status = super::common::get_wifi_status().map_err(|e| {
                    debug!("Failed to get WiFi status: {}", e);
                    return DriverError::execution(format!("Failed to get WiFi status: {}", e));
                })?;
                if let Some(signal) = status.signal_strength {
                    if signal > best_signal {
                        best_signal = signal;
                        best_network = Some(ssid.to_string());
                    }
                }
            }
        }
        // Auto-connect to the best network if requested
        let auto_connect_msg = if auto_connect {
            if let Some(ref network) = best_network {
                connect_wifi(network, None).map_err(|e| {
                    debug!("Failed to auto-connect to {}: {}", network, e);
                    return DriverError::execution(format!("Failed to auto-connect: {}", e));
                })?;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                format!(" (auto-connected to: {})", network)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        info!("Imported {} WiFi networks from: {}{}", imported_count, file_path, auto_connect_msg);
        return Ok(format!("Imported {} WiFi networks from: {}{}", imported_count, file_path, auto_connect_msg));
    }
}
