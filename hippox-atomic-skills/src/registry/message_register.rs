//! Messaging skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Message;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "message", feature = "all"))]
    {
        use crate::skills::message::*;
        
        map.insert("send_email".to_string(), Arc::new(SendEmailSkill));
        map.insert("send_telegram".to_string(), Arc::new(SendTelegramSkill));
        map.insert("send_dingding".to_string(), Arc::new(SendDingDingSkill));
        map.insert("send_feishu".to_string(), Arc::new(SendFeishuSkill));
        map.insert("send_wecom".to_string(), Arc::new(SendWecomSkill));
    }
}