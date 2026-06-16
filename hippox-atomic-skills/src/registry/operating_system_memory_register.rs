//! Operating system memory skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::OperatingSystemMemory;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_memory", feature = "all"))]
    {
        use crate::skills::operating_system_memory::*;

        map.insert("memory_read".to_string(), Arc::new(MemoryReadSkill));
        map.insert("memory_scan".to_string(), Arc::new(MemoryScanSkill));
        map.insert("module_base".to_string(), Arc::new(ModuleBaseSkill));
    }
}
