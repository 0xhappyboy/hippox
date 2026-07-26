//! CVE vulnerability query driver
//!
//! This driver provides functionality to query CVE vulnerability information by ID or keyword.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::{query_cve, query_cves_by_keyword},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for querying CVE vulnerabilities
#[derive(Debug)]
pub struct CveQueryDriver;
#[async_trait::async_trait]
impl Driver for CveQueryDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_cve_query"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Query CVE vulnerability information by ID or keyword"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to look up CVE vulnerabilities. Provide a CVE ID (e.g., CVE-2024-1234) or a keyword."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "CVE ID (e.g., CVE-2024-1234) or keyword to search".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("CVE-2024-1234".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of results to return (default: 10)".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_cve_query",
            "parameters": {
                "query": "CVE-2024-1234"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "CVE-2024-1234\nSeverity: HIGH\nCVSS: 7.5\nDescription: Buffer overflow in service X\nExploit available: Yes".to_string();
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
        debug!("Executing security_cve_query driver");
        let query = parameters.get("query").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("query"))?;
        let limit = parameters.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        info!("Querying CVE with: '{}', limit: {}", query, limit);
        let is_cve_id = query.starts_with("CVE-") || query.starts_with("cve-");
        let mut result = String::new();
        if is_cve_id {
            debug!("Querying by CVE ID: {}", query);
            if let Some(cve) = query_cve(query) {
                info!("Found CVE: {}", cve.id);
                result.push_str(&format!("CVE: {}\n", cve.id));
                result.push_str(&format!("Severity: {}\n", cve.severity));
                if let Some(score) = cve.cvss_score {
                    result.push_str(&format!("CVSS Score: {:.1}\n", score));
                }
                result.push_str(&format!("Description: {}\n", cve.description));
                result.push_str(&format!("Published: {}\n", cve.published_date.unwrap_or("Unknown".to_string())));
                result.push_str(&format!("Exploit Available: {}\n", if cve.exploit_available { "Yes" } else { "No" }));
                result.push_str("Affected Products:\n");
                for product in &cve.affected_products {
                    result.push_str(&format!("  - {}\n", product));
                }
                result.push_str("References:\n");
                for ref_url in &cve.references {
                    result.push_str(&format!("  - {}\n", ref_url));
                }
            } else {
                info!("No CVE found with ID: {}", query);
                result.push_str(&format!("No CVE found with ID: {}", query));
            }
        } else {
            debug!("Searching CVEs by keyword: {}", query);
            let cves = query_cves_by_keyword(query);
            let results: Vec<_> = cves.into_iter().take(limit).collect();
            if results.is_empty() {
                info!("No CVEs found matching keyword: {}", query);
                result.push_str(&format!("No CVEs found matching keyword: {}", query));
            } else {
                info!("Found {} CVEs matching '{}'", results.len(), query);
                result.push_str(&format!("Found {} CVEs matching '{}':\n\n", results.len(), query));
                for (i, cve) in results.iter().enumerate() {
                    result.push_str(&format!("{}. {}\n", i + 1, cve.id));
                    result.push_str(&format!("   Severity: {}\n", cve.severity));
                    if let Some(score) = cve.cvss_score {
                        result.push_str(&format!("   CVSS: {:.1}\n", score));
                    }
                    result.push_str(&format!("   Description: {}\n", cve.description));
                    result.push_str(&format!("   Exploit: {}\n\n", if cve.exploit_available { "Available" } else { "Not Available" }));
                }
            }
        }
        return Ok(result);
    }
}
