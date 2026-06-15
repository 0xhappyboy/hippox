//! Bluetooth serial skill - read/write data via Bluetooth SPP

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct BluetoothSerialSkill;

#[async_trait::async_trait]
impl Skill for BluetoothSerialSkill {
    fn name(&self) -> &str {
        "bluetooth_serial"
    }

    fn description(&self) -> &str {
        "Read and write data via Bluetooth Serial Port Profile (SPP)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to communicate with Bluetooth serial devices like Arduino, GPS modules, or serial adapters."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the Bluetooth serial device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to send (for write operation)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("AT\r\n".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "read_timeout_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout for reading response in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(5000.into())),
                example: Some(Value::Number(3000.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_serial",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "command": "AT\r\n",
                "read_timeout_ms": 5000
            }
        })
    }

    fn example_output(&self) -> String {
        "Response: OK".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;
        
        let command = parameters
            .get("command")
            .and_then(|v| v.as_str());
        
        let read_timeout = parameters
            .get("read_timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);
        
        // Convert MAC address to RFCOMM channel
        let rfcomm_port = format!("00:{}", mac_address.replace(":", ""));
        
        // For Linux, use rfcomm or direct serial
        #[cfg(target_os = "linux")]
        {
            use std::fs::OpenOptions;
            
            let device_path = format!("/dev/rfcomm0");
            
            // Bind RFCOMM if not already bound
            let _ = Command::new("rfcomm")
                .args(["bind", "0", mac_address])
                .output();
            
            let mut serial = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&device_path)?;
            
            if let Some(cmd) = command {
                serial.write_all(cmd.as_bytes())?;
                serial.flush()?;
            }
            
            // Read response
            let mut buffer = vec![0u8; 1024];
            let mut response = String::new();
            let start = std::time::Instant::now();
            
            while start.elapsed() < std::time::Duration::from_millis(read_timeout) {
                if let Ok(n) = serial.read(&mut buffer) {
                    if n > 0 {
                        response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        if response.contains('\n') {
                            break;
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            return Ok(format!("Response: {}", response.trim()));
        }
        
        Ok(format!("Serial communication with {}", mac_address))
    }
}