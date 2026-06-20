//! OS memory info driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
#[derive(Debug)]
pub struct OsMemoryInfoDriver;
#[async_trait::async_trait]
impl Driver for OsMemoryInfoDriver {
    fn name(&self) -> &str {
        "os_memory_info"
    }
    fn description(&self) -> &str {
        "Get system memory (RAM) information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to check total, used, and available memory"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_memory_info"
        })
    }
    fn example_output(&self) -> String {
        "Total Memory: 16.0 GB\nUsed Memory: 8.2 GB (51%)\nAvailable Memory: 7.8 GB".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let mut system = System::new();
        system.refresh_memory();
        let total_gb = system.total_memory() as f64 / (1024.0 * 1024.0);
        let used_gb = system.used_memory() as f64 / (1024.0 * 1024.0);
        let free_gb = system.free_memory() as f64 / (1024.0 * 1024.0);
        let used_percent = (used_gb / total_gb) * 100.0;
        Ok(format!(
            "Total Memory: {:.1} GB\nUsed Memory: {:.1} GB ({:.0}%)\nFree Memory: {:.1} GB\nAvailable Memory: {:.1} GB",
            total_gb,
            used_gb,
            used_percent,
            free_gb,
            system.available_memory() as f64 / (1024.0 * 1024.0)
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_memory_info_metadata() {
        let driver = OsMemoryInfoDriver;
        assert_eq!(driver.name(), "os_memory_info");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
