//! Pipeline module for two-stage execution
//!
//! Stage One: Core workflow execution, outputs standard JSON
//! Stage Two: Format conversion based on user's structure requirements

pub(crate) mod core;
pub(crate) mod detector;
pub(crate) mod intent;
pub(crate) mod types;

pub(crate) use core::*;
pub(crate) use detector::*;
pub(crate) use intent::*;
pub(crate) use types::*;
