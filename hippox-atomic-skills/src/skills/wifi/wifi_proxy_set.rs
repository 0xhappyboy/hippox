//! WiFi proxy set skill - set proxy for WiFi connection

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WifiProxySetSkill;

#[async_trait::async_trait]
impl Skill for WifiProxySetSkill {
    fn name(&self) -> &str {
        "wifi_proxy_set"
    }

    fn description(&self) -> &str {
        "Set HTTP/HTTPS proxy settings for the current WiFi connection"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to configure a proxy server for your WiFi connection. Supports HTTP, HTTPS, and SOCKS5 proxies."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "proxy_type".to_string(),
                param_type: "string".to_string(),
                description: "Proxy type: 'http', 'https', 'socks5', or 'none' to disable"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("http".to_string())),
                enum_values: Some(vec![
                    "http".to_string(),
                    "https".to_string(),
                    "socks5".to_string(),
                    "none".to_string(),
                ]),
            },
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description:
                    "Proxy server hostname or IP address (required unless proxy_type is 'none')"
                        .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("127.0.0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Proxy server port (required unless proxy_type is 'none')".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(8080.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Username for proxy authentication (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("user".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password for proxy authentication (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("pass".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "bypass_list".to_string(),
                param_type: "array".to_string(),
                description: "List of addresses to bypass proxy (e.g., localhost, 192.168.*)"
                    .to_string(),
                required: false,
                default: Some(json!(["localhost", "127.0.0.1"])),
                example: Some(json!(["localhost", "127.0.0.1", "192.168.*"])),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_proxy_set",
            "parameters": {
                "proxy_type": "http",
                "host": "127.0.0.1",
                "port": 8080,
                "bypass_list": ["localhost", "127.0.0.1", "192.168.*"]
            }
        })
    }

    fn example_output(&self) -> String {
        "HTTP proxy configured: 127.0.0.1:8080".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let proxy_type = parameters
            .get("proxy_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'proxy_type' parameter"))?;

        if proxy_type == "none" {
            disable_proxy()?;
            return Ok("Proxy disabled".to_string());
        }

        let host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'host' parameter"))?;

        let port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'port' parameter"))?;

        let username = parameters.get("username").and_then(|v| v.as_str());

        let password = parameters.get("password").and_then(|v| v.as_str());

        let bypass_list = parameters
            .get("bypass_list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
                    .join(",")
            })
            .unwrap_or_else(|| "localhost,127.0.0.1".to_string());

        set_proxy(proxy_type, host, port, username, password, &bypass_list)?;

        Ok(format!(
            "{} proxy configured: {}:{}",
            proxy_type.to_uppercase(),
            host,
            port
        ))
    }
}

#[cfg(target_os = "windows")]
fn set_proxy(
    proxy_type: &str,
    host: &str,
    port: u64,
    username: Option<&str>,
    password: Option<&str>,
    bypass_list: &str,
) -> Result<()> {
    let proxy_url = match proxy_type {
        "http" => format!("http://{}:{}", host, port),
        "https" => format!("https://{}:{}", host, port),
        "socks5" => format!("socks5://{}:{}", host, port),
        _ => anyhow::bail!("Unsupported proxy type: {}", proxy_type),
    };

    // Set proxy via netsh
    Command::new("netsh")
        .args([
            "winhttp",
            "set",
            "proxy",
            &proxy_url,
            &format!("bypass-list=\"{}\"", bypass_list),
        ])
        .output()?;

    // Set via registry for system-wide proxy
    Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v",
            "ProxyEnable",
            "/t",
            "REG_DWORD",
            "/d",
            "1",
            "/f",
        ])
        .output()?;
    Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v",
            "ProxyServer",
            "/t",
            "REG_SZ",
            "/d",
            &proxy_url,
            "/f",
        ])
        .output()?;

    if let Some(user) = username {
        if let Some(pass) = password {
            Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                    "/v",
                    "ProxyUser",
                    "/t",
                    "REG_SZ",
                    "/d",
                    user,
                    "/f",
                ])
                .output()?;
            Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                    "/v",
                    "ProxyPass",
                    "/t",
                    "REG_SZ",
                    "/d",
                    pass,
                    "/f",
                ])
                .output()?;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn set_proxy(
    proxy_type: &str,
    host: &str,
    port: u64,
    username: Option<&str>,
    password: Option<&str>,
    bypass_list: &str,
) -> Result<()> {
    let proxy_url = match proxy_type {
        "http" => format!("http://{}:{}", host, port),
        "https" => format!("https://{}:{}", host, port),
        "socks5" => format!("socks5://{}:{}", host, port),
        _ => anyhow::bail!("Unsupported proxy type: {}", proxy_type),
    };

    let auth = if let (Some(user), Some(pass)) = (username, password) {
        format!("{}:{}@", user, pass)
    } else {
        String::new()
    };

    let auth_proxy_url = match proxy_type {
        "http" => format!("http://{}{}:{}", auth, host, port),
        "https" => format!("https://{}{}:{}", auth, host, port),
        "socks5" => format!("socks5://{}{}:{}", auth, host, port),
        _ => proxy_url.clone(),
    };

    // Set environment variables via /etc/environment or gsettings
    Command::new("gsettings")
        .args(["set", "org.gnome.system.proxy", "mode", "manual"])
        .output()?;
    Command::new("gsettings")
        .args(["set", "org.gnome.system.proxy.http", "host", host])
        .output()?;
    Command::new("gsettings")
        .args([
            "set",
            "org.gnome.system.proxy.http",
            "port",
            &port.to_string(),
        ])
        .output()?;
    Command::new("gsettings")
        .args(["set", "org.gnome.system.proxy.https", "host", host])
        .output()?;
    Command::new("gsettings")
        .args([
            "set",
            "org.gnome.system.proxy.https",
            "port",
            &port.to_string(),
        ])
        .output()?;

    if proxy_type == "socks5" {
        Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.socks", "host", host])
            .output()?;
        Command::new("gsettings")
            .args([
                "set",
                "org.gnome.system.proxy.socks",
                "port",
                &port.to_string(),
            ])
            .output()?;
    }

    let bypass_array = format!("['{}']", bypass_list.replace(",', '"));
    Command::new("gsettings")
        .args([
            "set",
            "org.gnome.system.proxy",
            "ignore-hosts",
            &bypass_array,
        ])
        .output()?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_proxy(
    proxy_type: &str,
    host: &str,
    port: u64,
    username: Option<&str>,
    password: Option<&str>,
    bypass_list: &str,
) -> Result<()> {
    let service_name = get_wifi_service_name()?;

    let proxy_cmd = match proxy_type {
        "http" => "webproxy",
        "https" => "securewebproxy",
        "socks5" => "socksfirewallproxy",
        _ => anyhow::bail!("Unsupported proxy type: {}", proxy_type),
    };

    Command::new("networksetup")
        .args(["-set", proxy_cmd, &service_name, host, &port.to_string()])
        .output()?;

    if let (Some(user), Some(pass)) = (username, password) {
        Command::new("networksetup")
            .args([
                "-set",
                &format!("{}auth", proxy_cmd),
                &service_name,
                user,
                pass,
            ])
            .output()?;
    }

    Command::new("networksetup")
        .args([
            "-setproxybypassdomains",
            &service_name,
            &bypass_list.replace(',', " "),
        ])
        .output()?;

    Command::new("networksetup")
        .args(["-set", &format!("{}state", proxy_cmd), &service_name, "on"])
        .output()?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn disable_proxy() -> Result<()> {
    Command::new("netsh")
        .args(["winhttp", "reset", "proxy"])
        .output()?;
    Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "/v",
            "ProxyEnable",
            "/t",
            "REG_DWORD",
            "/d",
            "0",
            "/f",
        ])
        .output()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_proxy() -> Result<()> {
    Command::new("gsettings")
        .args(["set", "org.gnome.system.proxy", "mode", "none"])
        .output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_proxy() -> Result<()> {
    let service_name = get_wifi_service_name()?;

    Command::new("networksetup")
        .args(["-setwebproxystate", &service_name, "off"])
        .output()?;
    Command::new("networksetup")
        .args(["-setsecurewebproxystate", &service_name, "off"])
        .output()?;
    Command::new("networksetup")
        .args(["-setsocksfirewallproxystate", &service_name, "off"])
        .output()?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn get_wifi_service_name() -> Result<String> {
    let output = Command::new("networksetup")
        .args(["-listallhardwareports"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut current_device = String::new();
    for line in stdout.lines() {
        if line.contains("Hardware Port: Wi-Fi") || line.contains("Hardware Port: AirPort") {
            if let Some(device_line) = stdout.lines().skip_while(|l| *l != line).nth(1) {
                if device_line.contains("Device:") {
                    if let Some(device) = device_line.split(':').nth(1) {
                        current_device = device.trim().to_string();
                    }
                }
            }
        }
    }

    if current_device.is_empty() {
        Ok("Wi-Fi".to_string())
    } else {
        Ok(current_device)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn set_proxy(
    _proxy_type: &str,
    _host: &str,
    _port: u64,
    _username: Option<&str>,
    _password: Option<&str>,
    _bypass_list: &str,
) -> Result<()> {
    anyhow::bail!("Proxy configuration not implemented on this platform")
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn disable_proxy() -> Result<()> {
    anyhow::bail!("Proxy configuration not implemented on this platform")
}
