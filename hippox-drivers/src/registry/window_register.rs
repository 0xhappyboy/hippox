//! Window control drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Window;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "window_control", feature = "all"))]
    {
        use crate::drivers::window_control::*;
        map.insert(
            "window_control_minimize".to_string(),
            Arc::new(WindowControlMinimizeDriver),
        );
        map.insert(
            "window_control_maximize".to_string(),
            Arc::new(WindowControlMaximizeDriver),
        );
        map.insert(
            "window_control_restore".to_string(),
            Arc::new(WindowControlRestoreDriver),
        );
        map.insert(
            "window_control_resize".to_string(),
            Arc::new(WindowControlResizeDriver),
        );
        map.insert(
            "window_control_move".to_string(),
            Arc::new(WindowControlMoveDriver),
        );
        map.insert(
            "window_control_close".to_string(),
            Arc::new(WindowControlCloseDriver),
        );
        map.insert(
            "window_control_kill".to_string(),
            Arc::new(WindowControlKillDriver),
        );
        map.insert(
            "window_control_bring_to_top".to_string(),
            Arc::new(WindowControlBringToTopDriver),
        );
        map.insert(
            "window_control_send_to_back".to_string(),
            Arc::new(WindowControlSendToBackDriver),
        );
        map.insert(
            "window_control_set_always_on_top".to_string(),
            Arc::new(WindowControlSetAlwaysOnTopDriver),
        );
        map.insert(
            "window_control_get_title".to_string(),
            Arc::new(WindowControlGetTitleDriver),
        );
        map.insert(
            "window_control_get_process".to_string(),
            Arc::new(WindowControlGetProcessDriver),
        );
        map.insert(
            "window_control_screenshot".to_string(),
            Arc::new(WindowControlScreenshotDriver),
        );
        map.insert(
            "window_control_ocr_region".to_string(),
            Arc::new(WindowControlOcrRegionDriver),
        );
        map.insert(
            "window_control_list".to_string(),
            Arc::new(WindowControlListDriver),
        );
        map.insert(
            "window_control_find".to_string(),
            Arc::new(WindowControlFindDriver),
        );
        map.insert(
            "window_control_activate".to_string(),
            Arc::new(WindowControlActivateDriver),
        );
        map.insert(
            "window_control_get_position".to_string(),
            Arc::new(WindowControlGetPositionDriver),
        );
        map.insert(
            "window_control_get_focus".to_string(),
            Arc::new(WindowControlGetFocusDriver),
        );
        map.insert(
            "window_control_get_selected".to_string(),
            Arc::new(WindowControlGetSelectedDriver),
        );
        map.insert(
            "window_control_send_keys".to_string(),
            Arc::new(WindowControlSendKeysDriver),
        );
        map.insert(
            "window_control_send_shortcut".to_string(),
            Arc::new(WindowControlSendShortcutDriver),
        );
        map.insert(
            "window_control_wait_for_focus".to_string(),
            Arc::new(WindowControlWaitForFocusDriver),
        );
    }
}
