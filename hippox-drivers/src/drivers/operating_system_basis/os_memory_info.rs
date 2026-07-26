//! OS memory info driver
//!
//! This driver provides functionality to get system memory (RAM) information.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting memory information
#[derive(Debug)]
pub struct OsMemoryInfoDriver;
#[async_trait::async_trait]
impl Driver for OsMemoryInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_memory_info"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get system memory (RAM) information"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check total, used, and available memory"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_memory_info"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Total Memory: 16.0 GB\nUsed Memory: 8.2 GB (51%)\nAvailable Memory: 7.8 GB".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_memory_info driver");
        let mut system = System::new();
        system.refresh_memory();
        let total_gb = system.total_memory() as f64 / (1024.0 * 1024.0);
        let used_gb = system.used_memory() as f64 / (1024.0 * 1024.0);
        let free_gb = system.free_memory() as f64 / (1024.0 * 1024.0);
        let used_percent = (used_gb / total_gb) * 100.0;
        info!("Memory info retrieved: {:.1} GB total, {:.1} GB used ({:.0}%)", total_gb, used_gb, used_percent);
        return Ok(format!(
            "Total Memory: {:.1} GB\nUsed Memory: {:.1} GB ({:.0}%)\nFree Memory: {:.1} GB\nAvailable Memory: {:.1} GB",
            total_gb,
            used_gb,
            used_percent,
            free_gb,
            system.available_memory() as f64 / (1024.0 * 1024.0)
        ));
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
