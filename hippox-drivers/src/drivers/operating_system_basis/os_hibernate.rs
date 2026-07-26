//! OS hibernate driver
//!
//! This driver provides functionality to hibernate the system (suspend to disk).
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, exec_async,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for hibernating the system
#[derive(Debug)]
pub struct OsHibernateDriver;
#[async_trait::async_trait]
impl Driver for OsHibernateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_hibernate"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Hibernate the system (suspend to disk)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to save power while preserving system state to disk"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_hibernate"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System is hibernating".to_string();
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
        debug!("Executing os_hibernate driver");
        #[cfg(target_os = "windows")]
        {
            debug!("Hibernating on Windows");
            exec_async("shutdown", &["/h"], None).await.map_err(|e| DriverError::execution(format!("Failed to hibernate on Windows: {}", e)))?;
            info!("System is hibernating on Windows");
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Hibernating on Linux");
            exec_async("systemctl", &["hibernate"], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to hibernate on Linux: {}", e)))?;
            info!("System is hibernating on Linux");
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Hibernating on macOS");
            exec_async("pmset", &["sleepnow"], None).await.map_err(|e| DriverError::execution(format!("Failed to hibernate on macOS: {}", e)))?;
            info!("System is hibernating on macOS");
        }
        return Ok("System is hibernating".to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_hibernate_metadata() {
        let driver = OsHibernateDriver;
        assert_eq!(driver.name(), "os_hibernate");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
