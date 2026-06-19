//! Audio control drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Audio;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "audio_control", feature = "all"))]
    {
        use crate::drivers::audio_control::*;
        map.insert(
            "audio_control_volume_get".to_string(),
            Arc::new(AudioControlVolumeGetDriver),
        );
        map.insert(
            "audio_control_volume_set".to_string(),
            Arc::new(AudioControlVolumeSetDriver),
        );
        map.insert(
            "audio_control_volume_up".to_string(),
            Arc::new(AudioControlVolumeUpDriver),
        );
        map.insert(
            "audio_control_volume_down".to_string(),
            Arc::new(AudioControlVolumeDownDriver),
        );
        map.insert(
            "audio_control_mute".to_string(),
            Arc::new(AudioControlMuteDriver),
        );
        map.insert(
            "audio_control_unmute".to_string(),
            Arc::new(AudioControlUnmuteDriver),
        );
        map.insert(
            "audio_control_output_device_list".to_string(),
            Arc::new(AudioControlOutputDeviceListDriver),
        );
        map.insert(
            "audio_control_output_device_set".to_string(),
            Arc::new(AudioControlOutputDeviceSetDriver),
        );
        map.insert(
            "audio_control_input_device_list".to_string(),
            Arc::new(AudioControlInputDeviceListDriver),
        );
        map.insert(
            "audio_control_input_volume_set".to_string(),
            Arc::new(AudioControlInputVolumeSetDriver),
        );
    }
}
