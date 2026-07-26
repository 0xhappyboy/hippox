//! Shared utilities for Bluetooth control across platforms
//!
//! This module provides cross-platform utilities for Bluetooth management,
//! including device discovery, pairing, connection control, and adapter status.
//! It abstracts platform-specific implementations behind a unified API.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};
/// Bluetooth device information
///
/// Contains comprehensive information about a Bluetooth device including
/// its name, MAC address, connection status, and signal strength.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    /// Human-readable device name
    pub name: String,
    /// MAC address in format XX:XX:XX:XX:XX:XX
    pub mac_address: String,
    /// Device type (e.g., Audio, HID, Phone)
    pub device_type: String,
    /// Whether the device is paired with this system
    pub paired: bool,
    /// Whether the device is currently connected
    pub connected: bool,
    /// Signal strength in dBm (RSSI), if available
    pub rssi: Option<i32>,
    /// Battery level as percentage (0-100), if available
    pub battery_level: Option<u8>,
}
/// Bluetooth adapter status
///
/// Contains the current state of the Bluetooth adapter including
/// power state, discoverability, and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothAdapterStatus {
    /// Whether the Bluetooth adapter is powered on
    pub powered_on: bool,
    /// Whether the adapter is in discoverable mode
    pub discoverable: bool,
    /// Whether the adapter is pairable
    pub pairable: bool,
    /// The adapter's display name
    pub name: String,
    /// The adapter's MAC address
    pub mac_address: String,
    /// How long the adapter stays discoverable (seconds), 0 = unlimited
    pub discoverable_timeout: u32,
}
/// Bluetooth service/characteristic info for BLE GATT
///
/// Represents a BLE service with its UUID, name, and associated characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothService {
    /// Service UUID
    pub uuid: String,
    /// Human-readable service name
    pub name: String,
    /// Whether this is a primary service
    pub primary: bool,
    /// List of characteristics belonging to this service
    pub characteristics: Vec<BluetoothCharacteristic>,
}
/// Bluetooth characteristic info for BLE GATT
///
/// Represents a BLE characteristic with its UUID, properties, and value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothCharacteristic {
    /// Characteristic UUID
    pub uuid: String,
    /// Human-readable characteristic name
    pub name: String,
    /// List of properties (read, write, notify, etc.)
    pub properties: Vec<String>,
    /// Current value of the characteristic, if available
    pub value: Option<Vec<u8>>,
}
/// Get Bluetooth adapter status (Windows)
///
/// Retrieves the current status of the Bluetooth adapter on Windows
/// using PowerShell commands.
#[cfg(target_os = "windows")]
pub fn get_adapter_status() -> DriverResult<BluetoothAdapterStatus> {
    debug!("Getting adapter status on Windows");
    let output = Command::new("powershell")
        .args(["-Command", "Get-PnpDevice -Class Bluetooth | Select-Object Status, FriendlyName"])
        .output()
        .map_err(|e| {
            let err_msg = format!("Failed to execute PowerShell: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut powered_on = false;
    let mut name = "Unknown".to_string();
    for line in stdout.lines() {
        if line.contains("OK") || line.contains("正在运行") {
            powered_on = true;
            debug!("Bluetooth adapter is powered on");
        }
        if line.contains("Bluetooth") && !line.contains("Status") {
            name = line.trim().to_string();
            debug!("Adapter name: {}", name);
        }
    }
    let mac_address = get_mac_address()?;
    debug!("Adapter MAC address: {}", mac_address);
    return Ok(BluetoothAdapterStatus { powered_on, discoverable: false, pairable: true, name, mac_address, discoverable_timeout: 120 });
}
/// Get Bluetooth adapter status (Linux)
///
/// Retrieves the current status of the Bluetooth adapter on Linux
/// using bluetoothctl commands.
#[cfg(target_os = "linux")]
pub fn get_adapter_status() -> DriverResult<BluetoothAdapterStatus> {
    debug!("Getting adapter status on Linux");
    let output = Command::new("bluetoothctl").args(["show"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
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
            debug!("Adapter is powered on");
        }
        if line.contains("Discoverable:") && line.contains("yes") {
            discoverable = true;
            debug!("Adapter is discoverable");
        }
        if line.contains("Name:") {
            if let Some(n) = line.split(':').nth(1) {
                name = n.trim().to_string();
                debug!("Adapter name: {}", name);
            }
        }
        if line.contains("Address:") {
            if let Some(addr) = line.split(':').nth(1) {
                mac_address = addr.trim().to_string();
                debug!("Adapter MAC address: {}", mac_address);
            }
        }
        if line.contains("DiscoverableTimeout:") {
            if let Some(t) = line.split(':').nth(1) {
                discoverable_timeout = t.trim().parse().unwrap_or(0);
                debug!("Discoverable timeout: {}s", discoverable_timeout);
            }
        }
    }
    return Ok(BluetoothAdapterStatus { powered_on, discoverable, pairable, name, mac_address, discoverable_timeout });
}
/// Get Bluetooth adapter status (macOS)
///
/// Retrieves the current status of the Bluetooth adapter on macOS
/// using system_profiler.
#[cfg(target_os = "macos")]
pub fn get_adapter_status() -> DriverResult<BluetoothAdapterStatus> {
    debug!("Getting adapter status on macOS");
    let output = Command::new("system_profiler").args(["SPBluetoothDataType"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute system_profiler: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut powered_on = false;
    let mut name = "Unknown".to_string();
    let mut mac_address = "Unknown".to_string();
    for line in stdout.lines() {
        if line.contains("Bluetooth Power: On") {
            powered_on = true;
            debug!("Adapter is powered on");
        }
        if line.contains("Name:") {
            if let Some(n) = line.split(':').nth(1) {
                name = n.trim().to_string();
                debug!("Adapter name: {}", name);
            }
        }
        if line.contains("Address:") {
            if let Some(addr) = line.split(':').nth(1) {
                mac_address = addr.trim().to_string();
                debug!("Adapter MAC address: {}", mac_address);
            }
        }
    }
    return Ok(BluetoothAdapterStatus { powered_on, discoverable: true, pairable: true, name, mac_address, discoverable_timeout: 120 });
}
/// Get Bluetooth MAC address
///
/// Retrieves the MAC address of the local Bluetooth adapter.
/// This function works across all supported platforms.
pub fn get_mac_address() -> DriverResult<String> {
    debug!("Getting Bluetooth MAC address");
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("bluetoothctl").args(["show"]).output().map_err(|e| {
            let err_msg = format!("Failed to execute bluetoothctl: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Address:") {
                if let Some(addr) = line.split(':').nth(1) {
                    let mac = addr.trim().to_string();
                    debug!("MAC address found: {}", mac);
                    return Ok(mac);
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("getmac").output().map_err(|e| {
            let err_msg = format!("Failed to execute getmac: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Bluetooth") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 1 {
                    let mac = parts[0].to_string();
                    debug!("MAC address found: {}", mac);
                    return Ok(mac);
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("system_profiler").args(["SPBluetoothDataType"]).output().map_err(|e| {
            let err_msg = format!("Failed to execute system_profiler: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Address:") {
                if let Some(addr) = line.split(':').nth(1) {
                    let mac = addr.trim().to_string();
                    debug!("MAC address found: {}", mac);
                    return Ok(mac);
                }
            }
        }
    }
    warn!("MAC address not found, returning 'Unknown'");
    return Ok("Unknown".to_string());
}
/// Turn Bluetooth on
///
/// Enables the Bluetooth adapter. On macOS, this requires the `blueutil` tool.
#[cfg(target_os = "linux")]
pub fn bluetooth_on() -> DriverResult<()> {
    debug!("Turning Bluetooth on (Linux)");
    Command::new("bluetoothctl").args(["power", "on"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Bluetooth turned on");
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn bluetooth_on() -> DriverResult<()> {
    debug!("Turning Bluetooth on (Windows)");
    Command::new("powershell").args(["-Command", "Enable-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute PowerShell: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Bluetooth turned on");
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn bluetooth_on() -> DriverResult<()> {
    debug!("Turning Bluetooth on (macOS)");
    let output = Command::new("blueutil").args(["--power", "1"]).output();
    if output.is_err() {
        let err_msg = "blueutil not installed. Run: brew install blueutil".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
    info!("Bluetooth turned on");
    return Ok(());
}
/// Turn Bluetooth off
///
/// Disables the Bluetooth adapter. On macOS, this requires the `blueutil` tool.
#[cfg(target_os = "linux")]
pub fn bluetooth_off() -> DriverResult<()> {
    debug!("Turning Bluetooth off (Linux)");
    Command::new("bluetoothctl").args(["power", "off"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Bluetooth turned off");
    return Ok(());
}
#[cfg(target_os = "windows")]
pub fn bluetooth_off() -> DriverResult<()> {
    debug!("Turning Bluetooth off (Windows)");
    Command::new("powershell").args(["-Command", "Disable-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute PowerShell: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Bluetooth turned off");
    return Ok(());
}
#[cfg(target_os = "macos")]
pub fn bluetooth_off() -> DriverResult<()> {
    debug!("Turning Bluetooth off (macOS)");
    let output = Command::new("blueutil").args(["--power", "0"]).output();
    if output.is_err() {
        let err_msg = "blueutil not installed. Run: brew install blueutil".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
    info!("Bluetooth turned off");
    return Ok(());
}
/// Scan for Bluetooth devices
///
/// Performs a scan for nearby Bluetooth devices and returns a list
/// of discovered devices with their information.
#[cfg(target_os = "linux")]
pub fn scan_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Starting Bluetooth scan (Linux)");
    // Start scan
    debug!("Starting discovery scan");
    let _ = Command::new("bluetoothctl").args(["scan", "on"]).output();
    // Wait for scan results
    debug!("Waiting 5 seconds for scan results");
    std::thread::sleep(std::time::Duration::from_secs(5));
    let output = Command::new("bluetoothctl").args(["devices"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in stdout.lines() {
        if line.starts_with("Device") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let device = BluetoothDevice {
                    mac_address: parts[1].to_string(),
                    name: parts[2..].join(" "),
                    device_type: "Unknown".to_string(),
                    paired: false,
                    connected: false,
                    rssi: None,
                    battery_level: None,
                };
                debug!("Found device: {} ({})", device.name, device.mac_address);
                devices.push(device);
            }
        }
    }
    // Stop scan
    debug!("Stopping discovery scan");
    let _ = Command::new("bluetoothctl").args(["scan", "off"]).output();
    info!("Scan complete, found {} devices", devices.len());
    return Ok(devices);
}
#[cfg(target_os = "windows")]
pub fn scan_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Starting Bluetooth scan (Windows)");
    let output = Command::new("powershell")
        .args(["-Command", "Get-PnpDevice -Class Bluetooth | Select-Object FriendlyName, InstanceId"])
        .output()
        .map_err(|e| {
            let err_msg = format!("Failed to execute PowerShell: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 && !line.contains("Bluetooth") {
            let device = BluetoothDevice {
                name: parts[0].to_string(),
                mac_address: "Unknown".to_string(),
                device_type: "Unknown".to_string(),
                paired: false,
                connected: false,
                rssi: None,
                battery_level: None,
            };
            debug!("Found device: {}", device.name);
            devices.push(device);
        }
    }
    info!("Scan complete, found {} devices", devices.len());
    return Ok(devices);
}
#[cfg(target_os = "macos")]
pub fn scan_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Starting Bluetooth scan (macOS)");
    let output = Command::new("system_profiler").args(["SPBluetoothDataType"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute system_profiler: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    let mut current_device: Option<BluetoothDevice> = None;
    for line in stdout.lines() {
        if line.contains("Fully Qualified") {
            if let Some(device) = current_device.take() {
                devices.push(device);
            }
            if let Some(name) = line.split(':').nth(1) {
                debug!("Found device: {}", name.trim());
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
                    debug!("Device MAC: {}", device.mac_address);
                }
            }
            if line.contains("Connected: Yes") {
                device.connected = true;
                debug!("Device is connected");
            }
            if line.contains("Paired: Yes") {
                device.paired = true;
                debug!("Device is paired");
            }
        }
    }
    if let Some(device) = current_device {
        devices.push(device);
    }
    info!("Scan complete, found {} devices", devices.len());
    return Ok(devices);
}
/// Pair with a device
///
/// Initiates pairing with a discovered Bluetooth device.
/// On non-Linux platforms, this may require GUI interaction.
#[cfg(target_os = "linux")]
pub fn pair_device(mac_address: &str) -> DriverResult<()> {
    debug!("Pairing with device: {}", mac_address);
    Command::new("bluetoothctl").args(["pair", mac_address]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Paired with device: {}", mac_address);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn pair_device(mac_address: &str) -> DriverResult<()> {
    debug!("Pairing not supported on this platform: {}", mac_address);
    let err_msg = "Pairing on this platform requires GUI interaction".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Unpair a device
///
/// Removes a device from the paired devices list.
/// On non-Linux platforms, this may require GUI interaction.
#[cfg(target_os = "linux")]
pub fn unpair_device(mac_address: &str) -> DriverResult<()> {
    debug!("Unpairing device: {}", mac_address);
    Command::new("bluetoothctl").args(["remove", mac_address]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Unpaired device: {}", mac_address);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn unpair_device(mac_address: &str) -> DriverResult<()> {
    debug!("Unpairing not supported on this platform: {}", mac_address);
    let err_msg = "Unpairing on this platform requires GUI interaction".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// List paired devices
///
/// Retrieves the list of all devices that are paired with this system.
#[cfg(target_os = "linux")]
pub fn list_paired_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Listing paired devices (Linux)");
    let output = Command::new("bluetoothctl").args(["paired-devices"]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in stdout.lines() {
        if line.starts_with("Device") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let device = BluetoothDevice {
                    mac_address: parts[1].to_string(),
                    name: parts[2..].join(" "),
                    device_type: "Unknown".to_string(),
                    paired: true,
                    connected: false,
                    rssi: None,
                    battery_level: None,
                };
                debug!("Paired device: {} ({})", device.name, device.mac_address);
                devices.push(device);
            }
        }
    }
    info!("Found {} paired devices", devices.len());
    return Ok(devices);
}
#[cfg(target_os = "windows")]
pub fn list_paired_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Listing paired devices (Windows)");
    let output = Command::new("powershell")
        .args(["-Command", "Get-PnpDevice -Class Bluetooth | Where-Object {$_.FriendlyName -notlike '*Radio*'} | Select-Object FriendlyName, Status"])
        .output()
        .map_err(|e| {
            let err_msg = format!("Failed to execute PowerShell: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            let connected = parts.contains(&"OK");
            let device = BluetoothDevice {
                name: parts[0].to_string(),
                mac_address: "Unknown".to_string(),
                device_type: "Unknown".to_string(),
                paired: true,
                connected,
                rssi: None,
                battery_level: None,
            };
            debug!("Paired device: {} (connected: {})", device.name, connected);
            devices.push(device);
        }
    }
    info!("Found {} paired devices", devices.len());
    return Ok(devices);
}
#[cfg(target_os = "macos")]
pub fn list_paired_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Listing paired devices (macOS) - using scan_devices");
    return scan_devices();
}
/// Connect to a device
///
/// Establishes a connection to a paired Bluetooth device.
/// On non-Linux platforms, this may require GUI interaction.
#[cfg(target_os = "linux")]
pub fn connect_device(mac_address: &str) -> DriverResult<()> {
    debug!("Connecting to device: {}", mac_address);
    Command::new("bluetoothctl").args(["connect", mac_address]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Connected to device: {}", mac_address);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn connect_device(mac_address: &str) -> DriverResult<()> {
    debug!("Connecting not supported on this platform: {}", mac_address);
    let err_msg = "Connecting on this platform requires GUI interaction".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Disconnect a device
///
/// Disconnects a connected Bluetooth device while keeping it paired.
/// On non-Linux platforms, this may require GUI interaction.
#[cfg(target_os = "linux")]
pub fn disconnect_device(mac_address: &str) -> DriverResult<()> {
    debug!("Disconnecting device: {}", mac_address);
    Command::new("bluetoothctl").args(["disconnect", mac_address]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Disconnected from device: {}", mac_address);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn disconnect_device(mac_address: &str) -> DriverResult<()> {
    debug!("Disconnecting not supported on this platform: {}", mac_address);
    let err_msg = "Disconnecting on this platform requires GUI interaction".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Set discoverable mode
///
/// Enables or disables discoverable mode for the Bluetooth adapter.
/// On non-Linux platforms, this may require GUI interaction.
#[cfg(target_os = "linux")]
pub fn set_discoverable(discoverable: bool, timeout: Option<u32>) -> DriverResult<()> {
    debug!("Setting discoverable mode: enabled={}, timeout={:?}", discoverable, timeout);
    if discoverable {
        if let Some(t) = timeout {
            debug!("Setting discoverable timeout to {}s", t);
            Command::new("bluetoothctl").args(["discoverable-timeout", &t.to_string()]).output().map_err(|e| {
                let err_msg = format!("Failed to set discoverable timeout: {}", e);
                warn!("{}", err_msg);
                return DriverError::execution(err_msg);
            })?;
        }
        debug!("Turning discoverable on");
        Command::new("bluetoothctl").args(["discoverable", "on"]).output().map_err(|e| {
            let err_msg = format!("Failed to enable discoverable: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    } else {
        debug!("Turning discoverable off");
        Command::new("bluetoothctl").args(["discoverable", "off"]).output().map_err(|e| {
            let err_msg = format!("Failed to disable discoverable: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
    }
    info!("Discoverable mode set: enabled={}", discoverable);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn set_discoverable(discoverable: bool, timeout: Option<u32>) -> DriverResult<()> {
    let _ = (discoverable, timeout);
    debug!("Setting discoverable not supported on this platform");
    let err_msg = "Discoverable mode on this platform requires GUI interaction".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Set device name
///
/// Changes the display name of the Bluetooth adapter.
/// On non-Linux platforms, this may require system preferences.
#[cfg(target_os = "linux")]
pub fn set_device_name(name: &str) -> DriverResult<()> {
    debug!("Setting device name to: {}", name);
    Command::new("bluetoothctl").args(["name", name]).output().map_err(|e| {
        let err_msg = format!("Failed to execute bluetoothctl: {}", e);
        warn!("{}", err_msg);
        return DriverError::execution(err_msg);
    })?;
    info!("Device name set to: {}", name);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn set_device_name(name: &str) -> DriverResult<()> {
    let _ = name;
    debug!("Setting device name not supported on this platform");
    let err_msg = "Setting device name on this platform requires system preferences".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
/// Get list of connected devices
///
/// Retrieves the list of currently connected Bluetooth devices.
pub fn get_connected_devices() -> DriverResult<Vec<BluetoothDevice>> {
    debug!("Getting connected devices");
    let all_devices = list_paired_devices()?;
    let connected: Vec<BluetoothDevice> = all_devices.into_iter().filter(|d| d.connected).collect();
    info!("Found {} connected devices", connected.len());
    return Ok(connected);
}
/// Send file via Bluetooth (OBEX)
///
/// Sends a file to a Bluetooth device using OBEX Object Push.
/// On Linux, this requires the `obexftp` tool to be installed.
#[cfg(target_os = "linux")]
pub fn send_file(mac_address: &str, file_path: &str) -> DriverResult<()> {
    debug!("Sending file {} to {}", file_path, mac_address);
    let output = Command::new("obexftp").args(["-b", mac_address, "-p", file_path]).output();
    if output.is_err() {
        let err_msg = "obexftp not installed. Please install obexftp package".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::execution(err_msg));
    }
    info!("File sent successfully to {}", mac_address);
    return Ok(());
}
#[cfg(not(target_os = "linux"))]
pub fn send_file(mac_address: &str, file_path: &str) -> DriverResult<()> {
    let _ = (mac_address, file_path);
    debug!("File transfer not supported on this platform");
    let err_msg = "File transfer on this platform requires GUI interaction".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
// Platform-agnostic fallbacks
//
// These functions provide fallback implementations for platforms
// that don't have native Bluetooth support.
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn bluetooth_on() -> DriverResult<()> {
    debug!("Bluetooth not implemented on this platform");
    let err_msg = "Bluetooth not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn bluetooth_off() -> DriverResult<()> {
    debug!("Bluetooth not implemented on this platform");
    let err_msg = "Bluetooth not implemented on this platform".to_string();
    warn!("{}", err_msg);
    return Err(DriverError::execution(err_msg));
}
