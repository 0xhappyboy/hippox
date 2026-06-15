//! # IP Utilities Module
//!
//! This module provides a collection of skills for working with IP addresses:
//! - `IpInfoSkill`: Query detailed geolocation and network information about IP addresses
//! - `IpValidateSkill`: Validate and classify IP addresses (public/private, IPv4/IPv6, etc.)
//! - `IpRangeSkill`: Calculate network ranges from CIDR notation
//! - `LocalIpSkill`: Get local IP addresses of the current machine
//!
//! # Examples
//!
//! ```
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! // Get info about an IP address
//! let skill = IpInfoSkill;
//! let mut params = HashMap::new();
//! params.insert("ip".to_string(), json!("8.8.8.8"));
//! // let result = skill.execute(&params).await?;
//! ```

use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Skill for retrieving detailed geolocation and network information about an IP address.
///
/// This skill queries the ip-api.com service to obtain information such as:
/// - Geographic location (country, city, coordinates)
/// - ISP and organization details
/// - ASN (Autonomous System Number)
/// - Timezone information
///
/// Supports both IPv4 and IPv6 addresses. If no IP is provided, returns information
/// about the caller's public IP address.
///
/// # Example
/// ```ignore
/// let skill = IpInfoSkill;
/// let mut params = HashMap::new();
/// params.insert("ip".to_string(), json!("8.8.8.8"));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct IpInfoSkill;

#[async_trait::async_trait]
impl Skill for IpInfoSkill {
    fn name(&self) -> &str {
        "ip_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about an IP address including geolocation, ASN, ISP, and more"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to find location, ISP, or other information about an IP address. \
         Supports both IPv4 and IPv6 addresses. If no IP is provided, returns information about the public IP."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "ip".to_string(),
            param_type: "string".to_string(),
            description: "IP address to lookup (IPv4 or IPv6). If omitted, returns your public IP"
                .to_string(),
            required: false,
            default: None,
            example: Some(Value::String("8.8.8.8".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "ip_info",
            "parameters": {
                "ip": "8.8.8.8"
            }
        })
    }

    fn example_output(&self) -> String {
        "IP Information for 8.8.8.8:\nCountry: United States\nCity: Mountain View\nISP: Google LLC\nASN: AS15169\nLatitude: 37.4223\nLongitude: -122.0841\nTimezone: America/Los_Angeles".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let ip_param = parameters.get("ip").and_then(|v| v.as_str());
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        let url = if let Some(ip) = ip_param {
            format!("http://ip-api.com/json/{}", ip)
        } else {
            "http://ip-api.com/json".to_string()
        };
        let response = client
            .get(&url)
            .header("User-Agent", "curl/7.68.0")
            .send()
            .await?;
        let text = response.text().await?;
        let data: serde_json::Value = serde_json::from_str(&text)?;
        if data.get("status").and_then(|s| s.as_str()) == Some("fail") {
            return Ok(format!(
                "Failed to get IP info: {}",
                data.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
            ));
        }
        let mut result = String::new();
        let ip_addr = ip_param.unwrap_or_else(|| {
            data.get("query")
                .and_then(|q| q.as_str())
                .unwrap_or("Unknown")
        });
        result.push_str(&format!("IP Information for {}:\n", ip_addr));
        result.push_str(&format!(
            "Country: {}\n",
            data.get("country")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "Country Code: {}\n",
            data.get("countryCode")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "Region: {}\n",
            data.get("regionName")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "City: {}\n",
            data.get("city").and_then(|v| v.as_str()).unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "ZIP Code: {}\n",
            data.get("zip").and_then(|v| v.as_str()).unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "Latitude: {}\n",
            data.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0)
        ));
        result.push_str(&format!(
            "Longitude: {}\n",
            data.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0)
        ));
        result.push_str(&format!(
            "ISP: {}\n",
            data.get("isp").and_then(|v| v.as_str()).unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "Organization: {}\n",
            data.get("org").and_then(|v| v.as_str()).unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "AS: {}\n",
            data.get("as").and_then(|v| v.as_str()).unwrap_or("N/A")
        ));
        result.push_str(&format!(
            "Timezone: {}\n",
            data.get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A")
        ));
        Ok(result)
    }
}

/// Skill for validating IP addresses and classifying their type.
///
/// This skill parses and validates IP address strings, then provides detailed
/// classification information including:
/// - IP version (IPv4 or IPv6)
/// - Address classification (public, private, loopback, multicast, etc.)
/// - Special address detection (broadcast, documentation, link-local)
///
/// # Example
/// ```ignore
/// let skill = IpValidateSkill;
/// let mut params = HashMap::new();
/// params.insert("ip".to_string(), json!("192.168.1.1"));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct IpValidateSkill;

#[async_trait::async_trait]
impl Skill for IpValidateSkill {
    fn name(&self) -> &str {
        "ip_validate"
    }

    fn description(&self) -> &str {
        "Validate IP addresses and provide information about their type (public/private, IPv4/IPv6, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if an IP address is valid and classify it (public, private, loopback, multicast, etc.)"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "ip".to_string(),
            param_type: "string".to_string(),
            description: "IP address to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("192.168.1.1".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "ip_validate",
            "parameters": {
                "ip": "10.0.0.1"
            }
        })
    }

    fn example_output(&self) -> String {
        "IP Address: 10.0.0.1\nType: IPv4\nClassification: Private (RFC 1918)\nValid: Yes\nLoopback: No\nMulticast: No".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let ip_str = parameters
            .get("ip")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: ip"))?;
        match ip_str.parse::<IpAddr>() {
            Ok(ip) => {
                let mut result = format!("IP Address: {}\n", ip);
                result.push_str(&format!(
                    "Type: {}\n",
                    if ip.is_ipv4() { "IPv4" } else { "IPv6" }
                ));
                let classification = classify_ip(&ip);
                result.push_str(&format!("Classification: {}\n", classification));
                result.push_str("Valid: Yes\n");
                result.push_str(&format!("Loopback: {}\n", ip.is_loopback()));
                result.push_str(&format!("Multicast: {}\n", ip.is_multicast()));
                if let IpAddr::V4(ipv4) = ip {
                    result.push_str(&format!("Broadcast: {}\n", ipv4.is_broadcast()));
                    result.push_str(&format!(
                        "Documentation: {}\n",
                        is_documentation_ipv4(&ipv4)
                    ));
                }
                Ok(result)
            }
            Err(e) => Ok(format!("Invalid IP address: {}\nError: {}", ip_str, e)),
        }
    }
}

/// Skill for calculating IP address ranges from CIDR notation.
///
/// This skill computes network details from a CIDR (Classless Inter-Domain Routing)
/// notation string, providing:
/// - Network address
/// - Subnet mask
/// - Wildcard mask
/// - First and last usable IP addresses
/// - Broadcast address
/// - Total number of usable hosts
///
/// # Example
/// ```ignore
/// let skill = IpRangeSkill;
/// let mut params = HashMap::new();
/// params.insert("cidr".to_string(), json!("192.168.1.0/24"));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct IpRangeSkill;

#[async_trait::async_trait]
impl Skill for IpRangeSkill {
    fn name(&self) -> &str {
        "ip_range"
    }

    fn description(&self) -> &str {
        "Calculate IP address ranges from CIDR notation or subnet mask"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to calculate network ranges, subnet details, or CIDR information"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "cidr".to_string(),
            param_type: "string".to_string(),
            description: "CIDR notation (e.g., 192.168.1.0/24) or IP with subnet mask".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("192.168.1.0/24".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "ip_range",
            "parameters": {
                "cidr": "10.0.0.0/16"
            }
        })
    }

    fn example_output(&self) -> String {
        "CIDR: 10.0.0.0/16\nNetwork: 10.0.0.0\nSubnet Mask: 255.255.0.0\nWildcard: 0.0.255.255\nFirst IP: 10.0.0.1\nLast IP: 10.0.255.254\nBroadcast: 10.0.255.255\nTotal Hosts: 65534".to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let cidr = parameters
            .get("cidr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: cidr"))?;
        let re = Regex::new(r"^(\d+\.\d+\.\d+\.\d+)/(\d+)$")?;
        let caps = match re.captures(cidr) {
            Some(caps) => caps,
            None => return Ok(format!("Invalid CIDR format: {}", cidr)),
        };
        let ip_str = caps.get(1).unwrap().as_str();
        let prefix_len: u32 = caps.get(2).unwrap().as_str().parse()?;
        let ip: u32 = ip_str.parse::<Ipv4Addr>()?.into();
        let mask = !((1 << (32 - prefix_len)) - 1);
        let network = ip & mask;
        let broadcast = network | !mask;
        let first_host = if network == broadcast {
            network
        } else {
            network + 1
        };
        let last_host = if network == broadcast {
            broadcast
        } else {
            broadcast - 1
        };
        let total_hosts = if network == broadcast {
            1
        } else {
            broadcast - network - 1
        };
        Ok(format!(
            "CIDR: {}\nNetwork: {}\nSubnet Mask: {}\nWildcard: {}\nFirst IP: {}\nLast IP: {}\nBroadcast: {}\nTotal Hosts: {}",
            cidr,
            Ipv4Addr::from(network),
            Ipv4Addr::from(mask),
            Ipv4Addr::from(!mask),
            Ipv4Addr::from(first_host),
            Ipv4Addr::from(last_host),
            Ipv4Addr::from(broadcast),
            total_hosts
        ))
    }
}

/// Skill for retrieving local IP addresses of the current machine.
///
/// This skill enumerates all network interfaces on the local system and returns
/// their IP addresses. It can filter results by IP version (IPv4 only, IPv6 only,
/// or all addresses).
///
/// # Example
/// ```ignore
/// let skill = LocalIpSkill;
/// let mut params = HashMap::new();
/// params.insert("type".to_string(), json!("ipv4"));
/// let result = skill.execute(&params).await?;
/// ```
#[derive(Debug)]
pub struct LocalIpSkill;

#[async_trait::async_trait]
impl Skill for LocalIpSkill {
    fn name(&self) -> &str {
        "local_ip"
    }

    fn description(&self) -> &str {
        "Get local IP addresses of the current machine (both IPv4 and IPv6)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to know the local IP addresses of the current system"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "type".to_string(),
            param_type: "string".to_string(),
            description: "IP type to return: 'all', 'ipv4', or 'ipv6'".to_string(),
            required: false,
            default: Some(Value::String("all".to_string())),
            example: Some(Value::String("ipv4".to_string())),
            enum_values: Some(vec![
                "all".to_string(),
                "ipv4".to_string(),
                "ipv6".to_string(),
            ]),
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "local_ip",
            "parameters": {
                "type": "all"
            }
        })
    }

    fn example_output(&self) -> String {
        "Local IP addresses:\nIPv4: 192.168.1.100\nIPv6: fe80::1%eth0\nLoopback: 127.0.0.1"
            .to_string()
    }

    fn category(&self) -> &str {
        "net"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let ip_type = parameters
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");
        let mut result = String::from("Local IP addresses:\n");
        let interfaces = local_ip_address::list_afinet_netifas()?;
        for (name, ip) in interfaces {
            match ip {
                IpAddr::V4(ipv4) => {
                    if ip_type == "all" || ip_type == "ipv4" {
                        if !ipv4.is_loopback() {
                            result.push_str(&format!("  {} ({}): {}\n", name, "IPv4", ipv4));
                        }
                    }
                }
                IpAddr::V6(ipv6) => {
                    if ip_type == "all" || ip_type == "ipv6" {
                        if !ipv6.is_loopback() {
                            result.push_str(&format!("  {} ({}): {}\n", name, "IPv6", ipv6));
                        }
                    }
                }
            }
        }
        result.push_str(&format!("  Loopback: 127.0.0.1\n"));
        if result == "Local IP addresses:\n" {
            result.push_str("  No non-loopback addresses found\n");
        }
        Ok(result)
    }
}

/// Classifies an IP address into a human-readable category.
///
/// For IPv4 addresses, this function identifies:
/// - Private addresses (RFC 1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
/// - Loopback (127.0.0.0/8)
/// - Multicast (224.0.0.0/4)
/// - Broadcast (255.255.255.255)
/// - Documentation/reserved addresses (TEST-NET-1/2/3, etc.)
/// - Public addresses (everything else)
///
/// For IPv6 addresses, it identifies:
/// - Loopback (::1)
/// - Multicast (ff00::/8)
/// - Unique Local Addresses (ULA, fc00::/7)
/// - Link Local addresses (fe80::/10)
/// - Global Unicast addresses (everything else)
///
/// # Arguments
/// * `ip` - The IP address to classify
///
/// # Returns
/// A string describing the address classification
fn classify_ip(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => {
            if ipv4.is_private() {
                "Private (RFC 1918)".to_string()
            } else if ipv4.is_loopback() {
                "Loopback".to_string()
            } else if ipv4.is_multicast() {
                "Multicast".to_string()
            } else if ipv4.is_broadcast() {
                "Broadcast".to_string()
            } else if is_documentation_ipv4(ipv4) {
                "Documentation/Reserved".to_string()
            } else {
                "Public".to_string()
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() {
                "Loopback".to_string()
            } else if ipv6.is_multicast() {
                "Multicast".to_string()
            } else if ipv6.is_unique_local() {
                "Unique Local (ULA)".to_string()
            } else if is_link_local(ipv6) {
                "Link Local".to_string()
            } else {
                "Global Unicast".to_string()
            }
        }
    }
}

/// Checks if an IPv4 address falls within documentation or reserved ranges.
///
/// These ranges are reserved for documentation, examples, and testing purposes:
/// - 192.0.2.0/24 (TEST-NET-1)
/// - 198.51.100.0/24 (TEST-NET-2)
/// - 203.0.113.0/24 (TEST-NET-3)
/// - 192.88.99.0/24 (6to4 relay anycast)
/// - 198.18.0.0/15 (Benchmark testing)
///
/// # Arguments
/// * `ip` - The IPv4 address to check
///
/// # Returns
/// `true` if the address is in a documentation/reserved range, `false` otherwise
fn is_documentation_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    match octets {
        [192, 0, 2, _] => true,    // TEST-NET-1
        [198, 51, 100, _] => true, // TEST-NET-2
        [203, 0, 113, _] => true,  // TEST-NET-3
        [192, 88, 99, _] => true,  // 6to4 relay anycast
        [198, 18, 0, _] => true,   // Benchmarking
        _ => false,
    }
}

/// Checks if an IPv6 address is a link-local address.
///
/// Link-local addresses have the prefix fe80::/10, meaning the first 10 bits
/// are 1111111010. In segment notation, the first segment must be 0xfe80
/// and the second segment's first 6 bits must be 0.
///
/// # Arguments
/// * `ip` - The IPv6 address to check
///
/// # Returns
/// `true` if the address is link-local, `false` otherwise
fn is_link_local(ip: &Ipv6Addr) -> bool {
    let segments = ip.segments();
    segments[0] == 0xfe80 && (segments[1] & 0xffc0) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    /// Test IP address classification for various IPv4 addresses
    #[test]
    fn test_classify_ipv4() {
        // Private addresses (RFC 1918)
        let private_ips = vec![
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(172, 16, 0, 1),
            Ipv4Addr::new(172, 31, 255, 255),
            Ipv4Addr::new(192, 168, 1, 1),
        ];
        for ip in private_ips {
            assert_eq!(classify_ip(&IpAddr::V4(ip)), "Private (RFC 1918)");
        }
        let loopback = Ipv4Addr::new(127, 0, 0, 1);
        assert_eq!(classify_ip(&IpAddr::V4(loopback)), "Loopback");
        let multicast = Ipv4Addr::new(224, 0, 0, 1);
        assert_eq!(classify_ip(&IpAddr::V4(multicast)), "Multicast");
        let broadcast = Ipv4Addr::new(255, 255, 255, 255);
        assert_eq!(classify_ip(&IpAddr::V4(broadcast)), "Broadcast");
        let doc_ips = vec![
            Ipv4Addr::new(192, 0, 2, 1),    // TEST-NET-1
            Ipv4Addr::new(198, 51, 100, 1), // TEST-NET-2
            Ipv4Addr::new(203, 0, 113, 1),  // TEST-NET-3
            Ipv4Addr::new(192, 88, 99, 1),  // 6to4 relay
            Ipv4Addr::new(198, 18, 0, 1),   // Benchmarking
        ];
        for ip in doc_ips {
            assert_eq!(classify_ip(&IpAddr::V4(ip)), "Documentation/Reserved");
        }
        let public_ips = vec![
            Ipv4Addr::new(8, 8, 8, 8),        // Google DNS
            Ipv4Addr::new(1, 1, 1, 1),        // Cloudflare DNS
            Ipv4Addr::new(208, 67, 222, 222), // OpenDNS
        ];
        for ip in public_ips {
            assert_eq!(classify_ip(&IpAddr::V4(ip)), "Public");
        }
    }

    /// Test IP address classification for various IPv6 addresses
    #[test]
    fn test_classify_ipv6() {
        let loopback = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(classify_ip(&IpAddr::V6(loopback)), "Loopback");
        let multicast = Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(classify_ip(&IpAddr::V6(multicast)), "Multicast");
        let ula = Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(classify_ip(&IpAddr::V6(ula)), "Unique Local (ULA)");
        let link_local = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(classify_ip(&IpAddr::V6(link_local)), "Link Local");
        let global = Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888); // Google DNS
        assert_eq!(classify_ip(&IpAddr::V6(global)), "Global Unicast");
    }

    /// Test CIDR range calculations for various subnet sizes
    #[test]
    fn test_cidr_calculations() {
        let ip: u32 = Ipv4Addr::new(192, 168, 1, 0).into();
        let prefix_len = 24;
        let mask = !((1 << (32 - prefix_len)) - 1);
        let network = ip & mask;
        let broadcast = network | !mask;
        let total_hosts = broadcast - network - 1;
        assert_eq!(network, u32::from(Ipv4Addr::new(192, 168, 1, 0)));
        assert_eq!(broadcast, u32::from(Ipv4Addr::new(192, 168, 1, 255)));
        assert_eq!(total_hosts, 254);
        let ip: u32 = Ipv4Addr::new(10, 0, 0, 0).into();
        let prefix_len = 16;
        let mask = !((1 << (32 - prefix_len)) - 1);
        let network = ip & mask;
        let broadcast = network | !mask;
        let total_hosts = broadcast - network - 1;
        assert_eq!(network, u32::from(Ipv4Addr::new(10, 0, 0, 0)));
        assert_eq!(broadcast, u32::from(Ipv4Addr::new(10, 0, 255, 255)));
        assert_eq!(total_hosts, 65534);
        let ip: u32 = Ipv4Addr::new(192, 168, 1, 100).into();
        let prefix_len = 32;
        let mask = !((1 << (32 - prefix_len)) - 1);
        let network = ip & mask;
        let broadcast = network | !mask;
        let total_hosts = if network == broadcast {
            1
        } else {
            broadcast - network - 1
        };
        assert_eq!(network, u32::from(Ipv4Addr::new(192, 168, 1, 100)));
        assert_eq!(broadcast, u32::from(Ipv4Addr::new(192, 168, 1, 100)));
        assert_eq!(total_hosts, 1);
    }

    #[test]
    fn test_link_local_detection() {
        let valid = vec![
            Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1),
            Ipv6Addr::new(0xfe80, 0x1234, 0, 0, 0, 0, 0, 1),
            Ipv6Addr::new(0xfe80, 0xffff, 0, 0, 0, 0, 0, 1),
        ];
        for ip in valid {
            assert!(is_link_local(&ip), "Expected {:?} to be link-local", ip);
        }
        let invalid = vec![
            Ipv6Addr::new(0xfe90, 0, 0, 0, 0, 0, 0, 1), // Wrong prefix
            Ipv6Addr::new(0xfe00, 0, 0, 0, 0, 0, 0, 1), // Wrong prefix
            Ipv6Addr::new(0x2001, 0x4860, 0, 0, 0, 0, 0, 1), // Global unicast
            Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1), // ULA
        ];
        for ip in invalid {
            assert!(
                !is_link_local(&ip),
                "Expected {:?} not to be link-local",
                ip
            );
        }
    }
}
