//! Speech synthesis drivers registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::SpeechSpeak;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "speech_speak", feature = "all"))]
    {
        use crate::drivers::speech_speak::*;
        map.insert("speech_speak".to_string(), Arc::new(SpeechSpeakDriver));
    }
}