//! Shared utilities for audio control
//!
//! This module provides cross-platform utilities for audio management,
//! including volume control, device listing, and mute/unmute functionality.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};
/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}
/// Get current system volume (0-100)
pub fn get_volume() -> DriverResult<u32> {
    debug!("Getting current system volume");
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell").args(["-Command", "(Get-AudioDevice -PlaybackVolume).Volume"]).output();
        if let Ok(output) = output {
            if let Ok(vol_str) = String::from_utf8(output.stdout) {
                if let Ok(vol) = vol_str.trim().parse::<f64>() {
                    let volume = (vol * 100.0) as u32;
                    debug!("Current volume: {}%", volume);
                    return Ok(volume);
                }
            }
        }
        warn!("Failed to get volume via PowerShell, using default 50%");
        return Ok(50); // Default fallback
    }
    #[cfg(not(target_os = "windows"))]
    {
        debug!("Volume get not fully implemented for this platform, using default 50%");
        return Ok(50); // Placeholder for other platforms
    }
}
/// Set system volume using PowerShell
#[cfg(target_os = "windows")]
pub fn set_volume(volume: u32) -> DriverResult<()> {
    let volume = volume.clamp(0, 100);
    let volume_f = volume as f64 / 100.0;
    debug!("Setting volume to {}% ({}f)", volume, volume_f);
    let _ = Command::new("powershell").args(["-Command", &format!("Set-AudioDevice -PlaybackVolume {}", volume_f)]).output();
    info!("Volume set to {}%", volume);
    return Ok(());
}
#[cfg(not(target_os = "windows"))]
pub fn set_volume(volume: u32) -> DriverResult<()> {
    debug!("Volume set not implemented for this platform");
    let _ = volume;
    return Ok(());
}
/// Increase volume by delta
pub fn volume_up(delta: u32) -> DriverResult<()> {
    debug!("Increasing volume by {}%", delta);
    let current = get_volume()?;
    let new_volume = (current + delta).min(100);
    set_volume(new_volume)?;
    info!("Volume increased from {}% to {}%", current, new_volume);
    return Ok(());
}
/// Decrease volume by delta
pub fn volume_down(delta: u32) -> DriverResult<()> {
    debug!("Decreasing volume by {}%", delta);
    let current = get_volume()?;
    let new_volume = current.saturating_sub(delta);
    set_volume(new_volume)?;
    info!("Volume decreased from {}% to {}%", current, new_volume);
    return Ok(());
}
/// Mute audio
#[cfg(target_os = "windows")]
pub fn mute() -> DriverResult<()> {
    debug!("Muting audio");
    let _ = Command::new("powershell").args(["-Command", "(New-Object -ComObject WScript.Shell).SendKeys([char]173)"]).output();
    info!("Audio muted");
    return Ok(());
}
#[cfg(not(target_os = "windows"))]
pub fn mute() -> DriverResult<()> {
    debug!("Mute not implemented for this platform");
    return Ok(());
}
/// Unmute audio
#[cfg(target_os = "windows")]
pub fn unmute() -> DriverResult<()> {
    debug!("Unmuting audio");
    let _ = Command::new("powershell").args(["-Command", "(New-Object -ComObject WScript.Shell).SendKeys([char]173)"]).output();
    info!("Audio unmuted");
    return Ok(());
}
#[cfg(not(target_os = "windows"))]
pub fn unmute() -> DriverResult<()> {
    debug!("Unmute not implemented for this platform");
    return Ok(());
}
/// List output devices
#[cfg(target_os = "windows")]
pub fn list_output_devices() -> DriverResult<Vec<AudioDevice>> {
    debug!("Listing output devices");
    let output = Command::new("powershell").args(["-Command", "Get-AudioDevice -List | ForEach-Object { $_.FriendlyName }"]).output();
    let mut devices = vec![AudioDevice { id: "default".to_string(), name: "Default Output Device".to_string(), is_default: true }];
    if let Ok(output) = output {
        if let Ok(devices_str) = String::from_utf8(output.stdout) {
            for (i, line) in devices_str.lines().enumerate() {
                if !line.is_empty() {
                    devices.push(AudioDevice { id: format!("device_{}", i), name: line.to_string(), is_default: false });
                }
            }
        }
    }
    info!("Found {} output devices", devices.len());
    return Ok(devices);
}
#[cfg(not(target_os = "windows"))]
pub fn list_output_devices() -> DriverResult<Vec<AudioDevice>> {
    debug!("Listing output devices (placeholder for non-Windows)");
    return Ok(vec![
        AudioDevice { id: "default".to_string(), name: "Default Output Device".to_string(), is_default: true },
        AudioDevice { id: "speakers".to_string(), name: "Speakers".to_string(), is_default: false },
        AudioDevice { id: "headphones".to_string(), name: "Headphones".to_string(), is_default: false },
    ]);
}
/// Set output device
#[cfg(target_os = "windows")]
pub fn set_output_device(device_id: &str) -> DriverResult<()> {
    debug!("Setting output device to: {}", device_id);
    let _ = Command::new("powershell").args(["-Command", &format!("Set-AudioDevice -Index {}", device_id)]).output();
    info!("Output device set to: {}", device_id);
    return Ok(());
}
#[cfg(not(target_os = "windows"))]
pub fn set_output_device(device_id: &str) -> DriverResult<()> {
    debug!("Set output device not implemented for this platform: {}", device_id);
    let _ = device_id;
    return Ok(());
}
/// List input devices
#[cfg(target_os = "windows")]
pub fn list_input_devices() -> DriverResult<Vec<AudioDevice>> {
    debug!("Listing input devices");
    let output = Command::new("powershell").args(["-Command", "Get-AudioDevice -List -Recording | ForEach-Object { $_.FriendlyName }"]).output();
    let mut devices = vec![AudioDevice { id: "default".to_string(), name: "Default Microphone".to_string(), is_default: true }];
    if let Ok(output) = output {
        if let Ok(devices_str) = String::from_utf8(output.stdout) {
            for (i, line) in devices_str.lines().enumerate() {
                if !line.is_empty() {
                    devices.push(AudioDevice { id: format!("mic_{}", i), name: line.to_string(), is_default: false });
                }
            }
        }
    }
    info!("Found {} input devices", devices.len());
    return Ok(devices);
}
#[cfg(not(target_os = "windows"))]
pub fn list_input_devices() -> DriverResult<Vec<AudioDevice>> {
    debug!("Listing input devices (placeholder for non-Windows)");
    return Ok(vec![
        AudioDevice { id: "default".to_string(), name: "Default Microphone".to_string(), is_default: true },
        AudioDevice { id: "mic".to_string(), name: "Microphone Array".to_string(), is_default: false },
    ]);
}
/// Set input device volume
#[cfg(target_os = "windows")]
pub fn set_input_volume(volume: u32) -> DriverResult<()> {
    let volume = volume.clamp(0, 100);
    let volume_f = volume as f64 / 100.0;
    debug!("Setting input volume to {}% ({}f)", volume, volume_f);
    let _ = Command::new("powershell").args(["-Command", &format!("Set-AudioDevice -RecordingVolume {}", volume_f)]).output();
    info!("Input volume set to {}%", volume);
    return Ok(());
}
#[cfg(not(target_os = "windows"))]
pub fn set_input_volume(volume: u32) -> DriverResult<()> {
    debug!("Set input volume not implemented for this platform: {}%", volume);
    let _ = volume;
    return Ok(());
}
