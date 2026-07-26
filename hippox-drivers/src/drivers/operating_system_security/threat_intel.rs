//! Threat intelligence query driver
//!
//! This driver provides functionality to query threat intelligence for IP, domain, or file hash.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::query_threat_intel,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for querying threat intelligence
#[derive(Debug)]
pub struct ThreatIntelDriver;
#[async_trait::async_trait]
impl Driver for ThreatIntelDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_threat_intel"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Query threat intelligence for IP, domain, or file hash"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check if an IP address, domain name, or file hash is known to be malicious."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "indicator".to_string(),
            param_type: "string".to_string(),
            description: "IP address, domain name, or file hash (MD5, SHA-1, SHA-256)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("185.130.5.253".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_threat_intel",
            "parameters": {
                "indicator": "185.130.5.253"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Indicator: 185.130.5.253\nType: ip\nMalicious: Yes\nConfidence: 95%\nThreat Type: malware\nFirst Seen: 2024-01-01\nSource: Internal Threat Intelligence Database".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemSecurity;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing security_threat_intel driver");
        let indicator = parameters.get("indicator").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("indicator"))?;
        info!("Querying threat intelligence for: {}", indicator);
        let result = query_threat_intel(indicator);
        let mut output = String::new();
        output.push_str(&format!("Indicator: {}\n", result.indicator));
        output.push_str(&format!("Type: {}\n", result.indicator_type));
        output.push_str(&format!("Malicious: {}\n", if result.malicious { "Yes" } else { "No" }));
        output.push_str(&format!("Confidence: {:.0}%\n", result.confidence * 100.0));
        output.push_str(&format!("Threat Types: {}\n", result.threat_type.join(", ")));
        if let Some(first_seen) = result.first_seen {
            output.push_str(&format!("First Seen: {}\n", first_seen));
        }
        if let Some(last_seen) = result.last_seen {
            output.push_str(&format!("Last Seen: {}\n", last_seen));
        }
        if !result.related_indicators.is_empty() {
            output.push_str(&format!("Related Indicators: {}\n", result.related_indicators.join(", ")));
        }
        output.push_str(&format!("Source: {}", result.source));
        if result.malicious {
            info!("Threat intel indicates malicious indicator: {}", indicator);
        } else {
            info!("Threat intel indicates legitimate indicator: {}", indicator);
        }
        return Ok(output);
    }
}
