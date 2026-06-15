// mouse_control/mod.rs
//! Mouse control skills module

mod common;
mod mouse_control_position_get;
mod mouse_control_click;
mod mouse_control_double_click;
mod mouse_control_right_click;
mod mouse_control_move_to;
mod mouse_control_move_relative;
mod mouse_control_press;
mod mouse_control_release;
mod mouse_control_drag;
mod mouse_control_scroll;
mod mouse_control_smooth_move;
mod mouse_control_get_cursor;

pub use mouse_control_position_get::MouseControlPositionGetSkill;
pub use mouse_control_click::MouseControlClickSkill;
pub use mouse_control_double_click::MouseControlDoubleClickSkill;
pub use mouse_control_right_click::MouseControlRightClickSkill;
pub use mouse_control_move_to::MouseControlMoveToSkill;
pub use mouse_control_move_relative::MouseControlMoveRelativeSkill;
pub use mouse_control_press::MouseControlPressSkill;
pub use mouse_control_release::MouseControlReleaseSkill;
pub use mouse_control_drag::MouseControlDragSkill;
pub use mouse_control_scroll::MouseControlScrollSkill;
pub use mouse_control_smooth_move::MouseControlSmoothMoveSkill;
pub use mouse_control_get_cursor::MouseControlGetCursorSkill;