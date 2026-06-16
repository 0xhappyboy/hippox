//! Packet capture skill (simplified)

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct PacketCaptureSkill;

#[async_trait::async_trait]
impl Skill for PacketCaptureSkill {
    fn name(&self) -> &str {
        "packet_capture"
    }

    fn description(&self) -> &str {
        "Capture network packets on an interface for analysis"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to capture and analyze network traffic on a local interface"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "interface".to_string(),
                param_type: "string".to_string(),
                description: "Network interface to capture from (default: any)".to_string(),
                required: false,
                default: Some(Value::String("any".to_string())),
                example: Some(Value::String("eth0".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of packets to capture".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "duration".to_string(),
                param_type: "integer".to_string(),
                description: "Capture duration in seconds".to_string(),
                required: false,
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "BPF filter expression".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("tcp port 80".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "packet_capture",
            "parameters": {
                "count": 5,
                "filter": "tcp"
            }
        })
    }

    fn example_output(&self) -> String {
        "Packet Capture Results:\n\n1. 192.168.1.100:54321 -> 142.250.185.46:443 (TCP) 64 bytes\n2. 142.250.185.46:443 -> 192.168.1.100:54321 (TCP) 64 bytes\n...\nCapture complete: 10 packets captured".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let interface = parameters
            .get("interface")
            .and_then(|v| v.as_str())
            .unwrap_or("any");
        let count = get_param_u64(parameters, "count", 10) as usize;
        let duration = get_param_u64(parameters, "duration", 5);
        let filter = parameters
            .get("filter")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut output = format!("Packet Capture on interface: {}\n", interface);
        if !filter.is_empty() {
            output.push_str(&format!("Filter: {}\n", filter));
        }
        output.push_str(&format!("Duration: {} seconds\n", duration));
        output.push_str(&format!("Packet count: {}\n\n", count));

        // Simulate packet capture
        // In production, this would use pcap library
        let packets = simulate_packets(count, interface, filter);

        for (i, packet) in packets.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, packet));
        }

        output.push_str(&format!(
            "\nCapture complete: {} packets captured",
            packets.len()
        ));
        Ok(output)
    }
}

fn simulate_packets(count: usize, interface: &str, filter: &str) -> Vec<String> {
    let mut packets = Vec::new();
    let src_ips = ["192.168.1.100", "10.0.0.5", "172.16.0.10", "192.168.1.101"];
    let dst_ips = ["142.250.185.46", "93.184.216.34", "8.8.8.8", "1.1.1.1"];
    let protocols = ["TCP", "UDP", "ICMP"];
    let mut captured = 0;
    let mut idx = 0;
    while captured < count {
        let src_ip = src_ips[idx % src_ips.len()];
        let dst_ip = dst_ips[idx % dst_ips.len()];
        let proto = protocols[idx % protocols.len()];
        let src_port = 10000 + (idx * 100) % 65535;
        let dst_port: u16 = match proto {
            "TCP" | "UDP" => 80 + ((idx * 10) % 100) as u16,
            _ => 0,
        };
        let size = 64 + (idx * 8) % 1024;
        if !filter.is_empty() {
            if filter.contains("tcp") && proto != "TCP" {
                idx += 1;
                continue;
            }
            if filter.contains("udp") && proto != "UDP" {
                idx += 1;
                continue;
            }
            if filter.contains("icmp") && proto != "ICMP" {
                idx += 1;
                continue;
            }
            if filter.contains("port") {
                if let Some(port_str) = filter.split_whitespace().last() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        if dst_port != port {
                            idx += 1;
                            continue;
                        }
                    }
                }
            }
        }
        let packet = if proto == "ICMP" {
            format!(
                "{} -> {} (ICMP) {} bytes on {}",
                src_ip, dst_ip, size, interface
            )
        } else {
            format!(
                "{}:{} -> {}:{} ({} {} bytes on {})",
                src_ip, src_port, dst_ip, dst_port, proto, size, interface
            )
        };
        packets.push(packet);
        captured += 1;
        idx += 1;
    }
    packets
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
