//! Module base address retrieval driver module
//!
//! This module provides functionality to get the base address of a loaded
//! module (DLL/so) in a process.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_memory::common::ProcessMemory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting the base address of a loaded module
#[derive(Debug)]
pub struct ModuleBaseDriver;
#[async_trait::async_trait]
impl Driver for ModuleBaseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "module_base";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get the base address of a loaded module (DLL/so) in a process";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to find where a DLL is loaded in memory";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "module".to_string(),
                param_type: "string".to_string(),
                description: "Module name (e.g., 'kernel32.dll' or 'libc.so.6')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("kernel32.dll".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "module_base",
            "parameters": {
                "pid": 1234,
                "module": "kernel32.dll"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Module base address: 0x7FF6A1B40000".to_string();
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
        debug!("Executing module_base driver");
        // Extract required parameters
        let pid = parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))? as u32;
        let module = parameters.get("module").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("module"))?;
        debug!("Getting base address of module '{}' in PID {}", module, pid);
        let memory = ProcessMemory::open(pid, true).map_err(|e| DriverError::execution(format!("Failed to open process: {}", e)))?;
        let base = memory.get_module_base(module).map_err(|e| DriverError::execution(format!("Failed to get module base: {}", e)))?;
        let result = format!("Module base address: 0x{:X}", base);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))?;
        parameters.get("module").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("module"))?;
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_module_base_metadata() {
        let skill = ModuleBaseDriver;
        assert_eq!(skill.name(), "module_base");
        assert_eq!(skill.category(), DriverCategory::OperatingSystemMemory);
        assert!(!skill.parameters().is_empty());
    }
}
