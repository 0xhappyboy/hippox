//! Time skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Time;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "time", feature = "all"))]
    {
        use crate::skills::time::*;
        map.insert("datetime".to_string(), Arc::new(DateTimeSkill));
        map.insert("os_get_time".to_string(), Arc::new(OsGetTimeSkill));
        map.insert("os_set_time".to_string(), Arc::new(OsSetTimeSkill));
    }
}
