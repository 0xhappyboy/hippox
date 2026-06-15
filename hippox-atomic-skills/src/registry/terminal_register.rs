//! Terminal commands skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Terminal;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "terminal_commands", feature = "all"))]
    {
        use crate::skills::ExecCommandSkill;

        map.insert("exec_command".to_string(), Arc::new(ExecCommandSkill));
    }
}
