//! OS get user driver
//!
//! This driver provides functionality to get current user information.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::Users;
use tracing::{debug, info};
/// Driver for getting user information
#[derive(Debug)]
pub struct OsGetUserDriver;
#[async_trait::async_trait]
impl Driver for OsGetUserDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_user"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get current user information"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current username, home directory, and user ID"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_user"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Username: john\nUID: 1000\nGroups: sudo, docker".to_string();
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
        debug!("Executing os_get_user driver");
        let users = Users::new_with_refreshed_list();
        let current_user = users.iter().next();
        if let Some(user) = current_user {
            let groups: Vec<String> = user.groups().iter().map(|g| g.name().to_string()).collect();
            info!("User information retrieved: {}", user.name());
            return Ok(format!("Username: {}\nUID: {}\nGroups: {}", user.name().to_string(), user.id().to_string(), groups.join(", ")));
        } else {
            info!("User information could not be found");
            return Ok("User information could not be found.".to_string());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_user_metadata() {
        let driver = OsGetUserDriver;
        assert_eq!(driver.name(), "os_get_user");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
