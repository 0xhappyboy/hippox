use crate::common::net::{nslookup, NslookupResult};
use crate::types::{Skill, SkillParameter};
use crate::{SkillCallback, SkillCategory, SkillContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct NslookupSkill;

#[async_trait::async_trait]
impl Skill for NslookupSkill {
    fn name(&self) -> &str {
        "nslookup"
    }

    fn description(&self) -> &str {
        "Perform detailed DNS lookup with all record types (A, AAAA, MX, TXT, CNAME, NS, SOA)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need comprehensive DNS information beyond basic A records"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "domain".to_string(),
                param_type: "string".to_string(),
                description: "Domain name to query".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("google.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "dns_server".to_string(),
                param_type: "string".to_string(),
                description: "DNS server to use (default: 8.8.8.8)".to_string(),
                required: false,
                default: Some(Value::String("8.8.8.8".to_string())),
                example: Some(Value::String("1.1.1.1".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "nslookup",
            "parameters": {
                "domain": "google.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "NSLookup for google.com (DNS: 8.8.8.8):\nA: 142.250.185.46\nMX: smtp.google.com (priority 10)\nNS: ns1.google.com".to_string()
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
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(task_id.clone(), skill_index, Some("Starting NSLookup".to_string()));
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        let domain = parameters
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'domain' parameter"))?;

        let dns_server = parameters
            .get("dns_server")
            .and_then(|v| v.as_str());

        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some(format!("Domain: {}", domain)));
            if let Some(dns) = dns_server {
                cb.on_log(task_id.clone(), skill_index, Some(format!("DNS Server: {}", dns)));
            }
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }

        let result = nslookup(domain, dns_server).await?;

        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some("Lookup completed".to_string()));
            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
        }

        let mut output = format!("NSLookup for {} (DNS: {}):\n", result.domain, result.dns_server);

        if !result.a_records.is_empty() {
            output.push_str(&format!("A: {}\n", result.a_records.join(", ")));
        }
        if !result.aaaa_records.is_empty() {
            output.push_str(&format!("AAAA: {}\n", result.aaaa_records.join(", ")));
        }
        if !result.mx_records.is_empty() {
            let mx_str: Vec<String> = result.mx_records
                .iter()
                .map(|(server, priority)| format!("{} (priority {})", server, priority))
                .collect();
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
        }

        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), skill_index, Some("Result formatted".to_string()));
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(task_id.clone(), skill_index, Some("nslookup".to_string()), Some(output.clone()));
        }

        Ok(output)
    }
}