//! Skill Registry Module
//!
//! This module provides a central registry for managing all available skills in the system.

mod core;
mod types;

// Import all register modules
#[cfg(any(feature = "application_control", feature = "all"))]
mod application_register;
#[cfg(any(feature = "audio_control", feature = "all"))]
mod audio_register;
#[cfg(any(feature = "helloworld", feature = "all"))]
mod basic_register;
#[cfg(any(feature = "blockchain", feature = "all"))]
mod blockchain_register;
#[cfg(any(feature = "bluetooth", feature = "all"))]
mod bluetooth_register;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
mod browser_register;
#[cfg(any(feature = "db", feature = "all"))]
mod db_register;
#[cfg(any(feature = "devops", feature = "all"))]
mod devops_register;
#[cfg(any(feature = "display_control", feature = "all"))]
mod display_register;
#[cfg(any(feature = "document", feature = "all"))]
mod document_register;
#[cfg(any(feature = "file", feature = "all"))]
mod file_register;
#[cfg(any(feature = "keyboard_control", feature = "all"))]
mod keyboard_register;
#[cfg(any(feature = "math", feature = "all"))]
mod math_register;
#[cfg(any(feature = "media", feature = "all"))]
mod media_register;
#[cfg(any(feature = "message", feature = "all"))]
mod message_register;
#[cfg(any(feature = "mouse_control", feature = "all"))]
mod mouse_register;
#[cfg(any(feature = "net", feature = "all"))]
mod net_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
mod os_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
mod process_register;
#[cfg(any(feature = "speech_speak", feature = "all"))]
mod speech_register;
#[cfg(any(feature = "terminal_commands", feature = "all"))]
mod terminal_register;
#[cfg(any(feature = "text", feature = "all"))]
mod text_register;
#[cfg(any(feature = "wifi", feature = "all"))]
mod wifi_register;
#[cfg(any(feature = "window_control", feature = "all"))]
mod window_register;

pub use core::*;
pub use types::*;
