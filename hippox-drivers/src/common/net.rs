//! Network common utilities
//!
//! This module provides network-related utilities including port parsing,
//! service detection, DNS lookup, and TCP connection functions.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info, warn};
/// Parse port specification string into vector of ports
/// Supports: "80", "1-1024", "22,80,443", "22,80-90,443"
pub fn parse_ports(ports_spec: &str) -> DriverResult<Vec<u16>> {
    debug!("Parsing ports specification: {}", ports_spec);
    let mut ports = Vec::new();
    for part in ports_spec.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                let start = match range[0].parse::<u16>() {
                    Ok(s) => s,
                    Err(e) => {
                        let err_msg = format!("Invalid port range start: {}", e);
                        warn!("{}", err_msg);
                        return Err(DriverError::validation("ports_spec", err_msg));
                    }
                };
                let end = match range[1].parse::<u16>() {
                    Ok(e) => e,
                    Err(e) => {
                        let err_msg = format!("Invalid port range end: {}", e);
                        warn!("{}", err_msg);
                        return Err(DriverError::validation("ports_spec", err_msg));
                    }
                };
                for port in start..=end {
                    ports.push(port);
                }
            }
        } else if !part.is_empty() {
            let port = match part.parse::<u16>() {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Invalid port: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::validation("ports_spec", err_msg));
                }
            };
            ports.push(port);
        }
    }
    ports.sort();
    ports.dedup();
    info!("Parsed {} ports", ports.len());
    return Ok(ports);
}
/// Get service name for a port using range matching
pub fn get_service_name(port: u16) -> &'static str {
    match port {
        20 | 21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 | 465 | 587 => "SMTP",
        53 => "DNS",
        67 | 68 => "DHCP",
        69 => "TFTP",
        80 | 8000 | 8080 | 8081 | 8888 => "HTTP",
        110 | 995 => "POP3",
        111 | 135 => "RPC",
        123 => "NTP",
        137 | 138 | 139 => "NetBIOS",
        143 | 993 => "IMAP",
        161 | 162 => "SNMP",
        179 => "BGP",
        389 | 636 => "LDAP",
        443 | 8443 | 9443 => "HTTPS",
        445 => "SMB",
        514 => "Syslog",
        873 => "rsync",
        990 => "FTPS",
        1080 => "SOCKS",
        1433 => "MSSQL",
        1521 => "Oracle",
        1723 => "PPTP",
        1883 | 8883 => "MQTT",
        2049 => "NFS",
        2082 | 2083 => "cPanel",
        2222 => "SSH",
        2375 | 2376 => "Docker",
        2379 | 2380 => "etcd",
        2480 => "OrientDB",
        3000 => "Grafana",
        3306 => "MySQL",
        3389 => "RDP",
        4000 => "Zabbix",
        5000 | 5001 | 5002 => "Flask/Django",
        5432 => "PostgreSQL",
        5672 => "RabbitMQ",
        5900 | 5901 => "VNC",
        5984 => "CouchDB",
        5985 | 5986 => "WinRM",
        6379 => "Redis",
        7000 | 7001 | 7199 | 9042 => "Cassandra",
        8086 => "InfluxDB",
        8091..=8099 => "Couchbase",
        8111 => "Artifactory",
        8125 => "StatsD",
        8126 => "Datadog",
        8140 => "Puppet",
        8161 => "ActiveMQ",
        8200 => "Vault",
        8333 | 8334 => "Bitcoin",
        8384 => "Syncthing",
        8500 | 8600 => "Consul",
        8761 => "Eureka",
        8983 => "Solr",
        9000 => "Portainer",
        9090 | 9091 => "Prometheus",
        9092..=9099 => "Kafka",
        9100.. => "NodeExporter",
        9200..=9299 => "Elasticsearch",
        9300..=9399 => "Elasticsearch",
        9411 => "Zipkin",
        _ => "Unknown",
    }
}
/// Get probe string for a port
pub fn get_probe_for_port(port: u16) -> Option<&'static [u8]> {
    match port {
        21 => Some(b"QUIT\r\n"),
        22 => Some(b"SSH-2.0-Client\r\n"),
        25 => Some(b"EHLO example.com\r\n"),
        80 | 8080 | 8000 => Some(b"HEAD / HTTP/1.0\r\n\r\n"),
        110 => Some(b"QUIT\r\n"),
        143 => Some(b"A001 CAPABILITY\r\n"),
        443 | 8443 => Some(b"HEAD / HTTP/1.0\r\n\r\n"),
        3306 => Some(b"\x00\x00\x00\x01"),
        5432 => Some(b"\x00\x00\x00\x08\x04\xd2\x16\x2f"),
        6379 => Some(b"PING\r\n"),
        _ => None,
    }
}
/// Identify service from banner
pub fn identify_service(port: u16, banner: &str) -> (String, Option<String>, u8) {
    let banner_lower = banner.to_lowercase();
    let service = get_service_name(port);
    if service == "Unknown" {
        return ("Unknown".to_string(), None, 0);
    }
    // Extract version from banner
    let version = extract_version_from_banner(&banner_lower);
    let confidence = if version.is_some() { 90 } else { 70 };
    return (service.to_string(), version, confidence);
}
fn extract_version_from_banner(banner: &str) -> Option<String> {
    let patterns = [
        (r"nginx/([\d\.]+)", "nginx"),
        (r"Apache/([\d\.]+)", "apache"),
        (r"openssh[_\-]?([\d\.]+)", "openssh"),
        (r"OpenSSH[_\-]?([\d\.]+)", "openssh"),
        (r"vsftpd/([\d\.]+)", "vsftpd"),
        (r"proftpd[_\-]?([\d\.]+)", "proftpd"),
        (r"postfix/([\d\.]+)", "postfix"),
        (r"sendmail[_\-]?([\d\.]+)", "sendmail"),
        (r"exim[_\-]?([\d\.]+)", "exim"),
        (r"dovecot[_\-]?([\d\.]+)", "dovecot"),
        (r"MySQL[_\-]?([\d\.]+)", "mysql"),
        (r"MariaDB[_\-]?([\d\.]+)", "mariadb"),
        (r"PostgreSQL[_\-]?([\d\.]+)", "postgresql"),
        (r"Redis[_\-]?([\d\.]+)", "redis"),
        (r"Lighttpd/([\d\.]+)", "lighttpd"),
        (r"IIS/([\d\.]+)", "iis"),
        (r"Dropbear[_\-]?([\d\.]+)", "dropbear"),
        (r"OpenSSL/([\d\.]+)", "openssl"),
    ];
    for (pattern, _) in &patterns {
        let re = regex::Regex::new(pattern).ok()?;
        if let Some(cap) = re.captures(banner) {
            if let Some(m) = cap.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }
    return None;
}
/// Resolve hostname to IP address
pub fn resolve_host(host: &str) -> DriverResult<std::net::IpAddr> {
    debug!("Resolving host: {}", host);
    let addr = format!("{}:0", host);
    let mut addrs = match addr.to_socket_addrs() {
        Ok(a) => a,
        Err(e) => {
            let err_msg = format!("Failed to resolve host {}: {}", host, e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    match addrs.next() {
        Some(s) => {
            info!("Resolved host {} to {}", host, s.ip());
            return Ok(s.ip());
        }
        None => {
            let err_msg = format!("Failed to resolve host: {}", host);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// TCP connect with timeout
pub async fn tcp_connect(ip: std::net::IpAddr, port: u16, timeout_secs: u64) -> DriverResult<TcpStream> {
    debug!("Connecting to {}:{} with timeout {}s", ip, port, timeout_secs);
    let addr = std::net::SocketAddr::new(ip, port);
    let timeout_dur = Duration::from_secs(timeout_secs);
    let stream = match timeout(timeout_dur, TcpStream::connect(&addr)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            let err_msg = format!("Failed to connect to {}:{}: {}", ip, port, e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
        Err(_) => {
            let err_msg = format!("Connection to {}:{} timed out after {}s", ip, port, timeout_secs);
            warn!("{}", err_msg);
            return Err(DriverError::timeout(Some(timeout_secs.to_string())));
        }
    };
    info!("Connected to {}:{}", ip, port);
    return Ok(stream);
}
/// Get string parameter from HashMap
pub fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    debug!("Getting string parameter: {}", name);
    match params.get(name).and_then(|v| v.as_str()) {
        Some(s) => {
            info!("Parameter {}: {}", name, s);
            return Ok(s.to_string());
        }
        None => {
            let err_msg = format!("Missing parameter: {}", name);
            warn!("{}", err_msg);
            return Err(DriverError::missing_parameter(name));
        }
    }
}
/// Get u64 parameter from HashMap with default
pub fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    let value = params.get(name).and_then(|v| v.as_u64()).unwrap_or(default);
    debug!("Parameter {}: {} (default: {})", name, value, default);
    return value;
}
/// Get bool parameter from HashMap with default
pub fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    let value = params.get(name).and_then(|v| v.as_bool()).unwrap_or(default);
    debug!("Parameter {}: {} (default: {})", name, value, default);
    return value;
}
/// Nslookup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NslookupResult {
    pub domain: String,
    pub dns_server: String,
    pub a_records: Vec<String>,
    pub aaaa_records: Vec<String>,
    pub mx_records: Vec<(String, u16)>,
    pub txt_records: Vec<String>,
    pub cname_records: Vec<String>,
    pub ns_records: Vec<String>,
    pub soa_record: Option<String>,
}
/// Perform detailed DNS lookup (nslookup style)
pub async fn nslookup(domain: &str, dns_server: Option<&str>) -> DriverResult<NslookupResult> {
    use trust_dns_proto::rr::{RData, RecordType};
    use trust_dns_resolver::Resolver;
    use trust_dns_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};
    debug!("Performing nslookup for domain: {}, dns_server: {:?}", domain, dns_server);
    let dns_server = dns_server.unwrap_or("8.8.8.8");
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
    let resolver = match Resolver::new(resolver_config, resolver_opts) {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Failed to create DNS resolver: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let mut a_records = Vec::new();
    let mut aaaa_records = Vec::new();
    let mut mx_records = Vec::new();
    let mut txt_records = Vec::new();
    let mut cname_records = Vec::new();
    let mut ns_records = Vec::new();
    let mut soa_record = None;
    // A records
    if let Ok(response) = resolver.lookup(domain, RecordType::A) {
        for record in response.iter() {
            if let RData::A(ip) = record {
                a_records.push(ip.to_string());
            }
        }
        debug!("Found {} A records", a_records.len());
    }
    // AAAA records
    if let Ok(response) = resolver.lookup(domain, RecordType::AAAA) {
        for record in response.iter() {
            if let RData::AAAA(ip) = record {
                aaaa_records.push(ip.to_string());
            }
        }
        debug!("Found {} AAAA records", aaaa_records.len());
    }
    // MX records
    if let Ok(response) = resolver.lookup(domain, RecordType::MX) {
        for record in response.iter() {
            if let RData::MX(mx) = record {
                mx_records.push((mx.exchange().to_string(), mx.preference()));
            }
        }
        mx_records.sort_by_key(|(_, priority)| *priority);
        debug!("Found {} MX records", mx_records.len());
    }
    // TXT records
    if let Ok(response) = resolver.lookup(domain, RecordType::TXT) {
        for record in response.iter() {
            if let RData::TXT(txt) = record {
                let text: String = txt.txt_data().iter().map(|d| String::from_utf8_lossy(d)).collect::<Vec<_>>().join("");
                txt_records.push(text);
            }
        }
        debug!("Found {} TXT records", txt_records.len());
    }
    // CNAME records
    if let Ok(response) = resolver.lookup(domain, RecordType::CNAME) {
        for record in response.iter() {
            if let RData::CNAME(cname) = record {
                cname_records.push(cname.to_string());
            }
        }
        debug!("Found {} CNAME records", cname_records.len());
    }
    // NS records
    if let Ok(response) = resolver.lookup(domain, RecordType::NS) {
        for record in response.iter() {
            if let RData::NS(ns) = record {
                ns_records.push(ns.to_string());
            }
        }
        debug!("Found {} NS records", ns_records.len());
    }
    // SOA record
    if let Ok(response) = resolver.lookup(domain, RecordType::SOA) {
        for record in response.iter() {
            if let RData::SOA(soa) = record {
                soa_record = Some(format!("{} (serial: {})", soa.mname(), soa.serial()));
                break;
            }
        }
        debug!("SOA record found: {:?}", soa_record);
    }
    let result = NslookupResult {
        domain: domain.to_string(),
        dns_server: dns_server.to_string(),
        a_records,
        aaaa_records,
        mx_records,
        txt_records,
        cname_records,
        ns_records,
        soa_record,
    };
    info!("nslookup completed for domain: {}", domain);
    return Ok(result);
}
/// Get local network connections (netstat style)
pub fn get_network_connections() -> DriverResult<Vec<HashMap<String, String>>> {
    debug!("Getting network connections");
    #[cfg(target_os = "linux")]
    {
        let mut connections = Vec::new();
        let content = match std::fs::read_to_string("/proc/net/tcp") {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to read /proc/net/tcp: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        };
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let mut conn = HashMap::new();
                conn.insert("local_address".to_string(), parts[1].to_string());
                conn.insert("remote_address".to_string(), parts[2].to_string());
                conn.insert("state".to_string(), parts[3].to_string());
                connections.push(conn);
            }
        }
        info!("Found {} network connections", connections.len());
        return Ok(connections);
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = match Command::new("netstat").args(["-n", "-t"]).output() {
            Ok(o) => o,
            Err(e) => {
                let err_msg = format!("Failed to execute netstat: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::execution(err_msg));
            }
        };
        let mut connections = Vec::new();
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let mut conn = HashMap::new();
                    conn.insert("protocol".to_string(), parts[0].to_string());
                    conn.insert("recvq".to_string(), parts[1].to_string());
                    conn.insert("sendq".to_string(), parts[2].to_string());
                    conn.insert("local_address".to_string(), parts[3].to_string());
                    conn.insert("foreign_address".to_string(), parts[4].to_string());
                    if parts.len() > 5 {
                        conn.insert("state".to_string(), parts[5].to_string());
                    }
                    connections.push(conn);
                }
            }
        }
        info!("Found {} network connections", connections.len());
        return Ok(connections);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let err_msg = "netstat not supported on this platform".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::internal(err_msg));
    }
}
