//! Audio control skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Audio;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "audio_control", feature = "all"))]
    {
        use crate::skills::audio_control::*;
        map.insert(
            "audio_control_volume_get".to_string(),
            Arc::new(AudioControlVolumeGetSkill),
        );
        map.insert(
            "audio_control_volume_set".to_string(),
            Arc::new(AudioControlVolumeSetSkill),
        );
        map.insert(
            "audio_control_volume_up".to_string(),
            Arc::new(AudioControlVolumeUpSkill),
        );
        map.insert(
            "audio_control_volume_down".to_string(),
            Arc::new(AudioControlVolumeDownSkill),
        );
        map.insert(
            "audio_control_mute".to_string(),
            Arc::new(AudioControlMuteSkill),
        );
        map.insert(
            "audio_control_unmute".to_string(),
            Arc::new(AudioControlUnmuteSkill),
        );
        map.insert(
            "audio_control_output_device_list".to_string(),
            Arc::new(AudioControlOutputDeviceListSkill),
        );
        map.insert(
            "audio_control_output_device_set".to_string(),
            Arc::new(AudioControlOutputDeviceSetSkill),
        );
        map.insert(
            "audio_control_input_device_list".to_string(),
            Arc::new(AudioControlInputDeviceListSkill),
        );
        map.insert(
            "audio_control_input_volume_set".to_string(),
            Arc::new(AudioControlInputVolumeSetSkill),
        );
    }
}
