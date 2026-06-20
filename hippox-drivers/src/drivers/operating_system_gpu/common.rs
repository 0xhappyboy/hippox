//! Shared utilities for GPU operations

use serde::{Deserialize, Serialize};

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub driver_version: String,
    pub total_memory_mb: u64,
    pub memory_type: String,
    pub pcie_speed: String,
    pub pcie_width: u8,
    pub bios_version: Option<String>,
    pub serial_number: Option<String>,
}

/// GPU memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub usage_percent: f32,
}

/// GPU process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_used_mb: u64,
    pub gpu_usage_percent: f32,
}

/// GPU clock information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuClockInfo {
    pub core_mhz: u64,
    pub memory_mhz: u64,
    pub boost_mhz: Option<u64>,
}

/// GPU video engine info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuVideoEngineInfo {
    pub decode_usage_percent: f32,
    pub encode_usage_percent: f32,
}