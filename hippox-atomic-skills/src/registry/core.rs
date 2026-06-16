//! Core skill registry implementation

use crate::Skill;
use crate::SkillMetadata;
use crate::registry::SkillCategory;
#[cfg(any(feature = "cryptography", feature = "all"))]
use crate::registry::cryptography_register;
#[cfg(any(feature = "email", feature = "all"))]
use crate::registry::email_register;
#[cfg(any(feature = "time", feature = "all"))]
use crate::registry::time_register;
use once_cell::sync::Lazy;
use serde_json::Value;
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
#[cfg(any(feature = "mouse_control", feature = "all"))]
use crate::registry::mouse_register;
#[cfg(any(feature = "net", feature = "all"))]
use crate::registry::net_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
use crate::registry::os_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
use crate::registry::process_register;
#[cfg(any(feature = "social_platform", feature = "all"))]
use crate::registry::socialplatform_register;
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
        SkillCategory::SocialPlatform,
        SkillCategory::Db,
        SkillCategory::Text,
        SkillCategory::Devops,
        SkillCategory::Media,
        SkillCategory::Blockchain,
        SkillCategory::HaveHeadBrowser,
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
    #[cfg(any(feature = "social_platform", feature = "all"))]
    socialplatform_register::register(&mut registry);
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
    #[cfg(any(feature = "cryptography", feature = "all"))]
    cryptography_register::register(&mut registry);
    #[cfg(any(feature = "time", feature = "all"))]
    time_register::register(&mut registry);
    #[cfg(any(feature = "email", feature = "all"))]
    email_register::register(&mut registry);

    RwLock::new(registry)
});

/// Get read lock on the global skill registry.
///
/// This function provides read-only access to the registry, allowing multiple
/// concurrent readers. It panics if the registry lock is poisoned.
///
/// # Returns
/// A read guard that provides immutable access to the `SkillRegistryMap`.
///
/// # Panics
/// Panics if the `RwLock` is poisoned (e.g., if a previous write operation panicked).
///
/// # Examples
/// ```
/// use your_crate::core::get_registry;
///
/// let registry = get_registry();
/// // Read from the registry...
/// ```
pub fn get_registry() -> std::sync::RwLockReadGuard<'static, SkillRegistryMap> {
    SKILL_REGISTRY.read().unwrap()
}

/// Get write lock on the global skill registry.
///
/// This function provides exclusive write access to the registry, allowing
/// modification of registered skills. It panics if the registry lock is poisoned.
///
/// # Returns
/// A write guard that provides mutable access to the `SkillRegistryMap`.
///
/// # Panics
/// Panics if the `RwLock` is poisoned (e.g., if a previous write operation panicked).
///
/// # Examples
/// ```
/// use your_crate::core::get_registry_mut;
///
/// let mut registry = get_registry_mut();
/// // Modify the registry...
/// ```
pub fn get_registry_mut() -> std::sync::RwLockWriteGuard<'static, SkillRegistryMap> {
    SKILL_REGISTRY.write().unwrap()
}

/// Register a new skill dynamically at runtime.
///
/// This function inserts or updates a skill in the global registry under the
/// specified category. If the category doesn't exist, it will be created automatically.
/// If a skill with the same name already exists in the category, it will be overwritten.
///
/// # Arguments
/// * `category` - The `SkillCategory` under which to register the skill.
/// * `name` - The unique name identifier for the skill.
/// * `skill` - An `Arc<dyn Skill>` trait object implementing the skill logic.
///
/// # Examples
/// ```
/// use std::sync::Arc;
/// use your_crate::core::register_skill;
/// use your_crate::registry::SkillCategory;
///
/// let skill = Arc::new(MySkill::new());
/// register_skill(SkillCategory::Basic, "my_skill".to_string(), skill);
/// ```
pub fn register_skill(category: SkillCategory, name: String, skill: Arc<dyn Skill>) {
    let mut registry = get_registry_mut();
    registry
        .entry(category)
        .or_insert_with(HashMap::new)
        .insert(name, skill);
}

/// Retrieve all registered skills across all categories.
///
/// This function collects and returns a vector containing every skill from
/// every category in the registry.
///
/// # Returns
/// A `Vec<Arc<dyn Skill>>` containing clones of all skill trait objects.
///
/// # Examples
/// ```
/// use your_crate::core::get_all_skills;
///
/// let all_skills = get_all_skills();
/// println!("Total skills available: {}", all_skills.len());
/// ```
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

/// Retrieve a skill by its name, searching across all categories.
///
/// This function performs a linear search through all categories to find a skill
/// with the matching name. If multiple skills share the same name across different
/// categories, only the first one found is returned.
///
/// # Arguments
/// * `name` - The name of the skill to search for.
///
/// # Returns
/// `Some(Arc<dyn Skill>)` if a skill with the given name exists, otherwise `None`.
///
/// # Examples
/// ```
/// use your_crate::core::get_skill_by_name;
///
/// if let Some(skill) = get_skill_by_name("hello_world") {
///     println!("Found skill: {}", skill.name());
/// } else {
///     println!("Skill not found");
/// }
/// ```
pub fn get_skill_by_name(name: &str) -> Option<Arc<dyn Skill>> {
    let registry = get_registry();
    for category_map in registry.values() {
        if let Some(skill) = category_map.get(name) {
            return Some(skill.clone());
        }
    }
    None
}

/// Retrieve a skill by its name within a specific category.
///
/// This function searches only within the specified category, which is more efficient
/// than searching across all categories when the category is known.
///
/// # Arguments
/// * `name` - The name of the skill to search for.
/// * `category` - The `SkillCategory` to search within.
///
/// # Returns
/// `Some(Arc<dyn Skill>)` if a skill with the given name exists in the specified category,
/// otherwise `None`.
///
/// # Examples
/// ```
/// use your_crate::core::get_skill_by_name_and_category;
/// use your_crate::registry::SkillCategory;
///
/// let skill = get_skill_by_name_and_category("hello_world", SkillCategory::Basic);
/// assert!(skill.is_some());
/// ```
pub fn get_skill_by_name_and_category(
    name: &str,
    category: SkillCategory,
) -> Option<Arc<dyn Skill>> {
    let registry = get_registry();
    registry
        .get(&category)
        .and_then(|map| map.get(name))
        .cloned()
}

/// Check if a skill exists in the registry by its name.
///
/// This function searches across all categories to determine if any skill
/// with the given name is registered.
///
/// # Arguments
/// * `name` - The name of the skill to check.
///
/// # Returns
/// `true` if a skill with the given name exists, otherwise `false`.
///
/// # Examples
/// ```
/// use your_crate::core::has_skill;
///
/// if has_skill("hello_world") {
///     println!("Skill is available!");
/// } else {
///     println!("Skill not found!");
/// }
/// ```
pub fn has_skill(name: &str) -> bool {
    get_skill_by_name(name).is_some()
}

/// List the names of all registered skills across all categories.
///
/// This function collects and returns a vector of all skill names in the registry.
///
/// # Returns
/// A `Vec<String>` containing the names of all registered skills.
///
/// # Examples
/// ```
/// use your_crate::core::list_skills_names;
///
/// let names = list_skills_names();
/// println!("All skill names: {:?}", names);
/// ```
pub fn list_skills_names() -> Vec<String> {
    let registry = get_registry();
    let mut names = Vec::new();
    for category_map in registry.values() {
        names.extend(category_map.keys().cloned());
    }
    names
}

/// List the names of all skills in a specific category.
///
/// # Arguments
/// * `category` - The `SkillCategory` to list skills from.
///
/// # Returns
/// A `Vec<String>` containing the names of all skills in the specified category.
/// Returns an empty vector if the category has no registered skills.
///
/// # Examples
/// ```
/// use your_crate::core::list_skills_name_by_category;
/// use your_crate::registry::SkillCategory;
///
/// let names = list_skills_name_by_category(SkillCategory::Basic);
/// println!("Basic skills: {:?}", names);
/// ```
pub fn list_skills_name_by_category(category: SkillCategory) -> Vec<String> {
    let registry = get_registry();
    registry
        .get(&category)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

/// Get all skill categories with the count of skills in each.
///
/// This function returns a list of all categories present in the registry,
/// along with the number of registered skills in each category.
///
/// # Returns
/// A `Vec<(String, usize)>` where each tuple contains:
/// - The category name as a string
/// - The number of skills in that category
///
/// The results are sorted alphabetically by category name.
///
/// # Examples
/// ```
/// use your_crate::core::get_skill_category;
///
/// let categories = get_skill_category();
/// for (name, count) in categories {
///     println!("Category '{}' has {} skills", name, count);
/// }
/// ```
pub fn get_skill_category() -> Vec<(String, usize)> {
    let registry = get_registry();
    let mut result = Vec::new();
    for (category, map) in registry.iter() {
        result.push((category.name().to_string(), map.len()));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Get the names of all skill category.
///
/// This function returns a list of all category names that are present in the registry.
/// The names are the machine-readable identifiers (e.g., "basic", "file", "math").
///
/// # Returns
/// A `Vec<String>` containing the names of all category, sorted alphabetically.
///
/// # Examples
/// ```
/// use your_crate::core::get_skill_category_names;
///
/// let category_names = get_skill_category_names();
/// println!("Available categories: {:?}", category_names);
/// ```
pub fn get_skill_category_names() -> Vec<String> {
    let registry = get_registry();
    let mut result: Vec<String> = registry
        .keys()
        .map(|category| category.name().to_string())
        .collect();
    result.sort();
    result
}

/// Get skill category names with their human-readable display names and descriptions.
///
/// This function returns a list of all categories, pairing each category's
/// machine-readable name with a combined description containing both the
/// display name and the detailed description of the category's purpose.
///
/// # Returns
/// A `Vec<(String, String)>` where each tuple contains:
/// - The category name as a string (machine-readable, e.g., "basic")
/// - A formatted description containing the display name and description
///   (e.g., "Basic Skills - Basic example skills for demonstration and testing")
///
/// The results are sorted alphabetically by category name.
///
/// # Examples
/// ```
/// use your_crate::core::get_skill_category_name_and_describe;
///
/// let categories = get_skill_category_name_and_describe();
/// for (name, description) in categories {
///     println!("{}: {}", name, description);
/// }
/// ```
pub fn get_skill_category_name_and_describe() -> Vec<(String, String)> {
    let registry = get_registry();
    let mut result = Vec::new();
    for category in registry.keys() {
        let name = category.name().to_string();
        let description = format!("{} - {}", category.display_name(), category.description());
        result.push((name, description));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Generate AI-friendly metadata for all registered skills.
///
/// This function collects metadata from all skills in the registry,
/// including name, category, description, and parameter schema.
///
/// # Returns
/// A vector of `SkillMetadata` for all registered skills.
///
/// # Examples
/// ```
/// use your_crate::core::generate_registry_skills_metadata;
///
/// let metadata = generate_registry_skills_metadata();
/// for skill in metadata {
///     println!("Skill: {} - {}", skill.name, skill.description);
/// }
/// ```
pub fn generate_registry_skills_metadata() -> Vec<SkillMetadata> {
    let registry = get_registry();
    let mut metadata = Vec::new();
    for category_map in registry.values() {
        for skill in category_map.values() {
            metadata.push(skill.get_metadata());
        }
    }
    metadata
}

/// Generate a JSON representation of the skill registry.
///
/// This function creates a comprehensive JSON object containing
/// version information, total skill count, and all skill metadata.
///
/// # Returns
/// A `serde_json::Value` containing the complete registry information.
///
/// # JSON Structure
/// ```json
/// {
///   "version": "1.0",
///   "total_skills": 50,
///   "skills": [...],
///   "instruction": "You can call a skill by returning a JSON object..."
/// }
/// ```
pub fn generate_total_skills_registry_table_json() -> Value {
    let metadata = generate_registry_skills_metadata();
    serde_json::json!({
        "version": "1.0",
        "total_skills": metadata.len(),
        "skills": metadata,
        "instruction": r#"You can call a skill by returning a JSON object with 'action' and 'parameters' fields. Example: {"action": "calculator", "parameters": {"expression": "2+3"}}"#
    })
}

/// Generate a pretty-printed JSON string of the skill registry.
///
/// This is convenient for logging, debugging, or sending to LLM APIs.
///
/// # Returns
/// A formatted JSON string containing the complete registry information.
///
/// # Panics
/// Panics if serialization fails.
pub fn generate_skill_registry_table_json_str() -> String {
    serde_json::to_string_pretty(&generate_total_skills_registry_table_json()).unwrap()
}

/// Retrieve all skills belonging to a specific category.
///
/// # Arguments
/// * `category` - The `SkillCategory` to filter by.
///
/// # Returns
/// A `Vec<Arc<dyn Skill>>` containing clones of all skills in the specified category.
/// Returns an empty vector if the category has no registered skills.
///
/// # Examples
/// ```
/// use your_crate::core::get_skills_by_category;
/// use your_crate::registry::SkillCategory;
///
/// let basic_skills = get_skills_by_category(SkillCategory::Basic);
/// for skill in basic_skills {
///     println!("Found basic skill: {}", skill.name());
/// }
/// ```
pub fn get_skills_by_category(category: SkillCategory) -> Vec<Arc<dyn Skill>> {
    let registry = get_registry();
    registry
        .get(&category)
        .map(|map| map.values().cloned().collect())
        .unwrap_or_default()
}

/// Get skills by multiple categories.
///
/// This function filters skills that belong to any of the specified categories.
///
/// # Arguments
/// * `categories` - A slice of category name strings to filter by.
///
/// # Returns
/// A vector of `Arc<dyn Skill>` matching any of the specified categories.
///
/// # Examples
/// ```
/// let category = vec!["basic".to_string(), "math".to_string()];
/// let skills = get_skills_by_category_list(&category);
/// println!("Found {} skills", skills.len());
/// ```
pub fn get_skills_by_category_list(categorys: &[String]) -> Vec<Arc<dyn Skill>> {
    let registry = get_registry();
    let mut result = Vec::new();
    for category_map in registry.values() {
        for skill in category_map.values() {
            let skill_category = skill.category().name();
            if categorys.iter().any(|cat| cat == skill_category) {
                result.push(skill.clone());
            }
        }
    }
    result
}

/// List skill names by multiple category.
///
/// This function collects names of skills that belong to any of the specified category.
///
/// # Arguments
/// * `category` - A slice of category name strings to filter by.
///
/// # Returns
/// A vector of skill names matching any of the specified category.
///
/// # Examples
/// ```
/// let category = vec!["basic".to_string(), "math".to_string()];
/// let names = list_skills_name_by_category_list(&category);
/// println!("Skills in basic or math: {:?}", names);
/// ```
pub fn list_skills_name_by_category_list(categorys: &[String]) -> Vec<String> {
    let registry = get_registry();
    let mut result = Vec::new();
    for (category, category_map) in registry.iter() {
        let category_name = category.name().to_string();
        if categorys.iter().any(|cat| cat == &category_name) {
            result.extend(category_map.keys().cloned());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_skill_category() {
        let category = get_skill_category_name_and_describe();
        println!("category:{:?}", category);
    }
}
