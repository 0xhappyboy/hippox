//! Phishing URL detection driver
//!
//! This driver provides functionality to detect if a URL is a phishing link.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::detect_phishing,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for detecting phishing URLs
#[derive(Debug)]
pub struct PhishingDetectDriver;
#[async_trait::async_trait]
impl Driver for PhishingDetectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_phishing_detect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Detect if a URL is a phishing link"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to analyze a URL for phishing indicators. Checks for suspicious keywords, domain spoofing, URL shorteners, and more."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "url".to_string(),
            param_type: "string".to_string(),
            description: "URL to check for phishing".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("https://secure-login.example.com".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_phishing_detect",
            "parameters": {
                "url": "https://secure-login.example.com"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "URL: https://secure-login.example.com\nPhishing: Yes\nConfidence: 85%\nDomain Reputation: Suspicious\nReasons:\n- Contains suspicious keyword: Common phishing keyword\n- Potential domain spoofing with suspicious keywords".to_string();
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
        debug!("Executing security_phishing_detect driver");
        let url = parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("url"))?;
        info!("Detecting phishing for URL: {}", url);
        let result = detect_phishing(url);
        let mut output = String::new();
        output.push_str(&format!("URL: {}\n", result.url));
        output.push_str(&format!("Phishing: {}\n", if result.is_phishing { "Yes" } else { "No" }));
        output.push_str(&format!("Confidence: {:.0}%\n", result.confidence * 100.0));
        output.push_str(&format!("Domain Reputation: {}\n", result.domain_reputation));
        if !result.reasons.is_empty() {
            output.push_str("\nReasons:\n");
            for reason in &result.reasons {
                output.push_str(&format!("- {}\n", reason));
            }
        }
        if result.is_phishing {
            info!("URL is phishing: {}", url);
            output.push_str("\nThis URL appears to be a phishing attempt. Do not enter any credentials!");
        } else {
            info!("URL appears legitimate: {}", url);
            output.push_str("\nThis URL appears legitimate based on current analysis.");
        }
        return Ok(output);
    }
}
