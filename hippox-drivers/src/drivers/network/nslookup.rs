//! NSLookup driver
//!
//! This driver provides functionality to perform detailed DNS lookup with all record types.
use crate::common::net::{NslookupResult, nslookup};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for performing NSLookup
#[derive(Debug)]
pub struct NslookupDriver;
#[async_trait::async_trait]
impl Driver for NslookupDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "nslookup"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Perform detailed DNS lookup with all record types (A, AAAA, MX, TXT, CNAME, NS, SOA)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need comprehensive DNS information beyond basic A records"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "domain".to_string(),
                param_type: "string".to_string(),
                description: "Domain name to query".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "dns_server".to_string(),
                param_type: "string".to_string(),
                description: "DNS server to use (default: 8.8.8.8)".to_string(),
                required: false,
                default: Some(Value::String("8.8.8.8".to_string())),
                example: Some(Value::String("1.1.1.1".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "nslookup",
            "parameters": {
                "domain": "google.com"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "NSLookup for google.com (DNS: 8.8.8.8):\nA: 142.250.185.46\nMX: smtp.google.com (priority 10)\nNS: ns1.google.com".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing nslookup driver");
        let domain = parameters.get("domain").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("domain"))?;
        let dns_server = parameters.get("dns_server").and_then(|v| v.as_str());
        info!("NSLookup: domain={}, dns_server={:?}", domain, dns_server);
        let result = nslookup(domain, dns_server).await.map_err(|e| DriverError::execution(format!("NSLookup failed: {}", e)))?;
        info!("NSLookup completed for {}", domain);
        let mut output = format!("NSLookup for {} (DNS: {}):\n", result.domain, result.dns_server);
        if !result.a_records.is_empty() {
            output.push_str(&format!("A: {}\n", result.a_records.join(", ")));
        }
        if !result.aaaa_records.is_empty() {
            output.push_str(&format!("AAAA: {}\n", result.aaaa_records.join(", ")));
        }
        if !result.mx_records.is_empty() {
            let mx_str: Vec<String> = result.mx_records.iter().map(|(server, priority)| format!("{} (priority {})", server, priority)).collect();
            output.push_str(&format!("MX: {}\n", mx_str.join(", ")));
        }
        if !result.txt_records.is_empty() {
            output.push_str(&format!("TXT: {}\n", result.txt_records.join("; ")));
        }
        if !result.cname_records.is_empty() {
            output.push_str(&format!("CNAME: {}\n", result.cname_records.join(", ")));
        }
        if !result.ns_records.is_empty() {
            output.push_str(&format!("NS: {}\n", result.ns_records.join(", ")));
        }
        if let Some(soa) = result.soa_record {
            output.push_str(&format!("SOA: {}\n", soa));
        }
        if output == format!("NSLookup for {} (DNS: {}):\n", result.domain, result.dns_server) {
            output.push_str("No records found\n");
            info!("No DNS records found for {}", domain);
        }
        return Ok(output);
    }
}
