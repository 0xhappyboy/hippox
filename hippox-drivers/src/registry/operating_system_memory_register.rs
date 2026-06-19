//! Operating system memory drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemMemory;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_memory", feature = "all"))]
    {
        use crate::drivers::operating_system_memory::*;

        map.insert("memory_read".to_string(), Arc::new(MemoryReadDriver));
        map.insert("memory_scan".to_string(), Arc::new(MemoryScanDriver));
        map.insert("module_base".to_string(), Arc::new(ModuleBaseDriver));
    }
}
