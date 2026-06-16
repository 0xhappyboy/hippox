//! IP block/allowlist management skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct IpBlockSkill;

struct IpBlocklist {
    blocked: HashSet<IpAddr>,
    allowed: HashSet<IpAddr>,
}

impl IpBlocklist {
    fn new() -> Self {
        Self {
            blocked: HashSet::new(),
            allowed: HashSet::new(),
        }
    }

    fn add_blocked(&mut self, ip: IpAddr) {
        self.blocked.insert(ip);
        self.allowed.remove(&ip);
    }

    fn add_allowed(&mut self, ip: IpAddr) {
        self.allowed.insert(ip);
        self.blocked.remove(&ip);
    }

    fn remove_blocked(&mut self, ip: &IpAddr) {
        self.blocked.remove(ip);
    }

    fn remove_allowed(&mut self, ip: &IpAddr) {
        self.allowed.remove(ip);
    }

    fn is_blocked(&self, ip: &IpAddr) -> bool {
        self.blocked.contains(ip)
    }

    fn is_allowed(&self, ip: &IpAddr) -> bool {
        self.allowed.contains(ip)
    }

    fn to_string(&self) -> String {
        let mut parts = Vec::new();
        if !self.blocked.is_empty() {
            parts.push(format!("Blocked: {}", self.blocked.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")));
        }
        if !self.allowed.is_empty() {
            parts.push(format!("Allowed: {}", self.allowed.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")));
        }
        parts.join("\n")
    }
}

#[async_trait::async_trait]
impl Skill for IpBlockSkill {
    fn name(&self) -> &str {
        "ip_block"
    }

    fn description(&self) -> &str {
        "Manage IP blocklist and allowlist for network access control"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to add, remove, or check IP addresses in blocklist/allowlist"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action: add_block, remove_block, add_allow, remove_allow, check, list".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("add_block".to_string())),
                enum_values: Some(vec![
                    "add_block".to_string(),
                    "remove_block".to_string(),
                    "add_allow".to_string(),
                    "remove_allow".to_string(),
                    "check".to_string(),
                    "list".to_string(),
                ]),
            },
            SkillParameter {
                name: "ip".to_string(),
                param_type: "string".to_string(),
                description: "IP address to manage (required for add/remove/check)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("192.168.1.1".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "ip_block",
            "parameters": {
                "action": "add_block",
                "ip": "192.168.1.100"
            }
        })
    }

    fn example_output(&self) -> String {
        "IP Blocklist/Allowlist:\n\nBlocked: 192.168.1.100\nAllowed: 10.0.0.1".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let action = get_param_string(parameters, "action")?;
        let ip_str = parameters.get("ip").and_then(|v| v.as_str());

        let mut list = IpBlocklist::new();

        // For demo, use static lists
        // In production, this would persist to file/database
        let demo_blocked: Vec<&str> = vec!["192.168.1.100", "10.0.0.100", "172.16.0.100"];
        let demo_allowed: Vec<&str> = vec!["192.168.1.1", "10.0.0.1"];

        for ip in demo_blocked {
            if let Ok(addr) = IpAddr::from_str(ip) {
                list.add_blocked(addr);
            }
        }
        for ip in demo_allowed {
            if let Ok(addr) = IpAddr::from_str(ip) {
                list.add_allowed(addr);
            }
        }

        let mut output = String::new();

        match action.as_str() {
            "add_block" => {
                let ip = ip_str.ok_or_else(|| anyhow::anyhow!("Missing required parameter: ip"))?;
                let addr = IpAddr::from_str(ip)?;
                list.add_blocked(addr);
                output.push_str(&format!("Added {} to blocklist\n", ip));
                output.push_str(&list.to_string());
            }
            "remove_block" => {
                let ip = ip_str.ok_or_else(|| anyhow::anyhow!("Missing required parameter: ip"))?;
                let addr = IpAddr::from_str(ip)?;
                list.remove_blocked(&addr);
                output.push_str(&format!("Removed {} from blocklist\n", ip));
                output.push_str(&list.to_string());
            }
            "add_allow" => {
                let ip = ip_str.ok_or_else(|| anyhow::anyhow!("Missing required parameter: ip"))?;
                let addr = IpAddr::from_str(ip)?;
                list.add_allowed(addr);
                output.push_str(&format!("Added {} to allowlist\n", ip));
                output.push_str(&list.to_string());
            }
            "remove_allow" => {
                let ip = ip_str.ok_or_else(|| anyhow::anyhow!("Missing required parameter: ip"))?;
                let addr = IpAddr::from_str(ip)?;
                list.remove_allowed(&addr);
                output.push_str(&format!("Removed {} from allowlist\n", ip));
                output.push_str(&list.to_string());
            }
            "check" => {
                let ip = ip_str.ok_or_else(|| anyhow::anyhow!("Missing required parameter: ip"))?;
                let addr = IpAddr::from_str(ip)?;
                if list.is_blocked(&addr) {
                    output.push_str(&format!("{} is BLOCKED\n", ip));
                } else if list.is_allowed(&addr) {
                    output.push_str(&format!("{} is ALLOWED\n", ip));
                } else {
                    output.push_str(&format!("{} is NOT in any list (default: allow)\n", ip));
                }
            }
            "list" => {
                output.push_str("IP Blocklist/Allowlist:\n\n");
                output.push_str(&list.to_string());
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid action: {}", action));
            }
        }

        Ok(output)
    }
}

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}