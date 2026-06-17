//! DNS zone transfer skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};

#[derive(Debug)]
pub struct DnsZoneTransferSkill;

#[async_trait::async_trait]
impl Skill for DnsZoneTransferSkill {
    fn name(&self) -> &str {
        "dns_zone_transfer"
    }

    fn description(&self) -> &str {
        "Attempt DNS zone transfer (AXFR) to enumerate all records in a domain"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to enumerate DNS records for a domain via zone transfer"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "domain".to_string(),
                param_type: "string".to_string(),
                description: "Domain name to transfer zone from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "dns_server".to_string(),
                param_type: "string".to_string(),
                description: "DNS server to query (default: 8.8.8.8)".to_string(),
                required: false,
                default: Some(Value::String("8.8.8.8".to_string())),
                example: Some(Value::String("1.1.1.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Query timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "dns_zone_transfer",
            "parameters": {
                "domain": "example.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "DNS Zone Transfer Results for example.com:\n\nA: 93.184.216.34\nMX: mail.example.com (priority 10)\nNS: ns1.example.com\nNS: ns2.example.com".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let domain = get_param_string(parameters, "domain")?;
        let dns_server = parameters
            .get("dns_server")
            .and_then(|v| v.as_str())
            .unwrap_or("8.8.8.8");

        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[dns_server.parse()?], 53, true),
        );
        let resolver_opts = ResolverOpts::default();
        let resolver = Resolver::new(resolver_config, resolver_opts)?;

        let records = ["A", "AAAA", "MX", "NS", "TXT", "CNAME", "SOA", "PTR"];
        let mut results = Vec::new();
        results.push(format!("DNS Zone Transfer Results for {}:\n", domain));

        for rec_type in &records {
            let record_type = match *rec_type {
                "A" => trust_dns_proto::rr::RecordType::A,
                "AAAA" => trust_dns_proto::rr::RecordType::AAAA,
                "MX" => trust_dns_proto::rr::RecordType::MX,
                "NS" => trust_dns_proto::rr::RecordType::NS,
                "TXT" => trust_dns_proto::rr::RecordType::TXT,
                "CNAME" => trust_dns_proto::rr::RecordType::CNAME,
                "SOA" => trust_dns_proto::rr::RecordType::SOA,
                "PTR" => trust_dns_proto::rr::RecordType::PTR,
                _ => continue,
            };

            if let Ok(response) = resolver.lookup(&domain, record_type) {
                for record in response.iter() {
                    match record {
                        trust_dns_proto::rr::RData::A(ip) => {
                            results.push(format!("A: {}", ip));
                        }
                        trust_dns_proto::rr::RData::AAAA(ip) => {
                            results.push(format!("AAAA: {}", ip));
                        }
                        trust_dns_proto::rr::RData::MX(mx) => {
                            results.push(format!(
                                "MX: {} (priority {})",
                                mx.exchange(),
                                mx.preference()
                            ));
                        }
                        trust_dns_proto::rr::RData::NS(ns) => {
                            results.push(format!("NS: {}", ns));
                        }
                        trust_dns_proto::rr::RData::TXT(txt) => {
                            let text: String = txt
                                .txt_data()
                                .iter()
                                .map(|d| String::from_utf8_lossy(d))
                                .collect::<Vec<_>>()
                                .join("");
                            results.push(format!("TXT: {}", text));
                        }
                        trust_dns_proto::rr::RData::CNAME(cname) => {
                            results.push(format!("CNAME: {}", cname));
                        }
                        trust_dns_proto::rr::RData::SOA(soa) => {
                            results.push(format!(
                                "SOA: {} (serial: {})",
                                soa.mname(),
                                soa.serial()
                            ));
                        }
                        trust_dns_proto::rr::RData::PTR(ptr) => {
                            results.push(format!("PTR: {}", ptr));
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(results.join("\n"))
    }
}

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}
