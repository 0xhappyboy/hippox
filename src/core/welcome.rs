//! Welcome message generation

use super::registry::get_startup_banner;
use crate::t;
use serde_json::Value;

/// Welcome message structure containing registry information
#[derive(Debug, Clone, serde::Serialize)]
pub struct WelcomeMessage {
    pub type_: String,
    pub message: String,
    pub skills_registry: Value,
    pub instances_registry: Value,
    pub version: String,
}

/// Generate welcome message with registries
pub fn generate_welcome_message(skills_registry: &str, instances_registry: &str) -> String {
    let welcome = WelcomeMessage {
        type_: "welcome".to_string(),
        message: format!("{}\n\n{}", get_startup_banner(), t!("app.welcome_message")),
        skills_registry: serde_json::from_str(skills_registry).unwrap_or(Value::Null),
        instances_registry: serde_json::from_str(instances_registry).unwrap_or(Value::Null),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    serde_json::to_string_pretty(&welcome).unwrap_or_else(|_| {
        format!(
            "{{\"type\":\"welcome\",\"message\":\"{}\"}}",
            t!("app.welcome_message")
        )
    })
}
