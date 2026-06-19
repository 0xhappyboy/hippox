//! WiFi DNS set skill - set custom DNS servers

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WifiDnsSetDriver;

#[async_trait::async_trait]
impl Driver for WifiDnsSetDriver {
    fn name(&self) -> &str {
        "wifi_dns_set"
    }

    fn description(&self) -> &str {
        "Set custom DNS servers for the current WiFi connection"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to change DNS servers. Common options: Google (8.8.8.8,8.8.4.4), Cloudflare (1.1.1.1,1.0.0.1)"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "primary_dns".to_string(),
                param_type: "string".to_string(),
                description: "Primary DNS server IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("8.8.8.8".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "secondary_dns".to_string(),
                param_type: "string".to_string(),
                description: "Secondary DNS server IP address (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("8.8.4.4".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_dns_set",
            "parameters": {
                "primary_dns": "8.8.8.8",
                "secondary_dns": "8.8.4.4"
            }
        })
    }

    fn example_output(&self) -> String {
        "DNS set to: 8.8.8.8, 8.8.4.4".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Wifi
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let primary_dns = parameters
            .get("primary_dns")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'primary_dns' parameter"))?;
        let secondary_dns = parameters.get("secondary_dns").and_then(|v| v.as_str());
        #[cfg(target_os = "windows")]
        {
            let interface_name = get_wifi_interface_name()?;
            if let Some(secondary) = secondary_dns {
                Command::new("netsh")
                    .args([
                        "interface",
                        "ip",
                        "set",
                        "dns",
                        &interface_name,
                        "static",
                        primary_dns,
                    ])
                    .output()?;
                Command::new("netsh")
                    .args(["interface", "ip", "add", "dns", &interface_name, secondary])
                    .output()?;
            } else {
                Command::new("netsh")
                    .args([
                        "interface",
                        "ip",
                        "set",
                        "dns",
                        &interface_name,
                        "static",
                        primary_dns,
                    ])
                    .output()?;
            }
        }
        #[cfg(target_os = "linux")]
        {
            let dns_string = if let Some(secondary) = secondary_dns {
                format!("{} {}", primary_dns, secondary)
            } else {
                primary_dns.to_string()
            };
            Command::new("nmcli")
                .args(["connection", "modify", "Wired", "ipv4.dns", &dns_string])
                .output()?;
            Command::new("nmcli")
                .args(["connection", "up", "Wired"])
                .output()?;
        }
        let dns_list = if let Some(secondary) = secondary_dns {
            format!("{}, {}", primary_dns, secondary)
        } else {
            primary_dns.to_string()
        };
        Ok(format!("DNS set to: {}", dns_list))
    }
}

#[cfg(target_os = "windows")]
fn get_wifi_interface_name() -> Result<String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains("名称") || line.contains("Name") {
            if let Some(name) = line.split(':').nth(1) {
                return Ok(name.trim().to_string());
            }
        }
    }

    Ok("Wi-Fi".to_string())
}
