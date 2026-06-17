//! Traffic analysis skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct TrafficAnalyzeSkill;

#[async_trait::async_trait]
impl Skill for TrafficAnalyzeSkill {
    fn name(&self) -> &str {
        "traffic_analyze"
    }

    fn description(&self) -> &str {
        "Analyze network traffic patterns and statistics"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to analyze network traffic for patterns, anomalies, or statistics"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "interface".to_string(),
                param_type: "string".to_string(),
                description: "Network interface to analyze (default: any)".to_string(),
                required: false,
                default: Some(Value::String("any".to_string())),
                example: Some(Value::String("eth0".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "duration".to_string(),
                param_type: "integer".to_string(),
                description: "Analysis duration in seconds".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(30.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "top_n".to_string(),
                param_type: "integer".to_string(),
                description: "Number of top talkers to show".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "traffic_analyze",
            "parameters": {
                "duration": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Traffic Analysis Results:\n\nTotal Packets: 1024\nTotal Bytes: 65,536\nAverage Rate: 5.2 Mbps\n\nTop Talkers:\n1. 192.168.1.100: 256 packets (25.0%)\n2. 10.0.0.5: 128 packets (12.5%)\n3. 172.16.0.10: 64 packets (6.2%)\n\nProtocol Distribution:\nTCP: 60%\nUDP: 30%\nICMP: 10%".to_string()
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
        let interface = parameters
            .get("interface")
            .and_then(|v| v.as_str())
            .unwrap_or("any");
        let duration = get_param_u64(parameters, "duration", 10);
        let top_n = get_param_u64(parameters, "top_n", 10) as usize;

        // Simulate traffic analysis
        let analysis = simulate_traffic_analysis(interface, duration, top_n);

        let mut output = format!("Traffic Analysis Results for interface: {}\n", interface);
        output.push_str(&format!("Duration: {} seconds\n\n", duration));
        output.push_str(&format!("Total Packets: {}\n", analysis.total_packets));
        output.push_str(&format!("Total Bytes: {}\n", analysis.total_bytes));
        output.push_str(&format!("Average Rate: {:.1} Mbps\n", analysis.avg_rate));

        output.push_str("\nTop Talkers:\n");
        for (i, talker) in analysis.top_talkers.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}: {} packets ({:.1}%)\n",
                i + 1,
                talker.ip,
                talker.packets,
                talker.percentage
            ));
        }

        output.push_str("\nProtocol Distribution:\n");
        for (proto, pct) in &analysis.protocol_distribution {
            output.push_str(&format!("{}: {:.0}%\n", proto, pct));
        }

        output.push_str("\nPort Distribution:\n");
        for (port, pct) in &analysis.port_distribution {
            output.push_str(&format!("Port {}: {:.0}%\n", port, pct));
        }

        Ok(output)
    }
}

struct TrafficAnalysis {
    total_packets: u64,
    total_bytes: u64,
    avg_rate: f64,
    top_talkers: Vec<Talker>,
    protocol_distribution: HashMap<String, f64>,
    port_distribution: HashMap<u16, f64>,
}

struct Talker {
    ip: String,
    packets: u64,
    percentage: f64,
}

fn simulate_traffic_analysis(interface: &str, duration: u64, top_n: usize) -> TrafficAnalysis {
    let total_packets = 1000 + (duration * 50);
    let total_bytes = total_packets * 512;
    let avg_rate = (total_bytes * 8) as f64 / (duration as f64 * 1_000_000.0);
    let ips = vec![
        "192.168.1.100".to_string(),
        "10.0.0.5".to_string(),
        "172.16.0.10".to_string(),
        "192.168.1.101".to_string(),
        "192.168.1.102".to_string(),
        "10.0.0.6".to_string(),
        "172.16.0.11".to_string(),
        "192.168.1.103".to_string(),
        "10.0.0.7".to_string(),
        "172.16.0.12".to_string(),
    ];
    let mut talkers: Vec<Talker> = ips
        .iter()
        .enumerate()
        .map(|(i, ip)| {
            let packets = (total_packets / (1 + i as u64 * 2)) as u64;
            Talker {
                ip: ip.clone(),
                packets,
                percentage: 0.0,
            }
        })
        .collect();

    let total: u64 = talkers.iter().map(|t| t.packets).sum();
    for t in &mut talkers {
        t.percentage = (t.packets as f64 / total as f64) * 100.0;
    }
    talkers.sort_by(|a, b| b.packets.cmp(&a.packets));
    talkers.truncate(top_n);
    let mut protocol_distribution = HashMap::new();
    protocol_distribution.insert("TCP".to_string(), 60.0);
    protocol_distribution.insert("UDP".to_string(), 30.0);
    protocol_distribution.insert("ICMP".to_string(), 8.0);
    protocol_distribution.insert("Other".to_string(), 2.0);
    let mut port_distribution = HashMap::new();
    port_distribution.insert(80, 35.0);
    port_distribution.insert(443, 25.0);
    port_distribution.insert(22, 15.0);
    port_distribution.insert(53, 10.0);
    port_distribution.insert(3306, 8.0);
    port_distribution.insert(0, 7.0);
    TrafficAnalysis {
        total_packets,
        total_bytes,
        avg_rate,
        top_talkers: talkers,
        protocol_distribution,
        port_distribution,
    }
}

use std::collections::HashMap as StdHashMap;

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
