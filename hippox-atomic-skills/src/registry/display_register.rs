//! Display control skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Display;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "display_control", feature = "all"))]
    {
        use crate::skills::display_control::*;

        map.insert(
            "display_control_list".to_string(),
            Arc::new(DisplayControlListSkill),
        );
        map.insert(
            "display_control_primary_get".to_string(),
            Arc::new(DisplayControlPrimaryGetSkill),
        );
        map.insert(
            "display_control_resolution_get".to_string(),
            Arc::new(DisplayControlResolutionGetSkill),
        );
        map.insert(
            "display_control_resolution_set".to_string(),
            Arc::new(DisplayControlResolutionSetSkill),
        );
        map.insert(
            "display_control_scale_get".to_string(),
            Arc::new(DisplayControlScaleGetSkill),
        );
        map.insert(
            "display_control_orientation_get".to_string(),
            Arc::new(DisplayControlOrientationGetSkill),
        );
        map.insert(
            "display_control_orientation_set".to_string(),
            Arc::new(DisplayControlOrientationSetSkill),
        );
        map.insert(
            "display_control_refresh_rate_get".to_string(),
            Arc::new(DisplayControlRefreshRateGetSkill),
        );
        map.insert(
            "display_control_brightness_get".to_string(),
            Arc::new(DisplayControlBrightnessGetSkill),
        );
        map.insert(
            "display_control_brightness_set".to_string(),
            Arc::new(DisplayControlBrightnessSetSkill),
        );
    }
}
