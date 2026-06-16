//! Mouse control skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Mouse;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "mouse_control", feature = "all"))]
    {
        use crate::skills::mouse_control::*;
        map.insert("mouse_control_position_get".to_string(), Arc::new(MouseControlPositionGetSkill));
        map.insert("mouse_control_click".to_string(), Arc::new(MouseControlClickSkill));
        map.insert("mouse_control_double_click".to_string(), Arc::new(MouseControlDoubleClickSkill));
        map.insert("mouse_control_right_click".to_string(), Arc::new(MouseControlRightClickSkill));
        map.insert("mouse_control_move_to".to_string(), Arc::new(MouseControlMoveToSkill));
        map.insert("mouse_control_move_relative".to_string(), Arc::new(MouseControlMoveRelativeSkill));
        map.insert("mouse_control_press".to_string(), Arc::new(MouseControlPressSkill));
        map.insert("mouse_control_release".to_string(), Arc::new(MouseControlReleaseSkill));
        map.insert("mouse_control_drag".to_string(), Arc::new(MouseControlDragSkill));
        map.insert("mouse_control_scroll".to_string(), Arc::new(MouseControlScrollSkill));
        map.insert("mouse_control_smooth_move".to_string(), Arc::new(MouseControlSmoothMoveSkill));
        map.insert("mouse_control_get_cursor".to_string(), Arc::new(MouseControlGetCursorSkill));
    }
}