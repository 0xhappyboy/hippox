//! Keyboard control skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Keyboard;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "keyboard_control", feature = "all"))]
    {
        use crate::skills::keyboard_control::*;

        map.insert(
            "keyboard_control_press".to_string(),
            Arc::new(KeyboardControlPressSkill),
        );
        map.insert(
            "keyboard_control_down".to_string(),
            Arc::new(KeyboardControlDownSkill),
        );
        map.insert(
            "keyboard_control_up".to_string(),
            Arc::new(KeyboardControlUpSkill),
        );
        map.insert(
            "keyboard_control_type_text".to_string(),
            Arc::new(KeyboardControlTypeTextSkill),
        );
        map.insert(
            "keyboard_control_shortcut".to_string(),
            Arc::new(KeyboardControlShortcutSkill),
        );
        map.insert(
            "keyboard_control_hotkey".to_string(),
            Arc::new(KeyboardControlHotkeySkill),
        );
        map.insert(
            "keyboard_control_modifier".to_string(),
            Arc::new(KeyboardControlModifierSkill),
        );
    }
}
