//! Basic skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Basic;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "helloworld", feature = "all"))]
    {
        use crate::skills::HelloWorldSkill;
        map.insert("helloworld".to_string(), Arc::new(HelloWorldSkill));
    }
}
