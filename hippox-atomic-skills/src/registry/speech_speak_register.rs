//! Speech synthesis skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::SpeechSpeak;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "speech_speak", feature = "all"))]
    {
        use crate::skills::speech_speak::*;
        map.insert("speech_speak".to_string(), Arc::new(SpeechSpeakSkill));
    }
}