//! Core skill registry implementation

use crate::Skill;
use crate::registry::SkillCategory;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

// Import all register modules
#[cfg(any(feature = "application_control", feature = "all"))]
use crate::registry::application_register;
#[cfg(any(feature = "audio_control", feature = "all"))]
use crate::registry::audio_register;
#[cfg(any(feature = "helloworld", feature = "all"))]
use crate::registry::basic_register;
#[cfg(any(feature = "blockchain", feature = "all"))]
use crate::registry::blockchain_register;
#[cfg(any(feature = "bluetooth", feature = "all"))]
use crate::registry::bluetooth_register;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::registry::browser_register;
#[cfg(any(feature = "db", feature = "all"))]
use crate::registry::db_register;
#[cfg(any(feature = "devops", feature = "all"))]
use crate::registry::devops_register;
#[cfg(any(feature = "display_control", feature = "all"))]
use crate::registry::display_register;
#[cfg(any(feature = "document", feature = "all"))]
use crate::registry::document_register;
#[cfg(any(feature = "file", feature = "all"))]
use crate::registry::file_register;
#[cfg(any(feature = "keyboard_control", feature = "all"))]
use crate::registry::keyboard_register;
#[cfg(any(feature = "math", feature = "all"))]
use crate::registry::math_register;
#[cfg(any(feature = "media", feature = "all"))]
use crate::registry::media_register;
#[cfg(any(feature = "message", feature = "all"))]
use crate::registry::message_register;
#[cfg(any(feature = "mouse_control", feature = "all"))]
use crate::registry::mouse_register;
#[cfg(any(feature = "net", feature = "all"))]
use crate::registry::net_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
use crate::registry::os_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
use crate::registry::process_register;
#[cfg(any(feature = "speech_speak", feature = "all"))]
use crate::registry::speech_register;
#[cfg(any(feature = "terminal_commands", feature = "all"))]
use crate::registry::terminal_register;
#[cfg(any(feature = "text", feature = "all"))]
use crate::registry::text_register;
#[cfg(any(feature = "wifi", feature = "all"))]
use crate::registry::wifi_register;
#[cfg(any(feature = "window_control", feature = "all"))]
use crate::registry::window_register;

/// Global registry type: mapping from category to (skill_name -> skill_impl)
pub type SkillRegistryMap = HashMap<SkillCategory, HashMap<String, Arc<dyn Skill>>>;

/// Global, lazily-initialized, thread-safe registry of all available skills.
static SKILL_REGISTRY: Lazy<RwLock<SkillRegistryMap>> = Lazy::new(|| {
    let mut registry: SkillRegistryMap = HashMap::new();
    // Initialize all category maps
    for category in [
        SkillCategory::Basic,
        SkillCategory::File,
        SkillCategory::Math,
        SkillCategory::Net,
        SkillCategory::Os,
        SkillCategory::Process,
        SkillCategory::Document,
        SkillCategory::Message,
        SkillCategory::Db,
        SkillCategory::Text,
        SkillCategory::Devops,
        SkillCategory::Media,
        SkillCategory::Blockchain,
        SkillCategory::Browser,
        SkillCategory::Window,
        SkillCategory::Speech,
        SkillCategory::Keyboard,
        SkillCategory::Mouse,
        SkillCategory::Audio,
        SkillCategory::Application,
        SkillCategory::Display,
        SkillCategory::Wifi,
        SkillCategory::Bluetooth,
        SkillCategory::Terminal,
    ] {
        registry.insert(category, HashMap::new());
    }
    #[cfg(any(feature = "helloworld", feature = "all"))]
    basic_register::register(&mut registry);

    #[cfg(any(feature = "file", feature = "all"))]
    file_register::register(&mut registry);

    #[cfg(any(feature = "math", feature = "all"))]
    math_register::register(&mut registry);

    #[cfg(any(feature = "net", feature = "all"))]
    net_register::register(&mut registry);

    #[cfg(any(feature = "operating_system", feature = "all"))]
    os_register::register(&mut registry);

    #[cfg(any(feature = "operating_system", feature = "all"))]
    process_register::register(&mut registry);

    #[cfg(any(feature = "document", feature = "all"))]
    document_register::register(&mut registry);

    #[cfg(any(feature = "message", feature = "all"))]
    message_register::register(&mut registry);

    #[cfg(any(feature = "db", feature = "all"))]
    db_register::register(&mut registry);

    #[cfg(any(feature = "text", feature = "all"))]
    text_register::register(&mut registry);

    #[cfg(any(feature = "devops", feature = "all"))]
    devops_register::register(&mut registry);

    #[cfg(any(feature = "media", feature = "all"))]
    media_register::register(&mut registry);

    #[cfg(any(feature = "blockchain", feature = "all"))]
    blockchain_register::register(&mut registry);

    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    browser_register::register(&mut registry);

    #[cfg(any(feature = "window_control", feature = "all"))]
    window_register::register(&mut registry);

    #[cfg(any(feature = "speech_speak", feature = "all"))]
    speech_register::register(&mut registry);

    #[cfg(any(feature = "keyboard_control", feature = "all"))]
    keyboard_register::register(&mut registry);

    #[cfg(any(feature = "mouse_control", feature = "all"))]
    mouse_register::register(&mut registry);

    #[cfg(any(feature = "audio_control", feature = "all"))]
    audio_register::register(&mut registry);

    #[cfg(any(feature = "application_control", feature = "all"))]
    application_register::register(&mut registry);

    #[cfg(any(feature = "display_control", feature = "all"))]
    display_register::register(&mut registry);

    #[cfg(any(feature = "wifi", feature = "all"))]
    wifi_register::register(&mut registry);

    #[cfg(any(feature = "bluetooth", feature = "all"))]
    bluetooth_register::register(&mut registry);

    #[cfg(any(feature = "terminal_commands", feature = "all"))]
    terminal_register::register(&mut registry);

    RwLock::new(registry)
});

/// Get read lock on registry
pub fn get_registry() -> std::sync::RwLockReadGuard<'static, SkillRegistryMap> {
    SKILL_REGISTRY.read().unwrap()
}

/// Get write lock on registry
pub fn get_registry_mut() -> std::sync::RwLockWriteGuard<'static, SkillRegistryMap> {
    SKILL_REGISTRY.write().unwrap()
}

/// Get all skills
pub fn get_all_skills() -> Vec<Arc<dyn Skill>> {
    let registry = get_registry();
    let mut skills = Vec::new();
    for category_map in registry.values() {
        for skill in category_map.values() {
            skills.push(skill.clone());
        }
    }
    skills
}

/// Get skills by category
pub fn get_skills_by_category(category: SkillCategory) -> Vec<Arc<dyn Skill>> {
    let registry = get_registry();
    registry
        .get(&category)
        .map(|map| map.values().cloned().collect())
        .unwrap_or_default()
}

/// Get skill by name (searches across all categories)
pub fn get_skill(name: &str) -> Option<Arc<dyn Skill>> {
    let registry = get_registry();
    for category_map in registry.values() {
        if let Some(skill) = category_map.get(name) {
            return Some(skill.clone());
        }
    }
    None
}

/// Get skill by name and category
pub fn get_skill_by_category(name: &str, category: SkillCategory) -> Option<Arc<dyn Skill>> {
    let registry = get_registry();
    registry
        .get(&category)
        .and_then(|map| map.get(name))
        .cloned()
}

/// Register a skill dynamically
pub fn register_skill(category: SkillCategory, name: String, skill: Arc<dyn Skill>) {
    let mut registry = get_registry_mut();
    registry
        .entry(category)
        .or_insert_with(HashMap::new)
        .insert(name, skill);
}

/// Check if a skill exists
pub fn has_skill(name: &str) -> bool {
    get_skill(name).is_some()
}

/// List all skill names
pub fn list_skills() -> Vec<String> {
    let registry = get_registry();
    let mut names = Vec::new();
    for category_map in registry.values() {
        names.extend(category_map.keys().cloned());
    }
    names
}

/// List skills by category
pub fn list_skills_by_category(category: SkillCategory) -> Vec<String> {
    let registry = get_registry();
    registry
        .get(&category)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

/// Get skill categories with counts
pub fn get_skill_categories() -> Vec<(String, usize)> {
    let registry = get_registry();
    let mut result = Vec::new();
    for (category, map) in registry.iter() {
        result.push((category.as_str().to_string(), map.len()));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}
