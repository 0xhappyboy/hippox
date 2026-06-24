//! Task management module for Hippox core
//!
//! This module provides a GLOBAL static task pool (independent of Hippox instances)
//! with automatic background execution engine that starts at program load.

mod api;
mod core;
mod event_bus;
mod executor;

pub use api::*;
pub use core::*;
pub use event_bus::*;
pub use executor::ExecutableTask;
pub use executor::TaskStateUpdater;
pub use executor::get_state_updater;
