//! Shared utilities for GPU operations
//!
//! This module defines common data structures used across all GPU drivers.

use serde::{Deserialize, Serialize};

/// GPU information structure containing detailed hardware specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU model name
    pub name: String,
    /// GPU vendor name
    pub vendor: String,
    /// Driver version string
    pub driver_version: String,
    /// Total video memory in MB
    pub total_memory_mb: u64,
    /// Memory type (e.g., GDDR6, GDDR6X, HBM2)
    pub memory_type: String,
    /// PCIe link speed (e.g., "16 GT/s")
    pub pcie_speed: String,
    /// PCIe link width (e.g., 16 for x16)
    pub pcie_width: u8,
    /// BIOS version (optional)
    pub bios_version: Option<String>,
    /// Serial number (optional)
    pub serial_number: Option<String>,
}

/// GPU memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemoryInfo {
    /// Total memory in MB
    pub total_mb: u64,
    /// Used memory in MB
    pub used_mb: u64,
    /// Free memory in MB
    pub free_mb: u64,
    /// Memory usage percentage
    pub usage_percent: f32,
}

/// GPU process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Memory used by this process in MB
    pub memory_used_mb: u64,
    /// GPU usage percentage for this process
    pub gpu_usage_percent: f32,
}

/// GPU clock speed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuClockInfo {
    /// Core clock speed in MHz
    pub core_mhz: u64,
    /// Memory clock speed in MHz
    pub memory_mhz: u64,
    /// Boost clock speed in MHz (optional)
    pub boost_mhz: Option<u64>,
}

/// GPU video engine usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuVideoEngineInfo {
    /// Decode engine usage percentage
    pub decode_usage_percent: f32,
    /// Encode engine usage percentage
    pub encode_usage_percent: f32,
}
