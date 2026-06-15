// audio_control/shared.rs
//! Shared utilities for audio control

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Get current system volume (0-100)
pub fn get_volume() -> Result<u32> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-Command", "(Get-AudioDevice -PlaybackVolume).Volume"])
            .output();

        if let Ok(output) = output {
            if let Ok(vol_str) = String::from_utf8(output.stdout) {
                if let Ok(vol) = vol_str.trim().parse::<f64>() {
                    return Ok((vol * 100.0) as u32);
                }
            }
        }
        Ok(50) // Default fallback
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(50) // Placeholder for other platforms
    }
}

/// Set system volume using PowerShell
#[cfg(target_os = "windows")]
pub fn set_volume(volume: u32) -> Result<()> {
    let volume = volume.clamp(0, 100);
    let volume_f = volume as f64 / 100.0;

    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!("Set-AudioDevice -PlaybackVolume {}", volume_f),
        ])
        .output();

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_volume(volume: u32) -> Result<()> {
    let _ = volume;
    Ok(())
}

/// Increase volume by delta
pub fn volume_up(delta: u32) -> Result<()> {
    let current = get_volume()?;
    set_volume(current + delta)
}

/// Decrease volume by delta
pub fn volume_down(delta: u32) -> Result<()> {
    let current = get_volume()?;
    set_volume(current.saturating_sub(delta))
}

/// Mute audio
#[cfg(target_os = "windows")]
pub fn mute() -> Result<()> {
    let _ = Command::new("powershell")
        .args([
            "-Command",
            "(New-Object -ComObject WScript.Shell).SendKeys([char]173)",
        ])
        .output();
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn mute() -> Result<()> {
    Ok(())
}

/// Unmute audio
#[cfg(target_os = "windows")]
pub fn unmute() -> Result<()> {
    let _ = Command::new("powershell")
        .args([
            "-Command",
            "(New-Object -ComObject WScript.Shell).SendKeys([char]173)",
        ])
        .output();
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn unmute() -> Result<()> {
    Ok(())
}

/// List output devices
#[cfg(target_os = "windows")]
pub fn list_output_devices() -> Result<Vec<AudioDevice>> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-AudioDevice -List | ForEach-Object { $_.FriendlyName }",
        ])
        .output();

    let mut devices = vec![AudioDevice {
        id: "default".to_string(),
        name: "Default Output Device".to_string(),
        is_default: true,
    }];

    if let Ok(output) = output {
        if let Ok(devices_str) = String::from_utf8(output.stdout) {
            for (i, line) in devices_str.lines().enumerate() {
                if !line.is_empty() {
                    devices.push(AudioDevice {
                        id: format!("device_{}", i),
                        name: line.to_string(),
                        is_default: false,
                    });
                }
            }
        }
    }

    Ok(devices)
}

#[cfg(not(target_os = "windows"))]
pub fn list_output_devices() -> Result<Vec<AudioDevice>> {
    Ok(vec![
        AudioDevice {
            id: "default".to_string(),
            name: "Default Output Device".to_string(),
            is_default: true,
        },
        AudioDevice {
            id: "speakers".to_string(),
            name: "Speakers".to_string(),
            is_default: false,
        },
        AudioDevice {
            id: "headphones".to_string(),
            name: "Headphones".to_string(),
            is_default: false,
        },
    ])
}

/// Set output device
#[cfg(target_os = "windows")]
pub fn set_output_device(device_id: &str) -> Result<()> {
    let _ = Command::new("powershell")
        .args(["-Command", &format!("Set-AudioDevice -Index {}", device_id)])
        .output();
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_output_device(device_id: &str) -> Result<()> {
    let _ = device_id;
    Ok(())
}

/// List input devices
#[cfg(target_os = "windows")]
pub fn list_input_devices() -> Result<Vec<AudioDevice>> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-AudioDevice -List -Recording | ForEach-Object { $_.FriendlyName }",
        ])
        .output();

    let mut devices = vec![AudioDevice {
        id: "default".to_string(),
        name: "Default Microphone".to_string(),
        is_default: true,
    }];

    if let Ok(output) = output {
        if let Ok(devices_str) = String::from_utf8(output.stdout) {
            for (i, line) in devices_str.lines().enumerate() {
                if !line.is_empty() {
                    devices.push(AudioDevice {
                        id: format!("mic_{}", i),
                        name: line.to_string(),
                        is_default: false,
                    });
                }
            }
        }
    }

    Ok(devices)
}

#[cfg(not(target_os = "windows"))]
pub fn list_input_devices() -> Result<Vec<AudioDevice>> {
    Ok(vec![
        AudioDevice {
            id: "default".to_string(),
            name: "Default Microphone".to_string(),
            is_default: true,
        },
        AudioDevice {
            id: "mic".to_string(),
            name: "Microphone Array".to_string(),
            is_default: false,
        },
    ])
}

/// Set input device volume
#[cfg(target_os = "windows")]
pub fn set_input_volume(volume: u32) -> Result<()> {
    let volume = volume.clamp(0, 100);
    let volume_f = volume as f64 / 100.0;

    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!("Set-AudioDevice -RecordingVolume {}", volume_f),
        ])
        .output();
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_input_volume(volume: u32) -> Result<()> {
    let _ = volume;
    Ok(())
}
