//! Task management module for Hippox core
//!
//! This module provides a GLOBAL static task pool (independent of Hippox instances)
//! with automatic background execution engine that starts at program load.

mod api;
mod engine;
mod executor;
mod types;

pub use api::*;
pub use executor::ExecutableTask;
pub use executor::TaskStateUpdater;
pub use types::*;
