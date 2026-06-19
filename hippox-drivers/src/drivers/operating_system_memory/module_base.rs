//! Module base address retrieval Driver

use crate::{DriverCallback, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    DriverCategory,
    operating_system_memory::common::ProcessMemory,
    types::{Driver, DriverParameter},
};

/// Driver for getting the base address of a loaded module
#[derive(Debug)]
pub struct ModuleBaseDriver;

#[async_trait::async_trait]
impl Driver for ModuleBaseDriver {
    fn name(&self) -> &str {
        "module_base"
    }

    fn description(&self) -> &str {
        "Get the base address of a loaded module (DLL/so) in a process"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find where a DLL is loaded in memory"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "module_base",
            "parameters": {
                "pid": 1234,
                "module": "kernel32.dll"
            }
        })
    }

    fn example_output(&self) -> String {
        "Module base address: 0x7FF6A1B40000".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemMemory
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;

        let module = parameters
            .get("module")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module"))?;

        let memory = ProcessMemory::open(pid, true)
            .map_err(|e| anyhow::anyhow!("Failed to open process: {}", e))?;
        let base = memory
            .get_module_base(module)
            .map_err(|e| anyhow::anyhow!("Failed to get module base: {}", e))?;

        Ok(format!("Module base address: 0x{:X}", base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_base_skill_metadata() {
        let skill = ModuleBaseDriver;
        assert_eq!(skill.name(), "module_base");
        assert_eq!(skill.category(), DriverCategory::OperatingSystemMemory);
        assert!(!skill.parameters().is_empty());
    }
}
