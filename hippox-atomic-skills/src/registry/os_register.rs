//! Operating system skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Os;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "operating_system", feature = "all"))]
    {
        use crate::skills::operating_system::clipboard::*;
        use crate::skills::{PortLookupSkill, PortScanSkill, PortTestSkill, operating_system::*};
        map.insert("os_reboot".to_string(), Arc::new(OsRebootSkill));
        map.insert("os_shutdown".to_string(), Arc::new(OsShutdownSkill));
        map.insert("os_sleep".to_string(), Arc::new(OsSleepSkill));
        map.insert("os_lock".to_string(), Arc::new(OsLockSkill));
        map.insert("os_logout".to_string(), Arc::new(OsLogoutSkill));
        map.insert("os_hibernate".to_string(), Arc::new(OsHibernateSkill));
        map.insert("os_get_uptime".to_string(), Arc::new(OsGetUptimeSkill));
        map.insert(
            "os_get_load_average".to_string(),
            Arc::new(OsGetLoadAverageSkill),
        );
        map.insert("os_get_hostname".to_string(), Arc::new(OsGetHostnameSkill));
        map.insert("os_get_user".to_string(), Arc::new(OsGetUserSkill));
        map.insert("os_disk_usage".to_string(), Arc::new(OsDiskUsageSkill));
        map.insert("os_memory_info".to_string(), Arc::new(OsMemoryInfoSkill));
        map.insert("os_cpu_info".to_string(), Arc::new(OsCpuInfoSkill));
        map.insert("os_network_info".to_string(), Arc::new(OsNetworkInfoSkill));
        map.insert("os_battery_info".to_string(), Arc::new(OsBatteryInfoSkill));
        map.insert("os_notification".to_string(), Arc::new(OsNotificationSkill));
        map.insert("system_systeminfo".to_string(), Arc::new(SystemInfoSkill));
        map.insert("port_scan".to_string(), Arc::new(PortScanSkill));
        map.insert("port_lookup".to_string(), Arc::new(PortLookupSkill));
        map.insert("port_test".to_string(), Arc::new(PortTestSkill));
        map.insert("clipboard_get".to_string(), Arc::new(ClipboardGetSkill));
        map.insert("clipboard_set".to_string(), Arc::new(ClipboardSetSkill));
        map.insert("clipboard_clear".to_string(), Arc::new(ClipboardClearSkill));
    }
}
