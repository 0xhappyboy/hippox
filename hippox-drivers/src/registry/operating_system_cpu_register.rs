//! Operating system CPU drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemCpu;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_cpu", feature = "all"))]
    {
        use crate::drivers::operating_system_cpu::*;

        map.insert("cpu_info".to_string(), Arc::new(CpuInfoDriver));
        map.insert("cpu_usage".to_string(), Arc::new(CpuUsageDriver));
        map.insert("cpu_load".to_string(), Arc::new(CpuLoadDriver));
        map.insert("cpu_cache".to_string(), Arc::new(CpuCacheDriver));
        map.insert("cpu_affinity".to_string(), Arc::new(CpuAffinityDriver));
        map.insert("cpu_frequency".to_string(), Arc::new(CpuFrequencyDriver));
        map.insert("cpu_features".to_string(), Arc::new(CpuFeaturesDriver));
        map.insert(
            "cpu_temperature".to_string(),
            Arc::new(CpuTemperatureDriver),
        );
    }
}
