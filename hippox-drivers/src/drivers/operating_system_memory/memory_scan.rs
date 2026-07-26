//! Memory scan driver module
//!
//! This module provides functionality to scan process memory for a specific
//! byte pattern with support for wildcards.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_memory::common::{Pattern, ProcessMemory},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for scanning memory for a specific byte pattern
#[derive(Debug)]
pub struct MemoryScanDriver;
#[async_trait::async_trait]
impl Driver for MemoryScanDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "memory_scan";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Scan process memory for a specific byte pattern (hex pattern with wildcards)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to find memory addresses containing specific values. Pattern format: '48 8B 05 ? ? ? ?' where '?' is a wildcard.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID to scan".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Hex pattern to search for (e.g., '48 8B 05 ? ? ? ?')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("48 8B 05 ? ? ? ?".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "module".to_string(),
                param_type: "string".to_string(),
                description: "Optional module name to limit scan to a specific module".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("game.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "size".to_string(),
                param_type: "integer".to_string(),
                description: "Size of memory region to scan in bytes (default: 64MB)".to_string(),
                required: false,
                default: Some(Value::Number(67108864.into())),
                example: Some(Value::Number(1048576.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "memory_scan",
            "parameters": {
                "pid": 1234,
                "pattern": "48 8B 05 ? ? ? ?"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Found at addresses:\n0x7FF6A1B4C000\n0x7FF6A1B4C100".to_string();
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
        debug!("Executing memory_scan driver");
        // Extract required parameters
        let pid = parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))? as u32;
        let pattern_str = parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        let pattern = Pattern::from_hex(pattern_str).map_err(|e| DriverError::execution(format!("Invalid pattern: {}", e)))?;
        let default_size = 64 * 1024 * 1024;
        let scan_size = parameters.get("size").and_then(|v| v.as_u64()).unwrap_or(default_size) as usize;
        debug!("Scanning PID {} for pattern: {}, size: {}", pid, pattern_str, scan_size);
        let mut memory = ProcessMemory::open(pid, true).map_err(|e| DriverError::execution(format!("Failed to open process: {}", e)))?;
        let start_address = if let Some(module) = parameters.get("module").and_then(|v| v.as_str()) {
            memory.get_module_base(module).map_err(|e| DriverError::execution(format!("Failed to get module base: {}", e)))?
        } else {
            0x10000
        };
        let results = memory.scan(start_address, scan_size, &pattern).map_err(|e| DriverError::execution(format!("Failed to scan memory: {}", e)))?;
        let result = if results.is_empty() {
            "Pattern not found".to_string()
        } else {
            let mut output = format!("Found at {} address(es):\n", results.len());
            for addr in results.iter().take(100) {
                output.push_str(&format!("0x{:X}\n", addr));
            }
            if results.len() > 100 {
                output.push_str(&format!("... and {} more", results.len() - 100));
            }
            output
        };
        info!("Memory scan completed: {}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))?;
        parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_scan_metadata() {
        let skill = MemoryScanDriver;
        assert_eq!(skill.name(), "memory_scan");
        assert_eq!(skill.category(), DriverCategory::OperatingSystemMemory);
        assert!(!skill.parameters().is_empty());
    }
}
