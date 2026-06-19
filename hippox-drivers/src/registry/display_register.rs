//! Display control drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Display;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "display_control", feature = "all"))]
    {
        use crate::drivers::display_control::*;

        map.insert(
            "display_control_list".to_string(),
            Arc::new(DisplayControlListDriver),
        );
        map.insert(
            "display_control_primary_get".to_string(),
            Arc::new(DisplayControlPrimaryGetDriver),
        );
        map.insert(
            "display_control_resolution_get".to_string(),
            Arc::new(DisplayControlResolutionGetDriver),
        );
        map.insert(
            "display_control_resolution_set".to_string(),
            Arc::new(DisplayControlResolutionSetDriver),
        );
        map.insert(
            "display_control_scale_get".to_string(),
            Arc::new(DisplayControlScaleGetDriver),
        );
        map.insert(
            "display_control_orientation_get".to_string(),
            Arc::new(DisplayControlOrientationGetDriver),
        );
        map.insert(
            "display_control_orientation_set".to_string(),
            Arc::new(DisplayControlOrientationSetDriver),
        );
        map.insert(
            "display_control_refresh_rate_get".to_string(),
            Arc::new(DisplayControlRefreshRateGetDriver),
        );
        map.insert(
            "display_control_brightness_get".to_string(),
            Arc::new(DisplayControlBrightnessGetDriver),
        );
        map.insert(
            "display_control_brightness_set".to_string(),
            Arc::new(DisplayControlBrightnessSetDriver),
        );
    }
}
