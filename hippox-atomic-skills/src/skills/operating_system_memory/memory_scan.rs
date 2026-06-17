//! Memory scan skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    operating_system_memory::common::{Pattern, ProcessMemory},
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Skill for scanning memory for a specific byte pattern
#[derive(Debug)]
pub struct MemoryScanSkill;

#[async_trait::async_trait]
impl Skill for MemoryScanSkill {
    fn name(&self) -> &str {
        "memory_scan"
    }

    fn description(&self) -> &str {
        "Scan process memory for a specific byte pattern (hex pattern with wildcards)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find memory addresses containing specific values. Pattern format: '48 8B 05 ? ? ? ?' where '?' is a wildcard."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID to scan".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Hex pattern to search for (e.g., '48 8B 05 ? ? ? ?')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("48 8B 05 ? ? ? ?".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "module".to_string(),
                param_type: "string".to_string(),
                description: "Optional module name to limit scan to a specific module".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("game.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "size".to_string(),
                param_type: "integer".to_string(),
                description: "Size of memory region to scan in bytes (default: 64MB)".to_string(),
                required: false,
                default: Some(Value::Number(67108864.into())),
                example: Some(Value::Number(1048576.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "memory_scan",
            "parameters": {
                "pid": 1234,
                "pattern": "48 8B 05 ? ? ? ?"
            }
        })
    }

    fn example_output(&self) -> String {
        "Found at addresses:\n0x7FF6A1B4C000\n0x7FF6A1B4C100".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemMemory
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;

        let pattern_str = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;

        let pattern = Pattern::from_hex(pattern_str)
            .map_err(|e| anyhow::anyhow!("Invalid pattern: {}", e))?;

        let default_size = 64 * 1024 * 1024;
        let scan_size = parameters
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(default_size) as usize;

        let mut memory = ProcessMemory::open(pid, true)
            .map_err(|e| anyhow::anyhow!("Failed to open process: {}", e))?;

        let start_address = if let Some(module) = parameters.get("module").and_then(|v| v.as_str())
        {
            memory
                .get_module_base(module)
                .map_err(|e| anyhow::anyhow!("Failed to get module base: {}", e))?
        } else {
            0x10000
        };

        let results = memory
            .scan(start_address, scan_size, &pattern)
            .map_err(|e| anyhow::anyhow!("Failed to scan memory: {}", e))?;

        if results.is_empty() {
            Ok("Pattern not found".to_string())
        } else {
            let mut output = format!("Found at {} address(es):\n", results.len());
            for addr in results.iter().take(100) {
                output.push_str(&format!("0x{:X}\n", addr));
            }
            if results.len() > 100 {
                output.push_str(&format!("... and {} more", results.len() - 100));
            }
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_scan_skill_metadata() {
        let skill = MemoryScanSkill;
        assert_eq!(skill.name(), "memory_scan");
        assert_eq!(skill.category(), SkillCategory::OperatingSystemMemory);
        assert!(!skill.parameters().is_empty());
    }
}
