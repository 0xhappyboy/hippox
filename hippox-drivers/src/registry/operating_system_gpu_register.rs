//! Operating system GPU drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemGpu;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_gpu", feature = "all"))]
    {
        use crate::drivers::operating_system_gpu::*;

        map.insert("gpu_info".to_string(), Arc::new(GpuInfoDriver));
        map.insert("gpu_usage".to_string(), Arc::new(GpuUsageDriver));
        map.insert("gpu_memory".to_string(), Arc::new(GpuMemoryDriver));
        map.insert("gpu_temperature".to_string(), Arc::new(GpuTemperatureDriver));
        map.insert("gpu_fan_speed".to_string(), Arc::new(GpuFanSpeedDriver));
        map.insert("gpu_power".to_string(), Arc::new(GpuPowerDriver));
        map.insert("gpu_processes".to_string(), Arc::new(GpuProcessesDriver));
        map.insert("gpu_clock".to_string(), Arc::new(GpuClockDriver));
        map.insert("gpu_video_decode".to_string(), Arc::new(GpuVideoDecodeDriver));
        map.insert("gpu_video_encode".to_string(), Arc::new(GpuVideoEncodeDriver));
    }
}