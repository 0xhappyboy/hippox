//! Shared utilities for disk operations

use serde::{Deserialize, Serialize};

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub model: String,
    pub serial_number: String,
    pub interface_type: String,
    pub transfer_mode: String,
    pub size_gb: u64,
    pub is_ssd: bool,
    pub is_removable: bool,
}

/// Disk partition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPartition {
    pub device: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_size_gb: u64,
    pub used_size_gb: u64,
    pub free_size_gb: u64,
    pub usage_percent: f32,
    pub is_encrypted: bool,
}

/// Disk I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoStats {
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub read_ops_per_sec: u64,
    pub write_ops_per_sec: u64,
    pub avg_read_latency_ms: f32,
    pub avg_write_latency_ms: f32,
}

/// Disk SMART information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSmartInfo {
    pub health_percent: f32,
    pub temperature_celsius: f32,
    pub power_on_hours: u64,
    pub wear_level: Option<f32>,
    pub has_error: bool,
    pub error_message: Option<String>,
}

/// Disk IOPS information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIopsInfo {
    pub read_iops: u64,
    pub write_iops: u64,
    pub total_iops: u64,
}

/// Disk queue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskQueueInfo {
    pub queue_depth: u64,
    pub avg_wait_time_ms: f32,
    pub max_wait_time_ms: f32,
}
