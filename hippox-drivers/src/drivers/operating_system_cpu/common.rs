//! Shared utilities for CPU operations

use serde::{Deserialize, Serialize};

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub vendor: String,
    pub brand: String,
    pub model_name: String,
    pub architecture: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub max_frequency_mhz: u64,
    pub min_frequency_mhz: u64,
    pub is_hypervisor: bool,
}

/// CPU core information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub core_id: usize,
    pub usage_percent: f32,
    pub frequency_mhz: Option<u64>,
}

/// CPU load average
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLoadAverage {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
}

/// CPU cache information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCacheInfo {
    pub level: u8,
    pub size_kb: u64,
    pub line_size_bytes: u64,
    pub associativity: Option<u64>,
    pub cache_type: String,
}

/// CPU feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuFeatures {
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    pub aes_ni: bool,
    pub rdrand: bool,
    pub rdseed: bool,
    pub hypervisor: bool,
    pub vmx: bool,
    pub svm: bool,
    pub smx: bool,
    pub intel_pt: bool,
    pub vaes: bool,
    pub vpclmulqdq: bool,
}
