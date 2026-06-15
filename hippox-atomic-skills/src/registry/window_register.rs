//! Window control skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Window;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "window_control", feature = "all"))]
    {
        use crate::skills::window_control::*;
        map.insert(
            "window_control_minimize".to_string(),
            Arc::new(WindowControlMinimizeSkill),
        );
        map.insert(
            "window_control_maximize".to_string(),
            Arc::new(WindowControlMaximizeSkill),
        );
        map.insert(
            "window_control_restore".to_string(),
            Arc::new(WindowControlRestoreSkill),
        );
        map.insert(
            "window_control_resize".to_string(),
            Arc::new(WindowControlResizeSkill),
        );
        map.insert(
            "window_control_move".to_string(),
            Arc::new(WindowControlMoveSkill),
        );
        map.insert(
            "window_control_close".to_string(),
            Arc::new(WindowControlCloseSkill),
        );
        map.insert(
            "window_control_kill".to_string(),
            Arc::new(WindowControlKillSkill),
        );
        map.insert(
            "window_control_bring_to_top".to_string(),
            Arc::new(WindowControlBringToTopSkill),
        );
        map.insert(
            "window_control_send_to_back".to_string(),
            Arc::new(WindowControlSendToBackSkill),
        );
        map.insert(
            "window_control_set_always_on_top".to_string(),
            Arc::new(WindowControlSetAlwaysOnTopSkill),
        );
        map.insert(
            "window_control_get_title".to_string(),
            Arc::new(WindowControlGetTitleSkill),
        );
        map.insert(
            "window_control_get_process".to_string(),
            Arc::new(WindowControlGetProcessSkill),
        );
        map.insert(
            "window_control_screenshot".to_string(),
            Arc::new(WindowControlScreenshotSkill),
        );
        map.insert(
            "window_control_ocr_region".to_string(),
            Arc::new(WindowControlOcrRegionSkill),
        );
        map.insert(
            "window_control_list".to_string(),
            Arc::new(WindowControlListSkill),
        );
        map.insert(
            "window_control_find".to_string(),
            Arc::new(WindowControlFindSkill),
        );
        map.insert(
            "window_control_activate".to_string(),
            Arc::new(WindowControlActivateSkill),
        );
        map.insert(
            "window_control_get_position".to_string(),
            Arc::new(WindowControlGetPositionSkill),
        );
        map.insert(
            "window_control_get_focus".to_string(),
            Arc::new(WindowControlGetFocusSkill),
        );
        map.insert(
            "window_control_get_selected".to_string(),
            Arc::new(WindowControlGetSelectedSkill),
        );
        map.insert(
            "window_control_send_keys".to_string(),
            Arc::new(WindowControlSendKeysSkill),
        );
        map.insert(
            "window_control_send_shortcut".to_string(),
            Arc::new(WindowControlSendShortcutSkill),
        );
        map.insert(
            "window_control_wait_for_focus".to_string(),
            Arc::new(WindowControlWaitForFocusSkill),
        );
    }
}
