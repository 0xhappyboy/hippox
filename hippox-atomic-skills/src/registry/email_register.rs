//! Email skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Email;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "email", feature = "all"))]
    {
        use crate::skills::email::*;
        map.insert("send_email".to_string(), Arc::new(SendEmailSkill));
    }
}
