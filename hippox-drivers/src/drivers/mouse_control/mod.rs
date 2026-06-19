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

pub use mouse_control_position_get::MouseControlPositionGetDriver;
pub use mouse_control_click::MouseControlClickDriver;
pub use mouse_control_double_click::MouseControlDoubleClickDriver;
pub use mouse_control_right_click::MouseControlRightClickDriver;
pub use mouse_control_move_to::MouseControlMoveToDriver;
pub use mouse_control_move_relative::MouseControlMoveRelativeDriver;
pub use mouse_control_press::MouseControlPressDriver;
pub use mouse_control_release::MouseControlReleaseDriver;
pub use mouse_control_drag::MouseControlDragDriver;
pub use mouse_control_scroll::MouseControlScrollDriver;
pub use mouse_control_smooth_move::MouseControlSmoothMoveDriver;
pub use mouse_control_get_cursor::MouseControlGetCursorDriver;