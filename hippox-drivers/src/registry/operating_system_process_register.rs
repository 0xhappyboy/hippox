//! Operating system process drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemProcess;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_process", feature = "all"))]
    {
        use crate::drivers::operating_system_process::{
            ProcessGetPidDriver, ProcessInfoDriver, ProcessIsRunningDriver, ProcessKillByNameDriver,
            ProcessKillDriver, ProcessListDriver,
        };
        map.insert("process_list".to_string(), Arc::new(ProcessListDriver));
        map.insert("process_kill".to_string(), Arc::new(ProcessKillDriver));
        map.insert(
            "process_kill_by_name".to_string(),
            Arc::new(ProcessKillByNameDriver),
        );
        map.insert(
            "process_is_running".to_string(),
            Arc::new(ProcessIsRunningDriver),
        );
        map.insert("process_get_pid".to_string(), Arc::new(ProcessGetPidDriver));
        map.insert("process_info".to_string(), Arc::new(ProcessInfoDriver));
    }
}
