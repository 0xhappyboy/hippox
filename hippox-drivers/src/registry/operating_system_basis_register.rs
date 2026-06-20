//! Operating system basis drivers registration
use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;
pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemBasis;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "operating_system_basis", feature = "all"))]
    {
        use crate::drivers::operating_system_basis::*;
        map.insert("clipboard_get".to_string(), Arc::new(ClipboardGetDriver));
        map.insert("clipboard_set".to_string(), Arc::new(ClipboardSetDriver));
        map.insert(
            "clipboard_clear".to_string(),
            Arc::new(ClipboardClearDriver),
        );
        map.insert("system_info".to_string(), Arc::new(SystemInfoDriver));
        map.insert("os_reboot".to_string(), Arc::new(OsRebootDriver));
        map.insert("os_shutdown".to_string(), Arc::new(OsShutdownDriver));
        map.insert("os_sleep".to_string(), Arc::new(OsSleepDriver));
        map.insert("os_lock".to_string(), Arc::new(OsLockDriver));
        map.insert("os_logout".to_string(), Arc::new(OsLogoutDriver));
        map.insert("os_hibernate".to_string(), Arc::new(OsHibernateDriver));
        map.insert("os_get_uptime".to_string(), Arc::new(OsGetUptimeDriver));
        map.insert("os_get_hostname".to_string(), Arc::new(OsGetHostnameDriver));
        map.insert("os_get_user".to_string(), Arc::new(OsGetUserDriver));
        map.insert("os_memory_info".to_string(), Arc::new(OsMemoryInfoDriver));
        map.insert("os_battery_info".to_string(), Arc::new(OsBatteryInfoDriver));
        map.insert(
            "os_notification".to_string(),
            Arc::new(OsNotificationDriver),
        );
    }
}
