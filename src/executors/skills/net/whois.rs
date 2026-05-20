//! # WHOIS Module
//!
//! This module provides a collection of skills for querying WHOIS databases to retrieve
//! domain registration information:
//!
//! - `WhoisSkill`: Query WHOIS information for a single domain or IP address
//! - `WhoisBatchSkill`: Perform WHOIS queries for multiple domains at once
//! - `WhoisAvailableSkill`: Check if a domain name is available for registration
//!
//! WHOIS is a query and response protocol used to query databases that store registered
//! users or assignees of an Internet resource, such as a domain name or IP address block.
//!
//! # Examples
//!
//! ```
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! // Query WHOIS for a single domain
//! let skill = WhoisSkill;
//! let mut params = HashMap::new();
//! params.insert("query".to_string(), json!("google.com"));
//! params.insert("summary".to_string(), json!(true));
//! // let result = skill.execute(&params).await?;
//! ```

use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use whois_rust::{WhoIsLookupOptions, Whois};

/// Skill for performing WHOIS queries on domains and IP addresses.
///
/// This skill queries WHOIS databases to retrieve registration information
/// including registrar details, creation/expiration dates, name servers,
/// and contact information for domain owners.
///
/// Supports both domain names and IP addresses. Optional parameters allow
/// specifying a custom WHOIS server or requesting only a summary of key fields.
///
/// # Parameters
/// - `query` (required): Domain name or IP address to query
/// - `server` (optional): Custom WHOIS server to use
/// - `summary` (optional): If true, returns only extracted key fields
///
/// # Example
/// ```ignore
/// let skill = WhoisSkill;
/// let mut params = HashMap::new();
/// params.insert("query".to_string(), json!("rust-lang.org"));
/// params.insert("summary".to_string(), json!(true));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct WhoisSkill;

#[async_trait::async_trait]
impl Skill for WhoisSkill {
    fn name(&self) -> &str {
        "whois"
    }

    fn description(&self) -> &str {
        "Query WHOIS databases for domain registration information"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to find domain registration details, including registrar, \
         creation/expiration dates, name servers, and contact information for domain owners."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "Domain name or IP address to query WHOIS information for".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "server".to_string(),
                param_type: "string".to_string(),
                description: "Custom WHOIS server to use (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("whois.verisign-grs.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "summary".to_string(),
                param_type: "boolean".to_string(),
                description: "Return only summary information (extract key fields)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "whois",
            "parameters": {
                "query": "rust-lang.org",
                "summary": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Domain: RUST-LANG.ORG\nRegistrar: Gandi SAS\nCreation Date: 2006-05-07\nExpiration Date: 2025-05-07\nName Servers: ns-aws.gandi.net, ns-206-b.gandi.net, ns-186-c.gandi.net\nStatus: clientTransferProhibited".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let summary = parameters
            .get("summary")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let whois = Whois::default()?;
        let mut opts = WhoIsLookupOptions::default();
        if let Some(server) = parameters.get("server").and_then(|v| v.as_str()) {
            opts.whois_server = Some(server.to_string());
        }
        match whois.lookup_with_options(query, opts) {
            Ok(result) => {
                if summary {
                    extract_summary(&result)
                } else {
                    Ok(result)
                }
            }
            Err(e) => Ok(format!("WHOIS query failed for '{}': {}", query, e)),
        }
    }
}

/// Skill for performing batch WHOIS queries on multiple domains.
///
/// This skill allows querying WHOIS information for multiple domains in a single
/// operation. It's particularly useful when you need to check registration status
/// or gather information about several domains at once.
///
/// # Parameters
/// - `domains` (required): Array of domain names to query
/// - `summary` (optional): If true, returns only summary for each domain (default: true)
///
/// # Example
/// ```ignore
/// let skill = WhoisBatchSkill;
/// let mut params = HashMap::new();
/// params.insert("domains".to_string(), json!(["google.com", "github.com"]));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct WhoisBatchSkill;

#[async_trait::async_trait]
impl Skill for WhoisBatchSkill {
    fn name(&self) -> &str {
        "whois_batch"
    }

    fn description(&self) -> &str {
        "Perform WHOIS queries for multiple domains at once"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to check WHOIS information for several domains at once"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "domains".to_string(),
                param_type: "array".to_string(),
                description: "List of domain names to query".to_string(),
                required: true,
                default: None,
                example: Some(json!(["google.com", "microsoft.com", "amazon.com"])),
                enum_values: None,
            },
            SkillParameter {
                name: "summary".to_string(),
                param_type: "boolean".to_string(),
                description: "Return only summary for each domain".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "whois_batch",
            "parameters": {
                "domains": ["google.com", "github.com", "rust-lang.org"]
            }
        })
    }

    fn example_output(&self) -> String {
        "WHOIS Batch Results:\n\ngoogle.com:\n  Registrar: MarkMonitor Inc.\n  Creation: 1997-09-15\n  Expiry: 2028-09-14\n\ngithub.com:\n  Registrar: MarkMonitor Inc.\n  Creation: 2007-10-09\n  Expiry: 2025-10-09".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let domains = parameters
            .get("domains")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: domains (array)"))?;
        let summary = parameters
            .get("summary")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let whois = Whois::default()?;
        let mut results = vec!["WHOIS Batch Results:\n".to_string()];
        for domain_value in domains {
            if let Some(domain) = domain_value.as_str() {
                results.push(format!("\n{}:", domain));
                match whois.lookup(domain) {
                    Ok(result) => {
                        if summary {
                            if let Ok(summary_text) = extract_summary(&result) {
                                for line in summary_text.lines() {
                                    if !line.is_empty() {
                                        results.push(format!("  {}", line));
                                    }
                                }
                            } else {
                                results.push("  Failed to parse summary".to_string());
                            }
                        } else {
                            for line in result.lines().take(10) {
                                results.push(format!("  {}", line));
                            }
                            if result.lines().count() > 10 {
                                results.push("  ...".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        results.push(format!("  Error: {}", e));
                    }
                }
            }
        }
        Ok(results.join("\n"))
    }
}

/// Skill for checking domain name availability.
///
/// This skill determines whether a domain name is available for registration
/// by performing a WHOIS query. If no registration record is found or the
/// response indicates the domain is free, it reports the domain as available.
///
/// # Parameters
/// - `domain` (required): Domain name to check availability
///
/// # Example
/// ```ignore
/// let skill = WhoisAvailableSkill;
/// let mut params = HashMap::new();
/// params.insert("domain".to_string(), json!("example-domain.com"));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct WhoisAvailableSkill;

#[async_trait::async_trait]
impl Skill for WhoisAvailableSkill {
    fn name(&self) -> &str {
        "whois_available"
    }

    fn description(&self) -> &str {
        "Check if a domain name is available for registration"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you want to check if a domain name is already taken or available to register"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "domain".to_string(),
            param_type: "string".to_string(),
            description: "Domain name to check availability".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("example-domain-12345.com".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "whois_available",
            "parameters": {
                "domain": "mynewdomain123.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "Domain: mynewdomain123.com\nStatus: ✓ Available\nNo WHOIS record found, domain appears to be available for registration".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let domain = parameters
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: domain"))?;
        let whois = Whois::default()?;
        match whois.lookup(domain) {
            Ok(result) => {
                if result.to_lowercase().contains("no match")
                    || result.to_lowercase().contains("not found")
                    || result.to_lowercase().contains("is free")
                {
                    Ok(format!(
                        "Domain: {}\nStatus: ✓ Available\n{}",
                        domain,
                        "No active registration found".to_string()
                    ))
                } else {
                    let creation = extract_field(&result, "Creation Date:");
                    let expiry = extract_field(&result, "Expiry Date:");
                    let registrar = extract_field(&result, "Registrar:");

                    let mut output = format!("Domain: {}\nStatus: ✗ Registered\n", domain);
                    if let Some(c) = creation {
                        output.push_str(&format!("Creation Date: {}\n", c));
                    }
                    if let Some(e) = expiry {
                        output.push_str(&format!("Expiry Date: {}\n", e));
                    }
                    if let Some(r) = registrar {
                        output.push_str(&format!("Registrar: {}\n", r));
                    }

                    Ok(output)
                }
            }
            Err(_) => Ok(format!(
                "Domain: {}\nStatus: ✓ Available\nNo WHOIS record found, domain appears to be available for registration",
                domain
            )),
        }
    }
}

/// Extracts a summary of key WHOIS fields from the raw WHOIS response.
///
/// This function parses the WHOIS output and extracts commonly requested
/// fields such as Domain Name, Registrar, Creation Date, Expiry Date,
/// Name Server, Status, and others.
///
/// If no known fields are found, it returns the first 20 lines of the response
/// as a fallback.
///
/// # Arguments
/// * `whois_data` - The raw WHOIS response string
///
/// # Returns
/// A string containing the extracted summary lines, joined by newlines
fn extract_summary(whois_data: &str) -> Result<String> {
    let mut summary = Vec::new();
    let fields = vec![
        "Domain Name:",
        "Registrar:",
        "Creation Date:",
        "Expiry Date:",
        "Updated Date:",
        "Name Server:",
        "Status:",
        "Registrant:",
        "Registry Domain ID:",
        "Domain Status:",
        "Registrar URL:",
    ];
    for line in whois_data.lines() {
        for field in &fields {
            if line.starts_with(field) {
                summary.push(line.to_string());
                break;
            }
        }
    }
    if summary.is_empty() {
        Ok(whois_data
            .lines()
            .take(20)
            .collect::<Vec<&str>>()
            .join("\n"))
    } else {
        Ok(summary.join("\n"))
    }
}

/// Extracts a specific field value from the raw WHOIS response.
///
/// This function searches for a line that starts with the given field name
/// and returns the value portion (everything after the field name).
///
/// # Arguments
/// * `whois_data` - The raw WHOIS response string
/// * `field_name` - The field name to search for (including colon)
///
/// # Returns
/// `Some(String)` containing the field value if found, `None` otherwise
fn extract_field(whois_data: &str, field_name: &str) -> Option<String> {
    for line in whois_data.lines() {
        if line.starts_with(field_name) {
            let value = line.trim_start_matches(field_name).trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_WHOIS_RESPONSE: &str = r#"Domain Name: EXAMPLE.COM
Registry Domain ID: 123456789_DOMAIN_COM-VRSN
Registrar: Example Registrar, Inc.
Registrar IANA ID: 1234
Registrar URL: http://www.example-registrar.com
Creation Date: 1995-08-14T04:00:00Z
Expiry Date: 2025-08-13T04:00:00Z
Updated Date: 2023-08-14T00:00:00Z
Name Server: NS1.EXAMPLE.COM
Name Server: NS2.EXAMPLE.COM
Status: clientTransferProhibited
Registrant: Example Corp.
"#;

    const SAMPLE_AVAILABLE_RESPONSE: &str = r#"No match for domain "AVAILABLE-DOMAIN.COM".
>>> Last update of whois database: 2024-01-01T00:00:00Z <<<
"#;

    /// Test extract_summary function with standard WHOIS data
    #[test]
    fn test_extract_summary() {
        let summary = extract_summary(SAMPLE_WHOIS_RESPONSE).unwrap();
        assert!(summary.contains("Domain Name: EXAMPLE.COM"));
        assert!(summary.contains("Registrar: Example Registrar, Inc."));
        assert!(summary.contains("Creation Date: 1995-08-14T04:00:00Z"));
        assert!(summary.contains("Expiry Date: 2025-08-13T04:00:00Z"));
        assert!(summary.contains("Name Server: NS1.EXAMPLE.COM"));
        assert!(summary.contains("Status: clientTransferProhibited"));
        assert!(!summary.contains("Registrar IANA ID: 1234"));
    }

    /// Test extract_summary with empty or minimal data
    #[test]
    fn test_extract_summary_fallback() {
        let short_response = "This is a raw WHOIS response\nWithout standard fields\nThird line";
        let fallback = extract_summary(short_response).unwrap();
        assert_eq!(fallback, short_response);
        let empty_response = "";
        let empty_result = extract_summary(empty_response).unwrap();
        assert_eq!(empty_result, "");
    }

    /// Test extract_field function for various field types
    #[test]
    fn test_extract_field() {
        let creation_date = extract_field(SAMPLE_WHOIS_RESPONSE, "Creation Date:");
        assert_eq!(creation_date, Some("1995-08-14T04:00:00Z".to_string()));
        let registrar = extract_field(SAMPLE_WHOIS_RESPONSE, "Registrar:");
        assert_eq!(registrar, Some("Example Registrar, Inc.".to_string()));
        let name_server = extract_field(SAMPLE_WHOIS_RESPONSE, "Name Server:");
        assert_eq!(name_server, Some("NS1.EXAMPLE.COM".to_string()));
        let non_existent = extract_field(SAMPLE_WHOIS_RESPONSE, "NonExistent:");
        assert_eq!(non_existent, None);
        let no_match = extract_field(SAMPLE_AVAILABLE_RESPONSE, "Domain Name:");
        assert_eq!(no_match, None);
    }

    /// Test extract_field with edge cases
    #[test]
    fn test_extract_field_edge_cases() {
        let empty_field_data = "Empty Field: \nNext line";
        let empty_value = extract_field(empty_field_data, "Empty Field:");
        assert_eq!(empty_value, None);
        let whitespace_data = "Whitespace:    value with spaces   \n";
        let whitespace_value = extract_field(whitespace_data, "Whitespace:");
        assert_eq!(whitespace_value, Some("value with spaces".to_string()));
        let exact_match_data = "Field: exact value\n";
        let exact_value = extract_field(exact_match_data, "Field:");
        assert_eq!(exact_value, Some("exact value".to_string()));
        let partial_data = "FullField: full\nPartial: partial\n";
        let partial_result = extract_field(partial_data, "Full");
        assert_eq!(partial_result, None);
    }

    /// Test domain availability detection patterns
    #[test]
    fn test_availability_patterns() {
        let available_indicators = vec![
            "No match for domain",
            "NOT FOUND",
            "is free",
            "No entries found",
            "Domain not found",
        ];
        for indicator in available_indicators {
            let response = format!("{}: example.com\nAdditional data", indicator);
            let is_available = response.to_lowercase().contains("no match")
                || response.to_lowercase().contains("not found")
                || response.to_lowercase().contains("is free");
            assert!(
                is_available,
                "Indicator '{}' should mark domain as available",
                indicator
            );
        }
        let registered_response = "Domain Name: REGISTERED.COM\nCreation Date: 2020-01-01";
        let is_registered = !(registered_response.to_lowercase().contains("no match")
            || registered_response.to_lowercase().contains("not found")
            || registered_response.to_lowercase().contains("is free"));
        assert!(is_registered);
    }
}
