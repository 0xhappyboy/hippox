//! Mouse control drivers registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Mouse;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "mouse_control", feature = "all"))]
    {
        use crate::drivers::mouse_control::*;
        map.insert("mouse_control_position_get".to_string(), Arc::new(MouseControlPositionGetDriver));
        map.insert("mouse_control_click".to_string(), Arc::new(MouseControlClickDriver));
        map.insert("mouse_control_double_click".to_string(), Arc::new(MouseControlDoubleClickDriver));
        map.insert("mouse_control_right_click".to_string(), Arc::new(MouseControlRightClickDriver));
        map.insert("mouse_control_move_to".to_string(), Arc::new(MouseControlMoveToDriver));
        map.insert("mouse_control_move_relative".to_string(), Arc::new(MouseControlMoveRelativeDriver));
        map.insert("mouse_control_press".to_string(), Arc::new(MouseControlPressDriver));
        map.insert("mouse_control_release".to_string(), Arc::new(MouseControlReleaseDriver));
        map.insert("mouse_control_drag".to_string(), Arc::new(MouseControlDragDriver));
        map.insert("mouse_control_scroll".to_string(), Arc::new(MouseControlScrollDriver));
        map.insert("mouse_control_smooth_move".to_string(), Arc::new(MouseControlSmoothMoveDriver));
        map.insert("mouse_control_get_cursor".to_string(), Arc::new(MouseControlGetCursorDriver));
    }
}