//! Social-Platform drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::SocialPlatform;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "social_platform", feature = "all"))]
    {
        use crate::drivers::{
            SendDingDingDriver, SendFeishuDriver, SendTelegramDriver, SendWecomDriver,
        };
        map.insert("send_telegram".to_string(), Arc::new(SendTelegramDriver));
        map.insert("send_dingding".to_string(), Arc::new(SendDingDingDriver));
        map.insert("send_feishu".to_string(), Arc::new(SendFeishuDriver));
        map.insert("send_wecom".to_string(), Arc::new(SendWecomDriver));
    }
}
