//! Shared utilities for Bluetooth control across platforms

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Bluetooth device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub name: String,
    pub mac_address: String,
    pub device_type: String,
    pub paired: bool,
    pub connected: bool,
    pub rssi: Option<i32>,
    pub battery_level: Option<u8>,
}

/// Bluetooth adapter status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothAdapterStatus {
    pub powered_on: bool,
    pub discoverable: bool,
    pub pairable: bool,
    pub name: String,
    pub mac_address: String,
    pub discoverable_timeout: u32,
}

/// Bluetooth service/characteristic info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothService {
    pub uuid: String,
    pub name: String,
    pub primary: bool,
    pub characteristics: Vec<BluetoothCharacteristic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothCharacteristic {
    pub uuid: String,
    pub name: String,
    pub properties: Vec<String>,
    pub value: Option<Vec<u8>>,
}

/// Get Bluetooth adapter status (Windows)
#[cfg(target_os = "windows")]
pub fn get_adapter_status() -> Result<BluetoothAdapterStatus> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-PnpDevice -Class Bluetooth | Select-Object Status, FriendlyName",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut powered_on = false;
    let mut name = "Unknown".to_string();

    for line in stdout.lines() {
        if line.contains("OK") || line.contains("正在运行") {
            powered_on = true;
        }
        if line.contains("Bluetooth") && !line.contains("Status") {
            name = line.trim().to_string();
        }
    }

    Ok(BluetoothAdapterStatus {
        powered_on,
        discoverable: false,
        pairable: true,
        name,
        mac_address: get_mac_address()?,
        discoverable_timeout: 120,
    })
}

/// Get Bluetooth adapter status (Linux)
#[cfg(target_os = "linux")]
pub fn get_adapter_status() -> Result<BluetoothAdapterStatus> {
    let output = Command::new("bluetoothctl").args(["show"]).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut powered_on = false;
    let mut discoverable = false;
    let mut pairable = true;
    let mut name = "Unknown".to_string();
    let mut mac_address = "Unknown".to_string();
    let mut discoverable_timeout = 0;

    for line in stdout.lines() {
        if line.contains("Powered:") && line.contains("yes") {
            powered_on = true;
        }
        if line.contains("Discoverable:") && line.contains("yes") {
            discoverable = true;
        }
        if line.contains("Name:") {
            if let Some(n) = line.split(':').nth(1) {
                name = n.trim().to_string();
            }
        }
        if line.contains("Address:") {
            if let Some(addr) = line.split(':').nth(1) {
                mac_address = addr.trim().to_string();
            }
        }
        if line.contains("DiscoverableTimeout:") {
            if let Some(t) = line.split(':').nth(1) {
                discoverable_timeout = t.trim().parse().unwrap_or(0);
            }
        }
    }

    Ok(BluetoothAdapterStatus {
        powered_on,
        discoverable,
        pairable,
        name,
        mac_address,
        discoverable_timeout,
    })
}

/// Get Bluetooth adapter status (macOS)
#[cfg(target_os = "macos")]
pub fn get_adapter_status() -> Result<BluetoothAdapterStatus> {
    let output = Command::new("system_profiler")
        .args(["SPBluetoothDataType"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut powered_on = false;
    let mut name = "Unknown".to_string();
    let mut mac_address = "Unknown".to_string();

    for line in stdout.lines() {
        if line.contains("Bluetooth Power: On") {
            powered_on = true;
        }
        if line.contains("Name:") {
            if let Some(n) = line.split(':').nth(1) {
                name = n.trim().to_string();
            }
        }
        if line.contains("Address:") {
            if let Some(addr) = line.split(':').nth(1) {
                mac_address = addr.trim().to_string();
            }
        }
    }

    Ok(BluetoothAdapterStatus {
        powered_on,
        discoverable: true,
        pairable: true,
        name,
        mac_address,
        discoverable_timeout: 120,
    })
}

/// Get Bluetooth MAC address
pub fn get_mac_address() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("bluetoothctl").args(["show"]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Address:") {
                if let Some(addr) = line.split(':').nth(1) {
                    return Ok(addr.trim().to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("getmac").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Bluetooth") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 1 {
                    return Ok(parts[0].to_string());
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("system_profiler")
            .args(["SPBluetoothDataType"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Address:") {
                if let Some(addr) = line.split(':').nth(1) {
                    return Ok(addr.trim().to_string());
                }
            }
        }
    }

    Ok("Unknown".to_string())
}

/// Turn Bluetooth on
#[cfg(target_os = "linux")]
pub fn bluetooth_on() -> Result<()> {
    Command::new("bluetoothctl")
        .args(["power", "on"])
        .output()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn bluetooth_on() -> Result<()> {
    Command::new("powershell")
        .args([
            "-Command",
            "Enable-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue",
        ])
        .output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn bluetooth_on() -> Result<()> {
    Command::new("blueutil")
        .args(["--power", "1"])
        .output()
        .map_err(|_| anyhow::anyhow!("blueutil not installed. Run: brew install blueutil"))?;
    Ok(())
}

/// Turn Bluetooth off
#[cfg(target_os = "linux")]
pub fn bluetooth_off() -> Result<()> {
    Command::new("bluetoothctl")
        .args(["power", "off"])
        .output()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn bluetooth_off() -> Result<()> {
    Command::new("powershell")
        .args([
            "-Command",
            "Disable-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue",
        ])
        .output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn bluetooth_off() -> Result<()> {
    Command::new("blueutil")
        .args(["--power", "0"])
        .output()
        .map_err(|_| anyhow::anyhow!("blueutil not installed. Run: brew install blueutil"))?;
    Ok(())
}

/// Scan for Bluetooth devices
#[cfg(target_os = "linux")]
pub fn scan_devices() -> Result<Vec<BluetoothDevice>> {
    // Start scan
    let _ = Command::new("bluetoothctl").args(["scan", "on"]).output();

    // Wait for scan results
    std::thread::sleep(std::time::Duration::from_secs(5));

    let output = Command::new("bluetoothctl").args(["devices"]).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("Device") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                devices.push(BluetoothDevice {
                    mac_address: parts[1].to_string(),
                    name: parts[2..].join(" "),
                    device_type: "Unknown".to_string(),
                    paired: false,
                    connected: false,
                    rssi: None,
                    battery_level: None,
                });
            }
        }
    }

    // Stop scan
    let _ = Command::new("bluetoothctl").args(["scan", "off"]).output();

    Ok(devices)
}

#[cfg(target_os = "windows")]
pub fn scan_devices() -> Result<Vec<BluetoothDevice>> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-PnpDevice -Class Bluetooth | Select-Object FriendlyName, InstanceId",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 && !line.contains("Bluetooth") {
            devices.push(BluetoothDevice {
                name: parts[0].to_string(),
                mac_address: "Unknown".to_string(),
                device_type: "Unknown".to_string(),
                paired: false,
                connected: false,
                rssi: None,
                battery_level: None,
            });
        }
    }

    Ok(devices)
}

#[cfg(target_os = "macos")]
pub fn scan_devices() -> Result<Vec<BluetoothDevice>> {
    let output = Command::new("system_profiler")
        .args(["SPBluetoothDataType"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    let mut current_device: Option<BluetoothDevice> = None;

    for line in stdout.lines() {
        if line.contains("Fully Qualified") {
            if let Some(device) = current_device.take() {
                devices.push(device);
            }
            if let Some(name) = line.split(':').nth(1) {
                current_device = Some(BluetoothDevice {
                    name: name.trim().to_string(),
                    mac_address: "Unknown".to_string(),
                    device_type: "Unknown".to_string(),
                    paired: false,
                    connected: false,
                    rssi: None,
                    battery_level: None,
                });
            }
        }
        if let Some(ref mut device) = current_device {
            if line.contains("Address:") {
                if let Some(addr) = line.split(':').nth(1) {
                    device.mac_address = addr.trim().to_string();
                }
            }
            if line.contains("Connected: Yes") {
                device.connected = true;
            }
            if line.contains("Paired: Yes") {
                device.paired = true;
            }
        }
    }

    if let Some(device) = current_device {
        devices.push(device);
    }

    Ok(devices)
}

/// Pair with a device
#[cfg(target_os = "linux")]
pub fn pair_device(mac_address: &str) -> Result<()> {
    Command::new("bluetoothctl")
        .args(["pair", mac_address])
        .output()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn pair_device(mac_address: &str) -> Result<()> {
    anyhow::bail!("Pairing on this platform requires GUI interaction")
}

/// Unpair a device
#[cfg(target_os = "linux")]
pub fn unpair_device(mac_address: &str) -> Result<()> {
    Command::new("bluetoothctl")
        .args(["remove", mac_address])
        .output()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn unpair_device(mac_address: &str) -> Result<()> {
    anyhow::bail!("Unpairing on this platform requires GUI interaction")
}

/// List paired devices
#[cfg(target_os = "linux")]
pub fn list_paired_devices() -> Result<Vec<BluetoothDevice>> {
    let output = Command::new("bluetoothctl")
        .args(["paired-devices"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("Device") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                devices.push(BluetoothDevice {
                    mac_address: parts[1].to_string(),
                    name: parts[2..].join(" "),
                    device_type: "Unknown".to_string(),
                    paired: true,
                    connected: false,
                    rssi: None,
                    battery_level: None,
                });
            }
        }
    }

    Ok(devices)
}

#[cfg(target_os = "windows")]
pub fn list_paired_devices() -> Result<Vec<BluetoothDevice>> {
    let output = Command::new("powershell")
        .args(["-Command", "Get-PnpDevice -Class Bluetooth | Where-Object {$_.FriendlyName -notlike '*Radio*'} | Select-Object FriendlyName, Status"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            devices.push(BluetoothDevice {
                name: parts[0].to_string(),
                mac_address: "Unknown".to_string(),
                device_type: "Unknown".to_string(),
                paired: true,
                connected: parts.contains(&"OK"),
                rssi: None,
                battery_level: None,
            });
        }
    }

    Ok(devices)
}

#[cfg(target_os = "macos")]
pub fn list_paired_devices() -> Result<Vec<BluetoothDevice>> {
    scan_devices()
}

/// Connect to a device
#[cfg(target_os = "linux")]
pub fn connect_device(mac_address: &str) -> Result<()> {
    Command::new("bluetoothctl")
        .args(["connect", mac_address])
        .output()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn connect_device(mac_address: &str) -> Result<()> {
    anyhow::bail!("Connecting on this platform requires GUI interaction")
}

/// Disconnect a device
#[cfg(target_os = "linux")]
pub fn disconnect_device(mac_address: &str) -> Result<()> {
    Command::new("bluetoothctl")
        .args(["disconnect", mac_address])
        .output()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn disconnect_device(mac_address: &str) -> Result<()> {
    anyhow::bail!("Disconnecting on this platform requires GUI interaction")
}

/// Set discoverable mode
#[cfg(target_os = "linux")]
pub fn set_discoverable(discoverable: bool, timeout: Option<u32>) -> Result<()> {
    if discoverable {
        if let Some(t) = timeout {
            Command::new("bluetoothctl")
                .args(["discoverable-timeout", &t.to_string()])
                .output()?;
        }
        Command::new("bluetoothctl")
            .args(["discoverable", "on"])
            .output()?;
    } else {
        Command::new("bluetoothctl")
            .args(["discoverable", "off"])
            .output()?;
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn set_discoverable(discoverable: bool, timeout: Option<u32>) -> Result<()> {
    let _ = (discoverable, timeout);
    anyhow::bail!("Discoverable mode on this platform requires GUI interaction")
}

/// Set device name
#[cfg(target_os = "linux")]
pub fn set_device_name(name: &str) -> Result<()> {
    Command::new("bluetoothctl").args(["name", name]).output()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn set_device_name(name: &str) -> Result<()> {
    let _ = name;
    anyhow::bail!("Setting device name on this platform requires system preferences")
}

/// Get list of connected devices
pub fn get_connected_devices() -> Result<Vec<BluetoothDevice>> {
    let all_devices = list_paired_devices()?;
    Ok(all_devices.into_iter().filter(|d| d.connected).collect())
}

/// Send file via Bluetooth (OBEX)
#[cfg(target_os = "linux")]
pub fn send_file(mac_address: &str, file_path: &str) -> Result<()> {
    Command::new("obexftp")
        .args(["-b", mac_address, "-p", file_path])
        .output()
        .map_err(|_| anyhow::anyhow!("obexftp not installed"))?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn send_file(mac_address: &str, file_path: &str) -> Result<()> {
    let _ = (mac_address, file_path);
    anyhow::bail!("File transfer on this platform requires GUI interaction")
}

// Platform-agnostic fallbacks
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn bluetooth_on() -> Result<()> {
    anyhow::bail!("Bluetooth not implemented on this platform")
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn bluetooth_off() -> Result<()> {
    anyhow::bail!("Bluetooth not implemented on this platform")
}
