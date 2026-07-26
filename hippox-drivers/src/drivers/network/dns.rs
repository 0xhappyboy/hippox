//! DNS lookup and testing utilities for domain resolution, reverse lookups, batch operations, and DNS server testing.
//!
//! This module provides several skills for performing DNS operations:
//! - `DnsLookupDriver`: Query DNS records for a domain
//! - `ReverseDnsDriver`: Find domain names associated with IP addresses
//! - `DnsBatchLookupDriver`: Perform multiple DNS lookups simultaneously
//! - `DnsTestDriver`: Test DNS server performance and reliability
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::IpAddr;
use tracing::{debug, info};
use trust_dns_proto::rr::{RData, RecordType};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};
/// A skill for performing DNS lookups to resolve domain names to IP addresses
/// and retrieve various DNS record types.
#[derive(Debug)]
pub struct DnsLookupDriver;
#[async_trait::async_trait]
impl Driver for DnsLookupDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "dns_lookup"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Perform DNS queries to resolve domain names to IP addresses and retrieve DNS records"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to resolve domain names, look up DNS records (A, AAAA, MX, TXT, CNAME, NS), \
         or troubleshoot DNS issues. Supports custom DNS servers and various record types."
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "domain".to_string(),
                param_type: "string".to_string(),
                description: "Domain name to query (e.g., example.com)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "record_type".to_string(),
                param_type: "string".to_string(),
                description: "DNS record type (A, AAAA, MX, TXT, CNAME, NS, SOA, PTR)".to_string(),
                required: false,
                default: Some(Value::String("A".to_string())),
                example: Some(Value::String("MX".to_string())),
                enum_values: Some(vec![
                    "A".to_string(),
                    "AAAA".to_string(),
                    "MX".to_string(),
                    "TXT".to_string(),
                    "CNAME".to_string(),
                    "NS".to_string(),
                    "SOA".to_string(),
                    "PTR".to_string(),
                    "ALL".to_string(),
                ]),
            },
            DriverParameter {
                name: "dns_server".to_string(),
                param_type: "string".to_string(),
                description: "Custom DNS server to use (e.g., 8.8.8.8, 1.1.1.1)".to_string(),
                required: false,
                default: Some(Value::String("8.8.8.8".to_string())),
                example: Some(Value::String("1.1.1.1".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "dns_lookup",
            "parameters": {
                "domain": "github.com",
                "record_type": "A"
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "DNS lookup for github.com (A record):\n140.82.112.3\n140.82.114.3\n\nResolved 2 IP addresses\nUsing DNS server: 8.8.8.8".to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes a DNS lookup query with the provided parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing dns_lookup driver");
        let domain = parameters.get("domain").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("domain"))?;
        let record_type_str = parameters.get("record_type").and_then(|v| v.as_str()).unwrap_or("A");
        let dns_server = parameters.get("dns_server").and_then(|v| v.as_str()).unwrap_or("8.8.8.8");
        info!("DNS lookup: domain={}, record_type={}, dns_server={}", domain, record_type_str, dns_server);
        let record_type = match record_type_str {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "MX" => RecordType::MX,
            "TXT" => RecordType::TXT,
            "CNAME" => RecordType::CNAME,
            "NS" => RecordType::NS,
            "SOA" => RecordType::SOA,
            "PTR" => RecordType::PTR,
            "ALL" => RecordType::ANY,
            _ => RecordType::A,
        };
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &[dns_server.parse().map_err(|e| DriverError::execution(format!("Invalid DNS server: {}", e)))?],
                53,
                true,
            ),
        );
        let resolver_opts = ResolverOpts::default();
        let resolver =
            Resolver::new(resolver_config, resolver_opts).map_err(|e| DriverError::execution(format!("Failed to create resolver: {}", e)))?;
        let response = resolver.lookup(domain, record_type).map_err(|e| DriverError::execution(format!("DNS lookup failed: {}", e)))?;
        let mut result = format!("DNS lookup for {} ({} record):\n", domain, record_type_str);
        let mut count = 0;
        for record in response.iter() {
            match record {
                RData::A(ip) => {
                    result.push_str(&format!("  A: {}\n", ip));
                    count += 1;
                }
                RData::AAAA(ip) => {
                    result.push_str(&format!("  AAAA: {}\n", ip));
                    count += 1;
                }
                RData::MX(mx) => {
                    result.push_str(&format!("  MX: {} (priority {})\n", mx.exchange(), mx.preference()));
                    count += 1;
                }
                RData::TXT(txt) => {
                    result.push_str(&format!("  TXT: {}\n", txt.txt_data().iter().map(|d| String::from_utf8_lossy(d)).collect::<Vec<_>>().join("")));
                    count += 1;
                }
                RData::CNAME(cname) => {
                    result.push_str(&format!("  CNAME: {}\n", cname));
                    count += 1;
                }
                RData::NS(ns) => {
                    result.push_str(&format!("  NS: {}\n", ns));
                    count += 1;
                }
                RData::SOA(soa) => {
                    result.push_str(&format!("  SOA: {} (serial: {})\n", soa.mname(), soa.serial()));
                    count += 1;
                }
                _ => {
                    result.push_str(&format!("  {:?}\n", record));
                    count += 1;
                }
            }
        }
        result.push_str(&format!("\nFound {} records\nUsing DNS server: {}", count, dns_server));
        info!("DNS lookup completed: found {} records", count);
        return Ok(result);
    }
}
/// A skill for performing reverse DNS lookups to find domain names associated with IP addresses.
#[derive(Debug)]
pub struct ReverseDnsDriver;
#[async_trait::async_trait]
impl Driver for ReverseDnsDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "reverse_dns"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Perform reverse DNS lookup to find domain names associated with IP addresses"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when you have an IP address and want to find its hostname"
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "ip".to_string(),
                param_type: "string".to_string(),
                description: "IP address to reverse lookup".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("8.8.8.8".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "dns_server".to_string(),
                param_type: "string".to_string(),
                description: "Custom DNS server to use".to_string(),
                required: false,
                default: Some(Value::String("8.8.8.8".to_string())),
                example: Some(Value::String("1.1.1.1".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "reverse_dns",
            "parameters": {
                "ip": "8.8.8.8"
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "Reverse DNS lookup for 8.8.8.8:\ndns.google\n\nPTR record found".to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes a reverse DNS lookup for the given IP address
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing reverse_dns driver");
        let ip = parameters.get("ip").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("ip"))?;
        let dns_server = parameters.get("dns_server").and_then(|v| v.as_str()).unwrap_or("8.8.8.8");
        info!("Reverse DNS lookup: ip={}, dns_server={}", ip, dns_server);
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &[dns_server.parse().map_err(|e| DriverError::execution(format!("Invalid DNS server: {}", e)))?],
                53,
                true,
            ),
        );
        let resolver_opts = ResolverOpts::default();
        let resolver =
            Resolver::new(resolver_config, resolver_opts).map_err(|e| DriverError::execution(format!("Failed to create resolver: {}", e)))?;
        let ip_addr: IpAddr = ip.parse().map_err(|e| DriverError::execution(format!("Invalid IP address: {}", e)))?;
        let reverse_domain = match ip_addr {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                format!("{}.{}.{}.{}.in-addr.arpa", octets[3], octets[2], octets[1], octets[0])
            }
            IpAddr::V6(ipv6) => {
                let hex_str: String = ipv6.octets().iter().rev().flat_map(|&b| format!("{:02x}.", b).chars().collect::<Vec<_>>()).collect();
                format!("{}.ip6.arpa", hex_str.trim_end_matches('.'))
            }
        };
        let response =
            resolver.lookup(&reverse_domain, RecordType::PTR).map_err(|e| DriverError::execution(format!("Reverse DNS lookup failed: {}", e)))?;
        let mut result = format!("Reverse DNS lookup for {}:\n", ip);
        let mut found = false;
        for record in response.iter() {
            if let RData::PTR(ptr) = record {
                result.push_str(&format!("  {}\n", ptr));
                found = true;
            }
        }
        if !found {
            result.push_str("  No PTR record found\n");
            info!("No PTR record found for {}", ip);
        } else {
            info!("PTR record found for {}", ip);
        }
        return Ok(result);
    }
}
/// A skill for performing DNS lookups for multiple domains simultaneously.
#[derive(Debug)]
pub struct DnsBatchLookupDriver;
#[async_trait::async_trait]
impl Driver for DnsBatchLookupDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "dns_batch_lookup"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Perform DNS lookups for multiple domains simultaneously"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to resolve multiple domain names at once"
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "domains".to_string(),
                param_type: "array".to_string(),
                description: "List of domain names to query".to_string(),
                required: true,
                default: None,
                example: Some(json!(["google.com", "github.com", "rust-lang.org"])),
                enum_values: None,
            },
            DriverParameter {
                name: "record_type".to_string(),
                param_type: "string".to_string(),
                description: "DNS record type (default: A)".to_string(),
                required: false,
                default: Some(Value::String("A".to_string())),
                example: Some(Value::String("A".to_string())),
                enum_values: Some(vec!["A".to_string(), "AAAA".to_string(), "MX".to_string()]),
            },
        ];
    }
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "dns_batch_lookup",
            "parameters": {
                "domains": ["google.com", "microsoft.com", "amazon.com"]
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "DNS batch lookup results:\n\ngoogle.com:\n  142.250.185.46\n\nmicrosoft.com:\n  20.70.246.20\n\namazon.com:\n  205.251.242.103"
            .to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes batch DNS lookups for multiple domains
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing dns_batch_lookup driver");
        let domains = parameters.get("domains").and_then(|v| v.as_array()).ok_or_else(|| DriverError::missing_parameter("domains (array)"))?;
        let record_type_str = parameters.get("record_type").and_then(|v| v.as_str()).unwrap_or("A");
        info!("Batch DNS lookup: {} domains, record_type={}", domains.len(), record_type_str);
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
            .map_err(|e| DriverError::execution(format!("Failed to create resolver: {}", e)))?;
        let record_type = match record_type_str {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "MX" => RecordType::MX,
            _ => RecordType::A,
        };
        let mut results = Vec::new();
        results.push("DNS batch lookup results:\n".to_string());
        for domain_value in domains {
            if let Some(domain) = domain_value.as_str() {
                results.push(format!("\n{}:", domain));
                match resolver.lookup(domain, record_type) {
                    Ok(response) => {
                        for record in response.iter() {
                            match record {
                                RData::A(ip) => results.push(format!("  {}", ip)),
                                RData::AAAA(ip) => results.push(format!("  {}", ip)),
                                RData::MX(mx) => results.push(format!("  MX: {} (priority {})", mx.exchange(), mx.preference())),
                                _ => results.push(format!("  {:?}", record)),
                            }
                        }
                    }
                    Err(e) => {
                        results.push(format!("  Failed: {}", e));
                        debug!("DNS lookup failed for {}: {}", domain, e);
                    }
                }
            }
        }
        info!("Batch DNS lookup completed for {} domains", domains.len());
        return Ok(results.join("\n"));
    }
}
/// A skill for testing DNS server performance and reliability.
#[derive(Debug)]
pub struct DnsTestDriver;
#[async_trait::async_trait]
impl Driver for DnsTestDriver {
    /// Returns the name identifier for this skill
    fn name(&self) -> &str {
        "dns_test"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Test DNS server performance and reliability by querying multiple domains"
    }
    /// Returns a usage hint explaining when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when you want to test DNS server response times and reliability"
    }
    /// Returns the list of parameters accepted by this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "dns_server".to_string(),
                param_type: "string".to_string(),
                description: "DNS server to test".to_string(),
                required: false,
                default: Some(Value::String("8.8.8.8".to_string())),
                example: Some(Value::String("1.1.1.1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "domain".to_string(),
                param_type: "string".to_string(),
                description: "Domain to query for testing".to_string(),
                required: false,
                default: Some(Value::String("google.com".to_string())),
                example: Some(Value::String("cloudflare.com".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example JSON call demonstrating how to invoke this skill
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "dns_test",
            "parameters": {
                "dns_server": "8.8.8.8"
            }
        }));
    }
    /// Returns an example output that this skill might produce
    fn example_output(&self) -> String {
        return "DNS Server Test: 8.8.8.8\nQuerying google.com...\nResponse time: 24ms\nStatus: ✓ Healthy".to_string();
    }
    /// Returns the category of this skill for organizational purposes
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes a DNS server performance test
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing dns_test driver");
        let dns_server = parameters.get("dns_server").and_then(|v| v.as_str()).unwrap_or("8.8.8.8");
        let domain = parameters.get("domain").and_then(|v| v.as_str()).unwrap_or("google.com");
        info!("DNS test: server={}, domain={}", dns_server, domain);
        let start = std::time::Instant::now();
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &[dns_server.parse().map_err(|e| DriverError::execution(format!("Invalid DNS server: {}", e)))?],
                53,
                true,
            ),
        );
        let resolver_opts = ResolverOpts::default();
        let resolver =
            Resolver::new(resolver_config, resolver_opts).map_err(|e| DriverError::execution(format!("Failed to create resolver: {}", e)))?;
        match resolver.lookup(domain, RecordType::A) {
            Ok(response) => {
                let elapsed = start.elapsed();
                let ips: Vec<String> = response.iter().filter_map(|r| if let RData::A(ip) = r { Some(ip.to_string()) } else { None }).collect();
                info!("DNS test successful: {}ms, {} IPs", elapsed.as_millis(), ips.len());
                return Ok(format!(
                    "DNS Server Test: {}\nQuerying {}...\nResponse time: {}ms\nResolved to: {}\nStatus: ✓ Healthy",
                    dns_server,
                    domain,
                    elapsed.as_millis(),
                    ips.join(", ")
                ));
            }
            Err(e) => {
                let elapsed = start.elapsed();
                info!("DNS test failed: {}ms, error: {}", elapsed.as_millis(), e);
                return Ok(format!(
                    "DNS Server Test: {}\nQuerying {}...\nResponse time: {}ms\nError: {}\nStatus: ✗ Failed",
                    dns_server,
                    domain,
                    elapsed.as_millis(),
                    e
                ));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[tokio::test]
    async fn test_dns_lookup_a_record() {
        let skill = DnsLookupDriver;
        let mut params = HashMap::new();
        params.insert("domain".to_string(), json!("google.com"));
        params.insert("record_type".to_string(), json!("A"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("DNS lookup for google.com (A record):"));
        assert!(output.contains("A:"));
        assert!(output.contains("Using DNS server: 8.8.8.8"));
    }
    #[tokio::test]
    async fn test_reverse_dns_lookup() {
        let skill = ReverseDnsDriver;
        let mut params = HashMap::new();
        params.insert("ip".to_string(), json!("8.8.8.8"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Reverse DNS lookup for 8.8.8.8:"));
        assert!(output.contains("dns") || output.contains("No PTR record"));
    }
    #[tokio::test]
    async fn test_dns_test_skill() {
        let skill = DnsTestDriver;
        let mut params = HashMap::new();
        params.insert("dns_server".to_string(), json!("8.8.8.8"));
        params.insert("domain".to_string(), json!("google.com"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("DNS Server Test: 8.8.8.8"));
        assert!(output.contains("Querying google.com..."));
        assert!(output.contains("Response time:"));
    }
}
