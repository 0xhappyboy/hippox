//! CPU cache information driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_cpu::common::CpuCacheInfo,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting CPU cache information
#[derive(Debug)]
pub struct CpuCacheDriver;

#[async_trait::async_trait]
impl Driver for CpuCacheDriver {
    fn name(&self) -> &str {
        "cpu_cache"
    }

    fn description(&self) -> &str {
        "Get CPU cache information including size, line size, and associativity"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to understand CPU cache hierarchy and performance characteristics"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_cache",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"CPU Cache Information:
L1 Cache (Data): 32 KB, 64-byte lines, 8-way associative
L1 Cache (Instruction): 32 KB, 64-byte lines, 8-way associative
L2 Cache: 256 KB, 64-byte lines, 4-way associative
L3 Cache: 12 MB, 64-byte lines, 16-way associative"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let caches = get_cache_info()?;

        let mut output = String::from("CPU Cache Information:\n");
        for cache in caches {
            let associativity = cache
                .associativity
                .map(|a| format!("{}-way", a))
                .unwrap_or_else(|| "Unknown".to_string());

            output.push_str(&format!(
                "L{} Cache ({}): {} KB, {}-byte lines, {}\n",
                cache.level, cache.cache_type, cache.size_kb, cache.line_size_bytes, associativity
            ));
        }

        Ok(output)
    }
}

fn get_cache_info() -> Result<Vec<CpuCacheInfo>> {
    let mut caches = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Parse /sys/devices/system/cpu/cpu0/cache/
        let base_path = "/sys/devices/system/cpu/cpu0/cache";
        for entry in std::fs::read_dir(base_path)? {
            let entry = entry?;
            let path = entry.path();
            let index = entry.file_name().to_string_lossy().to_string();

            if !index.starts_with("index") {
                continue;
            }

            let size_path = path.join("size");
            let type_path = path.join("type");
            let line_size_path = path.join("coherency_line_size");
            let ways_path = path.join("ways_of_associativity");
            let level_path = path.join("level");

            if let (Ok(size_str), Ok(type_str), Ok(line_size_str)) = (
                std::fs::read_to_string(&size_path),
                std::fs::read_to_string(&type_path),
                std::fs::read_to_string(&line_size_path),
            ) {
                let size_kb = parse_size(&size_str.trim());
                let cache_type = type_str.trim().to_string();
                let line_size = line_size_str.trim().parse::<u64>().unwrap_or(64);
                let level = std::fs::read_to_string(&level_path)
                    .map(|s| s.trim().parse::<u8>().unwrap_or(1))
                    .unwrap_or(1);
                let associativity = std::fs::read_to_string(&ways_path)
                    .map(|s| s.trim().parse::<u64>().ok())
                    .unwrap_or(None);

                caches.push(CpuCacheInfo {
                    level,
                    size_kb,
                    line_size_bytes: line_size,
                    associativity,
                    cache_type,
                });
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Fallback: provide common cache sizes based on CPU model
        // This is a rough estimate and should be improved
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_all();
        if let Some(cpu) = system.cpus().first() {
            let brand = cpu.brand().to_lowercase();
            // Try to detect cache sizes from CPU brand string
            let (l1_size, l2_size, l3_size) = if brand.contains("intel") {
                (48, 256, 12288) // Common for recent Intel CPUs
            } else if brand.contains("amd") {
                (32, 512, 16384) // Common for recent AMD CPUs
            } else {
                (32, 256, 8192) // Default
            };
            caches.push(CpuCacheInfo {
                level: 1,
                size_kb: l1_size,
                line_size_bytes: 64,
                associativity: Some(8),
                cache_type: "Data + Instruction".to_string(),
            });
            caches.push(CpuCacheInfo {
                level: 2,
                size_kb: l2_size,
                line_size_bytes: 64,
                associativity: Some(4),
                cache_type: "Unified".to_string(),
            });

            if l3_size > 0 {
                caches.push(CpuCacheInfo {
                    level: 3,
                    size_kb: l3_size,
                    line_size_bytes: 64,
                    associativity: Some(16),
                    cache_type: "Unified".to_string(),
                });
            }
        } else {
            // Ultimate fallback
            caches.push(CpuCacheInfo {
                level: 1,
                size_kb: 32,
                line_size_bytes: 64,
                associativity: Some(8),
                cache_type: "Data + Instruction".to_string(),
            });
            caches.push(CpuCacheInfo {
                level: 2,
                size_kb: 256,
                line_size_bytes: 64,
                associativity: Some(4),
                cache_type: "Unified".to_string(),
            });
        }
    }
    // Sort by level
    caches.sort_by_key(|c| c.level);
    Ok(caches)
}

fn parse_size(size_str: &str) -> u64 {
    let size_str = size_str.trim();
    if size_str.ends_with('K') {
        size_str[..size_str.len() - 1].parse::<u64>().unwrap_or(0)
    } else if size_str.ends_with('M') {
        size_str[..size_str.len() - 1].parse::<u64>().unwrap_or(0) * 1024
    } else if size_str.ends_with('G') {
        size_str[..size_str.len() - 1].parse::<u64>().unwrap_or(0) * 1024 * 1024
    } else {
        size_str.parse::<u64>().unwrap_or(0) / 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_cache_metadata() {
        let driver = CpuCacheDriver;
        assert_eq!(driver.name(), "cpu_cache");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
