//! Pipeline module for two-stage execution
//!
//! Stage One: Core workflow execution, outputs standard JSON
//! Stage Two: Format conversion based on user's structure requirements

mod detector;
mod stage;

pub use detector::*;
pub use stage::*;
