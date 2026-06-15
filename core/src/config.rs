//! Core configuration structure and global state

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Global static configuration instance
pub(crate) static HIPPOX_CORE_CONFIG: Lazy<RwLock<HippoxConfig>> =
    Lazy::new(|| RwLock::new(HippoxConfig::default()));

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IdentityInformation {
    // Name, Default: Hippox
    pub name: Option<String>,
    // Sex
    pub sex: Option<String>,
    // age
    pub age: Option<String>,
    // Species
    pub species: Option<String>,
    // Role/Profession (e.g., "assistant", "teacher", "life coach")
    pub role: Option<String>,
    // Personality traits (e.g., "friendly", "humorous", "professional")
    pub personality: Option<String>,
    // Tone style (e.g., "casual", "formal", "poetic")
    pub tone_style: Option<String>,
    // Knowledge scope (e.g., "general", "medical", "programming")
    pub knowledge_scope: Option<String>,
    // Catchphrase / habitual expression (e.g., "Haha", "I see")
    pub catchphrase: Option<String>,
    // Taboos / prohibited topics (e.g., "no politics", "no medical advice")
    pub taboos: Option<String>,
}

impl Default for IdentityInformation {
    fn default() -> Self {
        Self {
            name: Some("Hippox".to_string()),
            sex: None,
            age: None,
            species: None,
            role: None,
            personality: None,
            tone_style: None,
            knowledge_scope: None,
            catchphrase: None,
            taboos: None,
        }
    }
}

/// Hippox global configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HippoxConfig {
    // Application settings
    pub lang: String,
    // Identity information
    pub identity_information: IdentityInformation,
}

impl Default for HippoxConfig {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            identity_information: IdentityInformation::default(),
        }
    }
}

impl HippoxConfig {
    /// Get identity information reference
    pub fn get_identity(&self) -> &IdentityInformation {
        &self.identity_information
    }

    /// Get mutable identity information reference
    pub fn get_identity_mut(&mut self) -> &mut IdentityInformation {
        &mut self.identity_information
    }

    /// Update identity information
    pub fn update_identity<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut IdentityInformation),
    {
        f(&mut self.identity_information);
        self
    }

    /// Load from TOML configuration file
    pub fn load_from_toml_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: HippoxConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from JSON configuration file
    pub fn load_from_json_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: HippoxConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}

/// Get a clone of the global configuration
pub fn get_config() -> HippoxConfig {
    HIPPOX_CORE_CONFIG.read().unwrap().clone()
}

/// Update config with a closure
pub fn update_config<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut HippoxConfig),
{
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    f(&mut global);
    Ok(())
}

/// Get current language setting
pub fn get_lang() -> String {
    HIPPOX_CORE_CONFIG.read().unwrap().lang.clone()
}

/// Set language
pub fn set_lang(lang: String) -> anyhow::Result<()> {
    let mut config = HIPPOX_CORE_CONFIG.write().unwrap();
    config.lang = lang;
    Ok(())
}

/// Get the global Hippox core configuration (alias for get_config for backward compatibility)
pub fn get_hippox_core_config() -> HippoxConfig {
    HIPPOX_CORE_CONFIG.read().unwrap().clone()
}
