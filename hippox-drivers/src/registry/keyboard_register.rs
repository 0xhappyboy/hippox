//! Keyboard control drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Keyboard;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "keyboard_control", feature = "all"))]
    {
        use crate::drivers::keyboard_control::*;

        map.insert(
            "keyboard_control_press".to_string(),
            Arc::new(KeyboardControlPressDriver),
        );
        map.insert(
            "keyboard_control_down".to_string(),
            Arc::new(KeyboardControlDownDriver),
        );
        map.insert(
            "keyboard_control_up".to_string(),
            Arc::new(KeyboardControlUpDriver),
        );
        map.insert(
            "keyboard_control_type_text".to_string(),
            Arc::new(KeyboardControlTypeTextDriver),
        );
        map.insert(
            "keyboard_control_shortcut".to_string(),
            Arc::new(KeyboardControlShortcutDriver),
        );
        map.insert(
            "keyboard_control_hotkey".to_string(),
            Arc::new(KeyboardControlHotkeyDriver),
        );
        map.insert(
            "keyboard_control_modifier".to_string(),
            Arc::new(KeyboardControlModifierDriver),
        );
    }
}
