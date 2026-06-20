//! Operating system disk drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemDisk;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_disk", feature = "all"))]
    {
        use crate::drivers::operating_system_disk::*;

        map.insert("disk_info".to_string(), Arc::new(DiskInfoDriver));
        map.insert("disk_usage".to_string(), Arc::new(DiskUsageDriver));
        map.insert("disk_partitions".to_string(), Arc::new(DiskPartitionsDriver));
        map.insert("disk_io".to_string(), Arc::new(DiskIoDriver));
        map.insert("disk_smart".to_string(), Arc::new(DiskSmartDriver));
        map.insert("disk_iops".to_string(), Arc::new(DiskIopsDriver));
        map.insert("disk_queue".to_string(), Arc::new(DiskQueueDriver));
        map.insert("disk_encryption".to_string(), Arc::new(DiskEncryptionDriver));
        map.insert("disk_trim".to_string(), Arc::new(DiskTrimDriver));
    }
}