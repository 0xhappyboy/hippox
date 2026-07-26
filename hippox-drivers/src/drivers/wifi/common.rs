//! Shared utilities for WiFi control across platforms
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};
/// WiFi network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiNetwork {
    pub ssid: String,
    pub signal_strength: i32,
    pub encryption_type: String,
    pub is_connected: bool,
    pub bssid: Option<String>,
    pub channel: Option<u32>,
    pub frequency: Option<u32>,
}
/// WiFi connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiStatus {
    pub connected: bool,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub ip_address: Option<String>,
    pub signal_strength: Option<i32>,
    pub frequency: Option<u32>,
    pub channel: Option<u32>,
    pub link_speed: Option<u32>,
}
/// WiFi interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiInterface {
    pub name: String,
    pub mac_address: String,
    pub state: String,
    pub is_default: bool,
}
/// WiFi quality analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiQualityAnalysis {
    pub current_channel: u32,
    pub recommended_channel: u32,
    pub score: u32,
    pub noise_level: i32,
    pub signal_to_noise: i32,
    pub recommendations: Vec<String>,
}
/// Get current WiFi status (Windows)
#[cfg(target_os = "windows")]
pub fn get_wifi_status() -> DriverResult<WiFiStatus> {
    debug!("Getting WiFi status on Windows");
    let output = Command::new("netsh").args(["wlan", "show", "interfaces"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute netsh: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut status = WiFiStatus {
        connected: false,
        ssid: None,
        bssid: None,
        ip_address: None,
        signal_strength: None,
        frequency: None,
        channel: None,
        link_speed: None,
    };
    for line in stdout.lines() {
        if line.contains("SSID") && !line.contains("BSSID") {
            if let Some(ssid) = line.split(':').nth(1) {
                status.ssid = Some(ssid.trim().to_string());
                if ssid.trim() != "" {
                    status.connected = true;
                }
            }
        }
        if line.contains("BSSID") {
            if let Some(bssid) = line.split(':').nth(1) {
                status.bssid = Some(bssid.trim().to_string());
            }
        }
        if line.contains("信号") || line.contains("Signal") {
            if let Some(signal) = line.split(':').nth(1) {
                let signal_str = signal.trim().replace("%", "");
                if let Ok(signal_val) = signal_str.parse::<i32>() {
                    status.signal_strength = Some(signal_val);
                }
            }
        }
        if line.contains("频道") || line.contains("Channel") {
            if let Some(channel) = line.split(':').nth(1) {
                if let Ok(channel_val) = channel.trim().parse::<u32>() {
                    status.channel = Some(channel_val);
                }
            }
        }
        if line.contains("速度") || line.contains("Speed") {
            if let Some(speed) = line.split(':').nth(1) {
                let speed_str = speed.trim().replace("Mbps", "");
                if let Ok(speed_val) = speed_str.parse::<u32>() {
                    status.link_speed = Some(speed_val);
                }
            }
        }
    }
    // Get IP address
    let ip_output = Command::new("ipconfig").output().map_err(|e| {
        let err_msg = format!("Failed to execute ipconfig: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let ip_stdout = String::from_utf8_lossy(&ip_output.stdout);
    if let Some(ssid) = &status.ssid {
        let mut in_section = false;
        for line in ip_stdout.lines() {
            if line.contains(ssid) || (line.contains("无线") && line.contains("适配器")) {
                in_section = true;
            }
            if in_section && line.contains("IPv4") {
                if let Some(ip) = line.split(':').nth(1) {
                    status.ip_address = Some(ip.trim().to_string());
                    break;
                }
            }
        }
    }
    info!("WiFi status retrieved: connected={}", status.connected);
    return Ok(status);
}
/// Get current WiFi status (Linux)
#[cfg(target_os = "linux")]
pub fn get_wifi_status() -> DriverResult<WiFiStatus> {
    debug!("Getting WiFi status on Linux");
    let output = Command::new("iwgetid").arg("-r").output();
    let mut status = WiFiStatus {
        connected: false,
        ssid: None,
        bssid: None,
        ip_address: None,
        signal_strength: None,
        frequency: None,
        channel: None,
        link_speed: None,
    };
    if let Ok(output) = output {
        let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ssid.is_empty() {
            status.connected = true;
            status.ssid = Some(ssid);
        }
    }
    // Get IP address
    let ip_output = Command::new("hostname").arg("-I").output();
    if let Ok(output) = ip_output {
        let ips = String::from_utf8_lossy(&output.stdout);
        status.ip_address = ips.split_whitespace().next().map(|s| s.to_string());
    }
    info!("WiFi status retrieved: connected={}", status.connected);
    return Ok(status);
}
/// Get current WiFi status (macOS)
#[cfg(target_os = "macos")]
pub fn get_wifi_status() -> DriverResult<WiFiStatus> {
    debug!("Getting WiFi status on macOS");
    let airport_path = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";
    let output = Command::new(airport_path).arg("-I").output().map_err(|e| {
        let err_msg = format!("Failed to execute airport command: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let mut status = WiFiStatus {
        connected: false,
        ssid: None,
        bssid: None,
        ip_address: None,
        signal_strength: None,
        frequency: None,
        channel: None,
        link_speed: None,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("SSID:") {
            if let Some(ssid) = line.split(':').nth(1) {
                let ssid = ssid.trim();
                if !ssid.is_empty() {
                    status.connected = true;
                    status.ssid = Some(ssid.to_string());
                }
            }
        }
        if line.contains("BSSID:") {
            if let Some(bssid) = line.split(':').nth(1) {
                status.bssid = Some(bssid.trim().to_string());
            }
        }
        if line.contains("agrCtlRSSI:") {
            if let Some(signal) = line.split(':').nth(1) {
                if let Ok(signal_val) = signal.trim().parse::<i32>() {
                    status.signal_strength = Some(signal_val);
                }
            }
        }
        if line.contains("channel:") {
            if let Some(channel) = line.split(':').nth(1) {
                let channel_str = channel.trim().split(',').next().unwrap_or("");
                if let Ok(channel_val) = channel_str.parse::<u32>() {
                    status.channel = Some(channel_val);
                }
            }
        }
    }
    // Get IP address
    let ip_output = Command::new("ifconfig").arg("en0").output().map_err(|e| {
        let err_msg = format!("Failed to execute ifconfig: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&ip_output.stdout);
    for line in stdout.lines() {
        if line.contains("inet ") && !line.contains("127.0.0.1") {
            if let Some(ip) = line.split_whitespace().nth(1) {
                status.ip_address = Some(ip.to_string());
                break;
            }
        }
    }
    info!("WiFi status retrieved: connected={}", status.connected);
    return Ok(status);
}
/// Scan for WiFi networks (Windows)
#[cfg(target_os = "windows")]
pub fn scan_wifi_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Scanning WiFi networks on Windows");
    // First, run a scan
    let _ = Command::new("netsh").args(["wlan", "show", "networks", "mode=bssid"]).output();
    let output = Command::new("netsh").args(["wlan", "show", "networks", "mode=bssid"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute netsh: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    let mut current_network: Option<WiFiNetwork> = None;
    for line in stdout.lines() {
        if line.contains("SSID") && !line.contains("BSSID") {
            if let Some(ssid) = line.split(':').nth(1) {
                let ssid = ssid.trim();
                if !ssid.is_empty() && ssid != "" {
                    current_network = Some(WiFiNetwork {
                        ssid: ssid.to_string(),
                        signal_strength: 0,
                        encryption_type: "Unknown".to_string(),
                        is_connected: false,
                        bssid: None,
                        channel: None,
                        frequency: None,
                    });
                }
            }
        }
        if let Some(ref mut network) = current_network {
            if line.contains("信号") || line.contains("Signal") {
                if let Some(signal) = line.split(':').nth(1) {
                    let signal_str = signal.trim().replace("%", "");
                    if let Ok(signal_val) = signal_str.parse::<i32>() {
                        network.signal_strength = signal_val;
                    }
                }
            }
            if line.contains("身份验证") || line.contains("Authentication") {
                if let Some(auth) = line.split(':').nth(1) {
                    network.encryption_type = auth.trim().to_string();
                }
            }
            if line.contains("BSSID") {
                if let Some(bssid) = line.split(':').nth(1) {
                    network.bssid = Some(bssid.trim().to_string());
                }
            }
            if line.contains("频道") || line.contains("Channel") {
                if let Some(channel) = line.split(':').nth(1) {
                    if let Ok(channel_val) = channel.trim().parse::<u32>() {
                        network.channel = Some(channel_val);
                    }
                }
            }
            if line.is_empty() && network.ssid != "" {
                networks.push(network.clone());
                current_network = None;
            }
        }
    }
    // Remove duplicates by SSID (keep strongest signal)
    let mut unique_networks = std::collections::HashMap::new();
    for network in networks {
        unique_networks
            .entry(network.ssid.clone())
            .and_modify(|existing: &mut WiFiNetwork| {
                if network.signal_strength > existing.signal_strength {
                    *existing = network.clone();
                }
            })
            .or_insert(network);
    }
    let mut result: Vec<WiFiNetwork> = unique_networks.into_values().collect();
    result.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
    info!("Found {} WiFi networks", result.len());
    return Ok(result);
}
/// Scan for WiFi networks (Linux)
#[cfg(target_os = "linux")]
pub fn scan_wifi_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Scanning WiFi networks on Linux");
    let output = Command::new("nmcli").args(["dev", "wifi", "list"]).output();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut networks = Vec::new();
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ssid = parts[1].to_string();
                let signal_str = parts[2].replace("%", "");
                let signal = signal_str.parse::<i32>().unwrap_or(0);
                let encryption = if parts.len() > 3 { parts[3].to_string() } else { "Unknown".to_string() };
                networks.push(WiFiNetwork {
                    ssid,
                    signal_strength: signal,
                    encryption_type: encryption,
                    is_connected: false,
                    bssid: None,
                    channel: None,
                    frequency: None,
                });
            }
        }
        networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
        info!("Found {} WiFi networks", networks.len());
        return Ok(networks);
    } else {
        info!("No WiFi networks found");
        return Ok(Vec::new());
    }
}
/// Scan for WiFi networks (macOS)
#[cfg(target_os = "macos")]
pub fn scan_wifi_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Scanning WiFi networks on macOS");
    let airport_path = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";
    let output = Command::new(airport_path).args(["-s"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute airport command: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    for line in stdout.lines().skip(1) {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let ssid = parts[0].to_string();
            let bssid = if parts.len() > 1 { Some(parts[1].to_string()) } else { None };
            let signal = parts[2].parse::<i32>().unwrap_or(0);
            networks.push(WiFiNetwork {
                ssid,
                signal_strength: signal,
                encryption_type: "Unknown".to_string(),
                is_connected: false,
                bssid,
                channel: None,
                frequency: None,
            });
        }
    }
    networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
    info!("Found {} WiFi networks", networks.len());
    return Ok(networks);
}
/// Connect to WiFi network (Windows)
#[cfg(target_os = "windows")]
pub fn connect_wifi(ssid: &str, password: Option<&str>) -> DriverResult<()> {
    debug!("Connecting to WiFi network: {}", ssid);
    // Create profile XML
    let profile_xml = if let Some(pwd) = password {
        format!(
            r#"<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>{}</name>
    <SSIDConfig>
        <SSID>
            <name>{}</name>
        </SSID>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>auto</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>WPA2PSK</authentication>
                <encryption>AES</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
            <sharedKey>
                <keyType>passPhrase</keyType>
                <protected>false</protected>
                <keyMaterial>{}</keyMaterial>
            </sharedKey>
        </security>
    </MSM>
</WLANProfile>"#,
            ssid, ssid, pwd
        )
    } else {
        format!(
            r#"<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>{}</name>
    <SSIDConfig>
        <SSID>
            <name>{}</name>
        </SSID>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>auto</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>open</authentication>
                <encryption>none</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
        </security>
    </MSM>
</WLANProfile>"#,
            ssid, ssid
        )
    };
    let profile_path = std::env::temp_dir().join(format!("{}.xml", ssid));
    std::fs::write(&profile_path, profile_xml).map_err(|e| {
        let err_msg = format!("Failed to write profile file: {}", e);
        warn!("{}", err_msg);
        return DriverError::io(err_msg);
    })?;
    let profile_path_str = profile_path.to_str().unwrap();
    Command::new("netsh").args(["wlan", "add", "profile", "filename=", profile_path_str]).output().map_err(|e| {
        let err_msg = format!("Failed to add profile: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    Command::new("netsh").args(["wlan", "connect", "name=", ssid]).output().map_err(|e| {
        let err_msg = format!("Failed to connect: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let _ = std::fs::remove_file(profile_path);
    info!("Connected to WiFi network: {}", ssid);
    return Ok(());
}
/// Connect to WiFi network (Linux)
#[cfg(target_os = "linux")]
pub fn connect_wifi(ssid: &str, password: Option<&str>) -> DriverResult<()> {
    debug!("Connecting to WiFi network: {}", ssid);
    if let Some(pwd) = password {
        Command::new("nmcli").args(["dev", "wifi", "connect", ssid, "password", pwd]).output().map_err(|e| {
            let err_msg = format!("Failed to connect to WiFi: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    } else {
        Command::new("nmcli").args(["dev", "wifi", "connect", ssid]).output().map_err(|e| {
            let err_msg = format!("Failed to connect to WiFi: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    }
    info!("Connected to WiFi network: {}", ssid);
    return Ok(());
}
/// Connect to WiFi network (macOS)
#[cfg(target_os = "macos")]
pub fn connect_wifi(ssid: &str, password: Option<&str>) -> DriverResult<()> {
    debug!("Connecting to WiFi network: {}", ssid);
    let airport_path = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";
    if let Some(pwd) = password {
        Command::new(airport_path).args(["--associate=", ssid, "--password=", pwd]).output().map_err(|e| {
            let err_msg = format!("Failed to connect to WiFi: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    } else {
        Command::new(airport_path).args(["--associate=", ssid]).output().map_err(|e| {
            let err_msg = format!("Failed to connect to WiFi: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    }
    info!("Connected to WiFi network: {}", ssid);
    return Ok(());
}
/// Disconnect from current WiFi
#[cfg(target_os = "windows")]
pub fn disconnect_wifi() -> DriverResult<()> {
    debug!("Disconnecting from WiFi on Windows");
    Command::new("netsh").args(["wlan", "disconnect"]).output().map_err(|e| {
        let err_msg = format!("Failed to disconnect: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Disconnected from WiFi");
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn disconnect_wifi() -> DriverResult<()> {
    debug!("Disconnecting from WiFi on Linux");
    Command::new("nmcli").args(["dev", "disconnect", "wlan0"]).output().map_err(|e| {
        let err_msg = format!("Failed to disconnect: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Disconnected from WiFi");
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn disconnect_wifi() -> DriverResult<()> {
    debug!("Disconnecting from WiFi on macOS");
    Command::new("networksetup").args(["-setairportpower", "en0", "off"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi off: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    Command::new("networksetup").args(["-setairportpower", "en0", "on"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi on: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Disconnected from WiFi");
    return Ok(());
}
/// Forget a saved WiFi network
#[cfg(target_os = "windows")]
pub fn forget_wifi(ssid: &str) -> DriverResult<()> {
    debug!("Forgetting WiFi network: {}", ssid);
    Command::new("netsh").args(["wlan", "delete", "profile", "name=", ssid]).output().map_err(|e| {
        let err_msg = format!("Failed to forget WiFi network: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Forgot WiFi network: {}", ssid);
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn forget_wifi(ssid: &str) -> DriverResult<()> {
    debug!("Forgetting WiFi network: {}", ssid);
    Command::new("nmcli").args(["connection", "delete", ssid]).output().map_err(|e| {
        let err_msg = format!("Failed to forget WiFi network: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Forgot WiFi network: {}", ssid);
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn forget_wifi(ssid: &str) -> DriverResult<()> {
    debug!("Forgetting WiFi network: {}", ssid);
    Command::new("networksetup").args(["-removepreferredwirelessnetwork", "en0", ssid]).output().map_err(|e| {
        let err_msg = format!("Failed to forget WiFi network: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Forgot WiFi network: {}", ssid);
    return Ok(());
}
/// Turn WiFi on
#[cfg(target_os = "windows")]
pub fn wifi_on() -> DriverResult<()> {
    debug!("Turning WiFi on on Windows");
    Command::new("netsh").args(["interface", "set", "interface", "name=\"Wi-Fi\"", "admin=ENABLED"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi on: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("WiFi turned on");
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn wifi_on() -> DriverResult<()> {
    debug!("Turning WiFi on on Linux");
    Command::new("nmcli").args(["radio", "wifi", "on"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi on: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("WiFi turned on");
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn wifi_on() -> DriverResult<()> {
    debug!("Turning WiFi on on macOS");
    Command::new("networksetup").args(["-setairportpower", "en0", "on"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi on: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("WiFi turned on");
    return Ok(());
}
/// Turn WiFi off
#[cfg(target_os = "windows")]
pub fn wifi_off() -> DriverResult<()> {
    debug!("Turning WiFi off on Windows");
    Command::new("netsh").args(["interface", "set", "interface", "name=\"Wi-Fi\"", "admin=DISABLED"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi off: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("WiFi turned off");
    return Ok(());
}
#[cfg(target_os = "linux")]
pub fn wifi_off() -> DriverResult<()> {
    debug!("Turning WiFi off on Linux");
    Command::new("nmcli").args(["radio", "wifi", "off"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi off: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("WiFi turned off");
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn wifi_off() -> DriverResult<()> {
    debug!("Turning WiFi off on macOS");
    Command::new("networksetup").args(["-setairportpower", "en0", "off"]).output().map_err(|e| {
        let err_msg = format!("Failed to turn WiFi off: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("WiFi turned off");
    return Ok(());
}
/// List saved WiFi networks
#[cfg(target_os = "windows")]
pub fn list_saved_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Listing saved networks on Windows");
    let output = Command::new("netsh").args(["wlan", "show", "profiles"]).output().map_err(|e| {
        let err_msg = format!("Failed to list saved networks: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    for line in stdout.lines() {
        if line.contains(":") && line.contains("所有用户配置文件") {
            if let Some(ssid) = line.split(':').nth(1) {
                let ssid = ssid.trim();
                if !ssid.is_empty() {
                    networks.push(WiFiNetwork {
                        ssid: ssid.to_string(),
                        signal_strength: 0,
                        encryption_type: "Saved".to_string(),
                        is_connected: false,
                        bssid: None,
                        channel: None,
                        frequency: None,
                    });
                }
            }
        }
    }
    info!("Found {} saved networks", networks.len());
    return Ok(networks);
}
#[cfg(target_os = "linux")]
pub fn list_saved_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Listing saved networks on Linux");
    let output = Command::new("nmcli").args(["connection", "show"]).output().map_err(|e| {
        let err_msg = format!("Failed to list saved networks: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            let name = parts[0];
            if name.starts_with("Wifi") || !name.contains(":") {
                networks.push(WiFiNetwork {
                    ssid: name.to_string(),
                    signal_strength: 0,
                    encryption_type: "Saved".to_string(),
                    is_connected: false,
                    bssid: None,
                    channel: None,
                    frequency: None,
                });
            }
        }
    }
    info!("Found {} saved networks", networks.len());
    return Ok(networks);
}
#[cfg(target_os = "macos")]
pub fn list_saved_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Listing saved networks on macOS");
    let output = Command::new("networksetup").args(["-listpreferredwirelessnetworks", "en0"]).output().map_err(|e| {
        let err_msg = format!("Failed to list saved networks: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            let ssid = parts[0].trim_matches(|c| c == '*' || c == ' ');
            if !ssid.is_empty() {
                networks.push(WiFiNetwork {
                    ssid: ssid.to_string(),
                    signal_strength: 0,
                    encryption_type: "Saved".to_string(),
                    is_connected: false,
                    bssid: None,
                    channel: None,
                    frequency: None,
                });
            }
        }
    }
    info!("Found {} saved networks", networks.len());
    return Ok(networks);
}
/// List WiFi interfaces
#[cfg(target_os = "windows")]
pub fn list_interfaces() -> DriverResult<Vec<WiFiInterface>> {
    debug!("Listing WiFi interfaces on Windows");
    let output = Command::new("netsh").args(["wlan", "show", "interfaces"]).output().map_err(|e| {
        let err_msg = format!("Failed to list interfaces: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();
    for line in stdout.lines() {
        if line.contains("名称") || line.contains("Name") {
            if let Some(name) = line.split(':').nth(1) {
                interfaces.push(WiFiInterface {
                    name: name.trim().to_string(),
                    mac_address: "Unknown".to_string(),
                    state: "Unknown".to_string(),
                    is_default: true,
                });
            }
        }
    }
    info!("Found {} WiFi interfaces", interfaces.len());
    return Ok(interfaces);
}
#[cfg(target_os = "linux")]
pub fn list_interfaces() -> DriverResult<Vec<WiFiInterface>> {
    debug!("Listing WiFi interfaces on Linux");
    let output = Command::new("iwconfig").output().map_err(|e| {
        let err_msg = format!("Failed to execute iwconfig: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();
    for line in stdout.lines() {
        if line.contains("IEEE 802.11") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 1 {
                let name = parts[0];
                interfaces.push(WiFiInterface {
                    name: name.to_string(),
                    mac_address: "Unknown".to_string(),
                    state: "Unknown".to_string(),
                    is_default: name.contains("wlan"),
                });
            }
        }
    }
    info!("Found {} WiFi interfaces", interfaces.len());
    return Ok(interfaces);
}
#[cfg(target_os = "macos")]
pub fn list_interfaces() -> DriverResult<Vec<WiFiInterface>> {
    debug!("Listing WiFi interfaces on macOS");
    let output = Command::new("networksetup").args(["-listallhardwareports"]).output().map_err(|e| {
        let err_msg = format!("Failed to list interfaces: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();
    let mut current_device = String::new();
    for line in stdout.lines() {
        if line.contains("Device:") {
            if let Some(device) = line.split(':').nth(1) {
                current_device = device.trim().to_string();
            }
        }
        if line.contains("Wi-Fi") || line.contains("AirPort") {
            if !current_device.is_empty() {
                interfaces.push(WiFiInterface {
                    name: current_device.clone(),
                    mac_address: "Unknown".to_string(),
                    state: "Unknown".to_string(),
                    is_default: current_device == "en0",
                });
            }
        }
    }
    info!("Found {} WiFi interfaces", interfaces.len());
    return Ok(interfaces);
}
/// Ping gateway
pub fn ping_gateway(gateway: &str) -> DriverResult<(bool, u64)> {
    debug!("Pinging gateway: {}", gateway);
    let output = Command::new("ping").args(["-n", "4", "-w", "3", gateway]).output().map_err(|e| {
        let err_msg = format!("Failed to execute ping: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Extract average time
    let mut avg_time = 0u64;
    if let Some(time_pos) = stdout.find("Average = ") {
        let time_str = &stdout[time_pos + 10..];
        if let Some(end) = time_str.find("ms") {
            if let Ok(time) = time_str[..end].trim().parse::<f64>() {
                avg_time = time as u64;
            }
        }
    }
    info!("Ping to gateway {}: success={}, avg_time={}ms", gateway, success, avg_time);
    return Ok((success, avg_time));
}
/// Get default gateway
#[cfg(target_os = "windows")]
pub fn get_default_gateway() -> DriverResult<String> {
    debug!("Getting default gateway on Windows");
    let output = Command::new("ipconfig").output().map_err(|e| {
        let err_msg = format!("Failed to execute ipconfig: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("Default Gateway") && line.contains(":") {
            if let Some(gateway) = line.split(':').nth(1) {
                let gateway = gateway.trim();
                if gateway != "" {
                    info!("Default gateway: {}", gateway);
                    return Ok(gateway.to_string());
                }
            }
        }
    }
    info!("Default gateway not found, using 8.8.8.8");
    return Ok("8.8.8.8".to_string());
}
#[cfg(target_os = "linux")]
pub fn get_default_gateway() -> DriverResult<String> {
    debug!("Getting default gateway on Linux");
    let output = Command::new("ip").args(["route", "show", "default"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute ip: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.split_whitespace().collect();
    if parts.len() >= 3 {
        let gateway = parts[2].to_string();
        info!("Default gateway: {}", gateway);
        return Ok(gateway);
    }
    info!("Default gateway not found, using 8.8.8.8");
    return Ok("8.8.8.8".to_string());
}
#[cfg(target_os = "macos")]
pub fn get_default_gateway() -> DriverResult<String> {
    debug!("Getting default gateway on macOS");
    let output = Command::new("netstat").args(["-rn"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute netstat: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("default") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let gateway = parts[1].to_string();
                info!("Default gateway: {}", gateway);
                return Ok(gateway);
            }
        }
    }
    info!("Default gateway not found, using 8.8.8.8");
    return Ok("8.8.8.8".to_string());
}
// Platform-agnostic implementations for other platforms
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn get_wifi_status() -> DriverResult<WiFiStatus> {
    debug!("Getting WiFi status on unsupported platform");
    return Ok(WiFiStatus {
        connected: false,
        ssid: None,
        bssid: None,
        ip_address: None,
        signal_strength: None,
        frequency: None,
        channel: None,
        link_speed: None,
    });
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn scan_wifi_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Scanning WiFi networks on unsupported platform");
    return Ok(Vec::new());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn connect_wifi(ssid: &str, password: Option<&str>) -> DriverResult<()> {
    let err_msg = "WiFi control not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn disconnect_wifi() -> DriverResult<()> {
    let err_msg = "WiFi control not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn forget_wifi(ssid: &str) -> DriverResult<()> {
    let err_msg = "WiFi control not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn wifi_on() -> DriverResult<()> {
    let err_msg = "WiFi control not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn wifi_off() -> DriverResult<()> {
    let err_msg = "WiFi control not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn list_saved_networks() -> DriverResult<Vec<WiFiNetwork>> {
    debug!("Listing saved networks on unsupported platform");
    return Ok(Vec::new());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn list_interfaces() -> DriverResult<Vec<WiFiInterface>> {
    debug!("Listing interfaces on unsupported platform");
    return Ok(Vec::new());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn get_default_gateway() -> DriverResult<String> {
    debug!("Getting default gateway on unsupported platform");
    return Ok("8.8.8.8".to_string());
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn ping_gateway(gateway: &str) -> DriverResult<(bool, u64)> {
    debug!("Pinging gateway on unsupported platform");
    return Ok((false, 0));
}
