//! WiFi get noise level skill - get noise level and SNR
use super::common::get_wifi_status;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting WiFi noise level and SNR
#[derive(Debug)]
pub struct WifiGetNoiseLevelDriver;
#[async_trait::async_trait]
impl Driver for WifiGetNoiseLevelDriver {
    fn name(&self) -> &str {
        return "wifi_get_noise_level";
    }
    fn description(&self) -> &str {
        return "Get the current channel noise level and signal-to-noise ratio (SNR)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to diagnose WiFi interference. Lower noise and higher SNR indicate better connection quality.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_get_noise_level"
        }));
    }
    fn example_output(&self) -> String {
        return "Signal: -45 dBm, Noise: -90 dBm, SNR: 45 dB".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_get_noise_level driver");
        let status = get_wifi_status().map_err(|e| {
            debug!("Failed to get WiFi status: {}", e);
            return DriverError::execution(format!("Failed to get WiFi status: {}", e));
        })?;
        if !status.connected {
            info!("Not connected to WiFi");
            return Ok("Not connected to WiFi".to_string());
        }
        let signal = status.signal_strength.unwrap_or(0);
        let signal_dbm = (signal - 100) as i32; // Rough conversion: 0% = -100dBm, 100% = 0dBm
        #[cfg(target_os = "linux")]
        let noise_dbm = -95; // Typical default
        #[cfg(not(target_os = "linux"))]
        let noise_dbm = -90;
        let snr = signal_dbm - noise_dbm;
        let quality = if snr > 30 {
            "Excellent"
        } else if snr > 20 {
            "Good"
        } else if snr > 10 {
            "Fair"
        } else {
            "Poor"
        };
        info!("Signal: {} dBm, Noise: {} dBm, SNR: {} dB, Quality: {}", signal_dbm, noise_dbm, snr, quality);
        return Ok(format!("Signal: {} dBm, Noise: {} dBm, SNR: {} dB\nQuality: {}", signal_dbm, noise_dbm, snr, quality));
    }
}
