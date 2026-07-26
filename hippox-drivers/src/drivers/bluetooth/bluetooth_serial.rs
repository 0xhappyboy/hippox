//! Bluetooth serial skill - read/write data via Bluetooth SPP
//!
//! This driver provides functionality to communicate with Bluetooth
//! Serial Port Profile (SPP) devices.
use crate::DriverCallback;
use crate::DriverCategory;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process::Command;
use tracing::{debug, info, warn};
/// Driver for Bluetooth serial communication
///
/// This driver enables read/write communication with Bluetooth Serial
/// Port Profile (SPP) devices like Arduino, GPS modules, and serial adapters.
#[derive(Debug)]
pub struct BluetoothSerialDriver;
#[async_trait::async_trait]
impl Driver for BluetoothSerialDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_serial"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Read and write data via Bluetooth Serial Port Profile (SPP)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to communicate with Bluetooth serial devices like Arduino, GPS modules, or serial adapters."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the Bluetooth serial device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to send (for write operation)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("AT\r\n".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "read_timeout_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout for reading response in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(5000.into())),
                example: Some(Value::Number(3000.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "bluetooth_serial",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "command": "AT\r\n",
                "read_timeout_ms": 5000
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Response: OK".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing bluetooth_serial driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let command = parameters.get("command").and_then(|v| v.as_str());
        let read_timeout = parameters.get("read_timeout_ms").and_then(|v| v.as_u64()).unwrap_or(5000);
        debug!("Serial communication with: {}, timeout: {}ms", mac_address, read_timeout);
        #[cfg(target_os = "linux")]
        {
            let device_path = "/dev/rfcomm0";
            debug!("Binding RFCOMM to {}", device_path);
            // Bind RFCOMM if not already bound
            let bind_output = Command::new("rfcomm").args(["bind", "0", mac_address]).output();
            if let Ok(output) = bind_output {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("rfcomm bind warning: {}", stderr);
                }
            }
            debug!("Opening serial device: {}", device_path);
            let mut serial = OpenOptions::new()
                .read(true)
                .write(true)
                .open(device_path)
                .map_err(|e| DriverError::execution(format!("Failed to open serial device: {}", e)))?;
            if let Some(cmd) = command {
                debug!("Writing command: {}", cmd);
                serial.write_all(cmd.as_bytes()).map_err(|e| DriverError::execution(format!("Failed to write to serial: {}", e)))?;
                serial.flush().map_err(|e| DriverError::execution(format!("Failed to flush serial: {}", e)))?;
            }
            // Read response
            debug!("Reading response (timeout: {}ms)", read_timeout);
            let mut buffer = vec![0u8; 1024];
            let mut response = String::new();
            let start = std::time::Instant::now();
            while start.elapsed() < std::time::Duration::from_millis(read_timeout) {
                if let Ok(n) = serial.read(&mut buffer) {
                    if n > 0 {
                        let chunk = String::from_utf8_lossy(&buffer[..n]);
                        response.push_str(&chunk);
                        debug!("Received: {}", chunk.trim());
                        if response.contains('\n') || response.contains('\r') {
                            break;
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            let result = format!("Response: {}", response.trim());
            info!("Serial communication completed: {}", result);
            return Ok(result);
        }
        info!("Serial communication with {}", mac_address);
        Ok(format!("Serial communication with {}", mac_address))
    }
}
