//! Operating system process skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::OperatingSystemProcess;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_process", feature = "all"))]
    {
        use crate::skills::operating_system_process::{
            ProcessGetPidSkill, ProcessInfoSkill, ProcessIsRunningSkill, ProcessKillByNameSkill,
            ProcessKillSkill, ProcessListSkill,
        };
        map.insert("process_list".to_string(), Arc::new(ProcessListSkill));
        map.insert("process_kill".to_string(), Arc::new(ProcessKillSkill));
        map.insert(
            "process_kill_by_name".to_string(),
            Arc::new(ProcessKillByNameSkill),
        );
        map.insert(
            "process_is_running".to_string(),
            Arc::new(ProcessIsRunningSkill),
        );
        map.insert("process_get_pid".to_string(), Arc::new(ProcessGetPidSkill));
        map.insert("process_info".to_string(), Arc::new(ProcessInfoSkill));
    }
}
