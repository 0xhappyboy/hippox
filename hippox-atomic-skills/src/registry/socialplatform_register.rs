//! Social-Platform skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::SocialPlatform;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "social_platform", feature = "all"))]
    {
        use crate::skills::{
            SendDingDingSkill, SendFeishuSkill, SendTelegramSkill, SendWecomSkill,
        };
        map.insert("send_telegram".to_string(), Arc::new(SendTelegramSkill));
        map.insert("send_dingding".to_string(), Arc::new(SendDingDingSkill));
        map.insert("send_feishu".to_string(), Arc::new(SendFeishuSkill));
        map.insert("send_wecom".to_string(), Arc::new(SendWecomSkill));
    }
}
