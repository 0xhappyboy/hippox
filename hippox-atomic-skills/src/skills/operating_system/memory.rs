//! Process memory operations skills
//!
//! This module provides low-level process memory access for debugging,
//! reverse engineering, and security analysis.
//!
//! # Warning
//! These skills can read memory from other processes. Use responsibly
//! and only on processes you own or have permission to analyze.

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::Pattern;
use crate::ProcessMemory;
use crate::{
    mem,
    types::{Skill, SkillParameter},
};

/// Get basic information about a specific process (PID, name, parent PID)
#[derive(Debug)]
pub struct ProcessBasicInfoSkill;

#[async_trait::async_trait]
impl Skill for ProcessBasicInfoSkill {
    fn name(&self) -> &str {
        "process_basic_info"
    }

    fn description(&self) -> &str {
        "Get basic information about a specific process by PID (name, parent PID)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill after process_list to get basic info about a specific process"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "pid".to_string(),
            param_type: "integer".to_string(),
            description: "Process ID to inspect".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(1234.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_basic_info",
            "parameters": {
                "pid": 1234
            }
        })
    }

    fn example_output(&self) -> String {
        "PID: 1234\nName: chrome.exe\nParent PID: 5678".to_string()
    }

    fn category(&self) -> &str {
        "operating_system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;
        let processes = mem::list_processes()?;
        let process = processes.iter().find(|p| p.pid == pid);
        if let Some(p) = process {
            let mut output = format!("PID: {}\nName: {}\n", p.pid, p.name);
            if let Some(ppid) = p.parent_pid {
                output.push_str(&format!("Parent PID: {}\n", ppid));
            }
            Ok(output)
        } else {
            anyhow::bail!("Process with PID {} not found", pid)
        }
    }
}

/// Read memory from a process at a specified address
#[derive(Debug)]
pub struct MemoryReadSkill;

#[async_trait::async_trait]
impl Skill for MemoryReadSkill {
    fn name(&self) -> &str {
        "memory_read"
    }

    fn description(&self) -> &str {
        "Read memory from a running process at a specified address"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to inspect process memory for debugging or analysis. This is a read-only operation."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID to read from".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "Memory address to read (hex format, e.g., '0x7FF6A1B4C000')"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("0x7FF6A1B4C000".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "size".to_string(),
                param_type: "integer".to_string(),
                description: "Number of bytes to read".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(4.into())),
                enum_values: None,
            },
            SkillParameter {
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "memory_read",
            "parameters": {
                "pid": 1234,
                "address": "0x7FF6A1B4C000",
                "size": 4,
                "data_type": "u32"
            }
        })
    }

    fn example_output(&self) -> String {
        "Value: 12345".to_string()
    }

    fn category(&self) -> &str {
        "operating_system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;
        let address_str = parameters
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: address"))?;
        let address = usize::from_str_radix(address_str.trim_start_matches("0x"), 16)
            .map_err(|_| anyhow::anyhow!("Invalid address format: {}", address_str))?;
        let data_type = parameters
            .get("data_type")
            .and_then(|v| v.as_str())
            .unwrap_or("bytes");
        let size = parameters.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let mut memory = ProcessMemory::open(pid, true)?;
        match data_type {
            "u8" => {
                let value = memory.read_u8(address)?;
                Ok(format!("Value: {}", value))
            }
            "u16" => {
                let value = memory.read_u16(address)?;
                Ok(format!("Value: {}", value))
            }
            "u32" => {
                let value = memory.read_u32(address)?;
                Ok(format!("Value: {}", value))
            }
            "u64" => {
                let value = memory.read_u64(address)?;
                Ok(format!("Value: {}", value))
            }
            "f32" => {
                let value = memory.read_f32(address)?;
                Ok(format!("Value: {:.6}", value))
            }
            "f64" => {
                let value = memory.read_f64(address)?;
                Ok(format!("Value: {:.6}", value))
            }
            "string" => {
                let max_len = if size > 0 { size } else { 256 };
                let value = memory.read_string(address, max_len)?;
                Ok(format!("String: \"{}\"", value))
            }
            _ => {
                let read_size = if size > 0 { size } else { 16 };
                let mut buffer = vec![0u8; read_size];
                let bytes_read = memory.read_memory(address, &mut buffer)?;
                let hex: String = buffer[..bytes_read]
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(format!("Bytes ({} bytes): {}", bytes_read, hex))
            }
        }
    }
}

/// Scan memory for a specific byte pattern
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

    fn category(&self) -> &str {
        "operating_system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;
        let pattern_str = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        let pattern = Pattern::from_hex(pattern_str)?;
        let mut memory = ProcessMemory::open(pid, true)?;
        let start_address = if let Some(module) = parameters.get("module").and_then(|v| v.as_str())
        {
            memory.get_module_base(module)?
        } else {
            0x10000
        };
        let scan_size = 64 * 1024 * 1024;
        let results = mem::scan_region(&mut memory, start_address, scan_size, &pattern)?;
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

/// Get the base address of a loaded module (DLL/so) in a process
#[derive(Debug)]
pub struct ModuleBaseSkill;

#[async_trait::async_trait]
impl Skill for ModuleBaseSkill {
    fn name(&self) -> &str {
        "module_base"
    }

    fn description(&self) -> &str {
        "Get the base address of a loaded module (DLL/so) in a process"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find where a DLL is loaded in memory"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1234.into())),
                enum_values: None,
            },
            SkillParameter {
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

    fn category(&self) -> &str {
        "operating_system"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pid = parameters
            .get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pid"))?
            as u32;
        let module = parameters
            .get("module")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module"))?;
        let memory = ProcessMemory::open(pid, true)?;
        let base = memory.get_module_base(module)?;
        Ok(format!("Module base address: 0x{:X}", base))
    }
}
