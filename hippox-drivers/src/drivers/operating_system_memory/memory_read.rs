//! Memory read driver module
//!
//! This module provides functionality to read memory from a running process
//! at a specified address with support for various data types.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_memory::common::ProcessMemory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading memory from a process
#[derive(Debug)]
pub struct MemoryReadDriver;
#[async_trait::async_trait]
impl Driver for MemoryReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "memory_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read memory from a running process at a specified address";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to inspect process memory for debugging or analysis. This is a read-only operation.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID to read from".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "Memory address to read (hex format, e.g., '0x7FF6A1B4C000')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("0x7FF6A1B4C000".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "size".to_string(),
                param_type: "integer".to_string(),
                description: "Number of bytes to read".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(4.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "data_type".to_string(),
                param_type: "string".to_string(),
                description: "Data type: bytes, u8, u16, u32, u64, f32, f64, string".to_string(),
                required: false,
                default: Some(Value::String("bytes".to_string())),
                example: Some(Value::String("u32".to_string())),
                enum_values: Some(vec![
                    "bytes".to_string(),
                    "u8".to_string(),
                    "u16".to_string(),
                    "u32".to_string(),
                    "u64".to_string(),
                    "f32".to_string(),
                    "f64".to_string(),
                    "string".to_string(),
                ]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "memory_read",
            "parameters": {
                "pid": 1234,
                "address": "0x7FF6A1B4C000",
                "size": 4,
                "data_type": "u32"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Value: 12345".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemMemory;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing memory_read driver");
        // Extract required parameters
        let pid = parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))? as u32;
        let address_str = parameters.get("address").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("address"))?;
        let address = usize::from_str_radix(address_str.trim_start_matches("0x"), 16)
            .map_err(|e| DriverError::execution(format!("Invalid address format: {}: {}", address_str, e)))?;
        let data_type = parameters.get("data_type").and_then(|v| v.as_str()).unwrap_or("bytes");
        let size = parameters.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        debug!("Reading memory from PID {} at 0x{:X}, type: {}", pid, address, data_type);
        let mut memory = ProcessMemory::open(pid, true).map_err(|e| DriverError::execution(format!("Failed to open process: {}", e)))?;
        let result = match data_type {
            "u8" => {
                let value = memory.read_u8(address).map_err(|e| DriverError::execution(format!("Failed to read u8: {}", e)))?;
                format!("Value: {}", value)
            }
            "u16" => {
                let value = memory.read_u16(address).map_err(|e| DriverError::execution(format!("Failed to read u16: {}", e)))?;
                format!("Value: {}", value)
            }
            "u32" => {
                let value = memory.read_u32(address).map_err(|e| DriverError::execution(format!("Failed to read u32: {}", e)))?;
                format!("Value: {}", value)
            }
            "u64" => {
                let value = memory.read_u64(address).map_err(|e| DriverError::execution(format!("Failed to read u64: {}", e)))?;
                format!("Value: {}", value)
            }
            "f32" => {
                let value = memory.read_f32(address).map_err(|e| DriverError::execution(format!("Failed to read f32: {}", e)))?;
                format!("Value: {:.6}", value)
            }
            "f64" => {
                let value = memory.read_f64(address).map_err(|e| DriverError::execution(format!("Failed to read f64: {}", e)))?;
                format!("Value: {:.6}", value)
            }
            "string" => {
                let max_len = if size > 0 { size } else { 256 };
                let value = memory.read_string(address, max_len).map_err(|e| DriverError::execution(format!("Failed to read string: {}", e)))?;
                format!("String: \"{}\"", value)
            }
            _ => {
                let read_size = if size > 0 { size } else { 16 };
                let mut buffer = vec![0u8; read_size];
                let bytes_read =
                    memory.read_memory(address, &mut buffer).map_err(|e| DriverError::execution(format!("Failed to read memory: {}", e)))?;
                let hex: String = buffer[..bytes_read].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                format!("Bytes ({} bytes): {}", bytes_read, hex)
            }
        };
        info!("Memory read completed: {}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))?;
        parameters.get("address").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("address"))?;
        parameters.get("size").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("size"))?;
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_read_metadata() {
        let skill = MemoryReadDriver;
        assert_eq!(skill.name(), "memory_read");
        assert_eq!(skill.category(), DriverCategory::OperatingSystemMemory);
        assert!(!skill.parameters().is_empty());
    }
}
