//! Process management skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Process;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "operating_system", feature = "all"))]
    {
        use crate::skills::operating_system::*;
        
        map.insert("process_list".to_string(), Arc::new(ProcessListSkill));
        map.insert("process_kill".to_string(), Arc::new(ProcessKillSkill));
        map.insert("process_kill_by_name".to_string(), Arc::new(ProcessKillByNameSkill));
        map.insert("process_is_running".to_string(), Arc::new(ProcessIsRunningSkill));
        map.insert("process_get_pid".to_string(), Arc::new(ProcessGetPidSkill));
        map.insert("process_info".to_string(), Arc::new(ProcessInfoSkill));
        map.insert("process_basic_info".to_string(), Arc::new(ProcessBasicInfoSkill));
    }
}