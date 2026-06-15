//! Text processing skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Text;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "text", feature = "all"))]
    {
        use crate::skills::text::*;
        map.insert("text_diff".to_string(), Arc::new(TextDiffSkill));
        map.insert("text_sort".to_string(), Arc::new(TextSortSkill));
        map.insert("text_deduplicate".to_string(), Arc::new(TextDeduplicateSkill));
        map.insert("text_filter".to_string(), Arc::new(TextFilterSkill));
        map.insert("regex_match".to_string(), Arc::new(RegexMatchSkill));
        map.insert("regex_find".to_string(), Arc::new(RegexFindSkill));
        map.insert("regex_replace".to_string(), Arc::new(RegexReplaceSkill));
        map.insert("regex_extract".to_string(), Arc::new(RegexExtractSkill));
    }
}