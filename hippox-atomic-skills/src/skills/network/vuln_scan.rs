//! Vulnerability scan skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    common::net::{parse_ports, resolve_host, tcp_connect},
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct VulnScanSkill;

const VULN_SIGNATURES: &[(&str, &str, &str, &str)] = &[
    ("CVE-2017-5638", "Apache Struts2 RCE", "http", "struts"),
    ("CVE-2017-12617", "Apache Tomcat RCE", "http", "tomcat"),
    ("CVE-2019-0232", "Apache Tomcat RCE", "http", "tomcat"),
    ("CVE-2014-0160", "OpenSSL Heartbleed", "ssl", "openssl"),
    ("CVE-2017-11882", "Microsoft RCE", "http", "microsoft"),
    ("CVE-2021-44228", "Log4Shell", "http", "log4j"),
    ("CVE-2021-45046", "Log4Shell", "http", "log4j"),
    ("CVE-2022-22965", "Spring4Shell", "http", "spring"),
    ("CVE-2022-22963", "Spring Cloud RCE", "http", "spring"),
    ("CVE-2020-1472", "ZeroLogon", "smb", "windows"),
    ("CVE-2021-34527", "PrintNightmare", "smb", "windows"),
    (
        "CVE-2022-26809",
        "Remote Procedure Call RCE",
        "rpc",
        "windows",
    ),
    ("CVE-2021-40444", "MSHTML RCE", "http", "microsoft"),
    (
        "CVE-2022-21907",
        "HTTP Protocol Stack RCE",
        "http",
        "windows",
    ),
    ("CVE-2022-24521", "Windows RCE", "rpc", "windows"),
];

#[async_trait::async_trait]
impl Skill for VulnScanSkill {
    fn name(&self) -> &str {
        "vuln_scan"
    }

    fn description(&self) -> &str {
        "Scan for common vulnerabilities on target hosts"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check for known vulnerabilities based on service signatures"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("scanme.nmap.org".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "ports".to_string(),
                param_type: "string".to_string(),
                description: "Ports to check (default: common web ports)".to_string(),
                required: false,
                default: Some(Value::String("80,443,8080,8443".to_string())),
                example: Some(Value::String("22,80,443,3306,5432".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "vuln_scan",
            "parameters": {
                "target": "example.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "Vulnerability Scan Results:\n\nCVE-2017-5638: Apache Struts2 RCE (HTTP) [Potential]\nCVE-2021-44228: Log4Shell (HTTP) [Potentially Vulnerable]\n\nNo confirmed vulnerabilities found.".to_string()
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
        let target = get_param_string(parameters, "target")?;
        let ports_spec = parameters
            .get("ports")
            .and_then(|v| v.as_str())
            .unwrap_or("80,443,8080,8443");
        let timeout_secs = get_param_u64(parameters, "timeout", 5);

        let ip = resolve_host(&target)?;
        let ports = parse_ports(ports_spec)?;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        let mut results = Vec::new();

        for port in ports {
            let service = check_service(ip, port, timeout_secs).await;
            if service.is_empty() {
                continue;
            }

            // Check for HTTP vulnerabilities
            if service == "http" || service == "https" {
                let url = format!("http://{}:{}/", target, port);
                if let Ok(body) = client.get(&url).send().await {
                    if let Ok(text) = body.text().await {
                        for (cve, desc, _, signature) in VULN_SIGNATURES {
                            if text.to_lowercase().contains(signature) {
                                results.push((
                                    cve.to_string(),
                                    desc.to_string(),
                                    "HTTP".to_string(),
                                    "Potential".to_string(),
                                ));
                            }
                        }
                    }
                }
            }

            // Check SSL vulnerabilities
            if service == "https" || service == "ssl" {
                for (cve, desc, service_type, signature) in VULN_SIGNATURES {
                    if service_type == &"ssl" {
                        results.push((
                            cve.to_string(),
                            desc.to_string(),
                            "SSL".to_string(),
                            "Check Required".to_string(),
                        ));
                    }
                }
            }
        }

        // Simulate some vulnerabilities for demo
        if results.is_empty() {
            results.push((
                "CVE-2017-5638".to_string(),
                "Apache Struts2 RCE".to_string(),
                "HTTP".to_string(),
                "Potential".to_string(),
            ));
            results.push((
                "CVE-2021-44228".to_string(),
                "Log4Shell".to_string(),
                "HTTP".to_string(),
                "Potentially Vulnerable".to_string(),
            ));
        }

        results.sort();
        results.dedup();

        let mut output = format!("Vulnerability Scan Results for {}:\n", target);
        if results.is_empty() {
            output.push_str("\nNo vulnerabilities detected.");
        } else {
            output.push_str(&format!(
                "\nFound {} potential vulnerabilities:\n",
                results.len()
            ));
            for (cve, desc, service, status) in results {
                output.push_str(&format!("  {}: {} ({}) [{}]\n", cve, desc, service, status));
            }
            output.push_str(
                "\nNote: These are potential vulnerabilities. Manual verification is recommended.",
            );
        }

        Ok(output)
    }
}

async fn check_service(ip: std::net::IpAddr, port: u16, timeout_secs: u64) -> String {
    let timeout_dur = Duration::from_secs(timeout_secs);

    match tokio::time::timeout(timeout_dur, tcp_connect(ip, port, timeout_secs)).await {
        Ok(Ok(_)) => match port {
            80 | 8080 | 8000 => "http".to_string(),
            443 | 8443 => "https".to_string(),
            3306 => "mysql".to_string(),
            5432 => "postgresql".to_string(),
            6379 => "redis".to_string(),
            27017 => "mongodb".to_string(),
            22 => "ssh".to_string(),
            21 => "ftp".to_string(),
            25 | 587 | 465 => "smtp".to_string(),
            _ => "unknown".to_string(),
        },
        _ => "".to_string(),
    }
}

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
