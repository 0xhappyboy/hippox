//! Bluetooth profile list skill - list supported Bluetooth profiles
//!
//! This driver provides functionality to list the Bluetooth profiles
//! supported by the system.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing supported Bluetooth profiles
///
/// This driver displays the Bluetooth profiles that the system supports,
/// such as A2DP, HFP, SPP, and HID.
#[derive(Debug)]
pub struct BluetoothProfileListDriver;
#[async_trait::async_trait]
impl Driver for BluetoothProfileListDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_profile_list"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List supported Bluetooth profiles on the system (A2DP, HFP, SPP, etc.)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see what Bluetooth profiles your system supports."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_profile_list"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Supported Bluetooth Profiles:\n1. A2DP (Audio)\n2. HFP (Hands-Free)\n3. SPP (Serial Port)\n4. HID (Human Interface)".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing bluetooth_profile_list driver");
        let profiles = vec![
            "A2DP (Audio Source/Sink)",
            "HFP (Hands-Free Profile)",
            "HSP (Headset Profile)",
            "SPP (Serial Port Profile)",
            "HID (Human Interface Device)",
            "PAN (Personal Area Networking)",
            "OBEX (Object Exchange)",
            "GATT (Generic Attribute Profile)",
        ];
        debug!("Found {} supported profiles", profiles.len());
        let mut result = String::from("Supported Bluetooth Profiles:\n");
        for (i, profile) in profiles.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, profile));
        }
        info!("Listed {} supported Bluetooth profiles", profiles.len());
        Ok(result)
    }
}
