//! Netstat driver
//!
//! This driver provides functionality to view network connections and listening ports
//! on the local system.
use crate::common::net::get_network_connections;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for viewing network connections
#[derive(Debug)]
pub struct NetstatDriver;
#[async_trait::async_trait]
impl Driver for NetstatDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "netstat"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "View network connections and listening ports on the local system"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to check what ports are open or what connections are active on the local machine"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "netstat"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Network connections:\nlocal: 0.0.0.0:22 remote: 0.0.0.0:* state: LISTEN\nlocal: 127.0.0.1:5432 remote: 0.0.0.0:* state: LISTEN"
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing netstat driver");
        let connections = get_network_connections().map_err(|e| DriverError::execution(format!("Failed to get network connections: {}", e)))?;
        info!("Found {} network connections", connections.len());
        let mut output = format!("Network connections ({}):\n", connections.len());
        for conn in connections.clone() {
            let parts: Vec<String> = conn.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
            output.push_str(&format!("  {}\n", parts.join(" ")));
        }
        if connections.is_empty() {
            output.push_str("  No connections found\n");
            info!("No network connections found");
        }
        return Ok(output);
    }
}
