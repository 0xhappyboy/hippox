//! Core driver registry implementation

use crate::Driver;
use crate::DriverMetadata;
use crate::registry::DriverCategory;
#[cfg(any(feature = "cryptography", feature = "all"))]
use crate::registry::cryptography_register;
#[cfg(any(feature = "email", feature = "all"))]
use crate::registry::email_register;
#[cfg(any(feature = "network", feature = "all"))]
use crate::registry::network_register;
#[cfg(any(feature = "scheduled_tasks", feature = "all"))]
use crate::registry::scheduled_tasks_register;
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
#[cfg(any(feature = "database", feature = "all"))]
use crate::registry::database_register;
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
#[cfg(any(feature = "operating_system_memory", feature = "all"))]
use crate::registry::operating_system_memory_register;
#[cfg(any(feature = "operating_system_process", feature = "all"))]
use crate::registry::operating_system_process_register;
#[cfg(any(feature = "operating_system", feature = "all"))]
use crate::registry::operating_system_register;
#[cfg(any(feature = "operating_system_security", feature = "all"))]
use crate::registry::operating_system_security_register;
#[cfg(any(feature = "operating_system_services", feature = "all"))]
use crate::registry::operating_system_services_register;
#[cfg(any(feature = "social_platform", feature = "all"))]
use crate::registry::socialplatform_register;
#[cfg(any(feature = "speech_speak", feature = "all"))]
use crate::registry::speech_speak_register;
#[cfg(any(feature = "terminal_commands", feature = "all"))]
use crate::registry::terminal_register;
#[cfg(any(feature = "text", feature = "all"))]
use crate::registry::text_register;
#[cfg(any(feature = "wifi", feature = "all"))]
use crate::registry::wifi_register;
#[cfg(any(feature = "window_control", feature = "all"))]
use crate::registry::window_register;

/// Global registry type: mapping from category to (driver_name -> driver_impl)
pub type DriverRegistryMap = HashMap<DriverCategory, HashMap<String, Arc<dyn Driver>>>;

/// Global, lazily-initialized, thread-safe registry of all available drivers.
static SKILL_REGISTRY: Lazy<RwLock<DriverRegistryMap>> = Lazy::new(|| {
    let mut registry: DriverRegistryMap = HashMap::new();
    #[cfg(any(feature = "helloworld", feature = "all"))]
    basic_register::register(&mut registry);
    #[cfg(any(feature = "file", feature = "all"))]
    file_register::register(&mut registry);
    #[cfg(any(feature = "math", feature = "all"))]
    math_register::register(&mut registry);
    #[cfg(any(feature = "network", feature = "all"))]
    network_register::register(&mut registry);
    #[cfg(any(feature = "operating_system", feature = "all"))]
    operating_system_register::register(&mut registry);
    #[cfg(any(feature = "document", feature = "all"))]
    document_register::register(&mut registry);
    #[cfg(any(feature = "social_platform", feature = "all"))]
    socialplatform_register::register(&mut registry);
    #[cfg(any(feature = "database", feature = "all"))]
    database_register::register(&mut registry);
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
    speech_speak_register::register(&mut registry);
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
    #[cfg(any(feature = "operating_system_services", feature = "all"))]
    operating_system_services_register::register(&mut registry);
    #[cfg(any(feature = "operating_system_security", feature = "all"))]
    operating_system_security_register::register(&mut registry);
    #[cfg(any(feature = "scheduled_tasks", feature = "all"))]
    scheduled_tasks_register::register(&mut registry);
    #[cfg(any(feature = "operating_system_process", feature = "all"))]
    operating_system_process_register::register(&mut registry);
    #[cfg(any(feature = "operating_system_memory", feature = "all"))]
    operating_system_memory_register::register(&mut registry);

    RwLock::new(registry)
});

/// Get read lock on the global driver registry.
///
/// This function provides read-only access to the registry, allowing multiple
/// concurrent readers. It panics if the registry lock is poisoned.
///
/// # Returns
/// A read guard that provides immutable access to the `DriverRegistryMap`.
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
pub fn get_registry() -> std::sync::RwLockReadGuard<'static, DriverRegistryMap> {
    SKILL_REGISTRY.read().unwrap()
}

/// Get write lock on the global driver registry.
///
/// This function provides exclusive write access to the registry, allowing
/// modification of registered drivers. It panics if the registry lock is poisoned.
///
/// # Returns
/// A write guard that provides mutable access to the `DriverRegistryMap`.
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
pub fn get_registry_mut() -> std::sync::RwLockWriteGuard<'static, DriverRegistryMap> {
    SKILL_REGISTRY.write().unwrap()
}

/// Get all category names that have at least one registered driver.
///
/// # Returns
/// A vector of category names (machine-readable strings) sorted alphabetically.
///
/// # Examples
/// ```
/// let categories = get_all_categorys();
/// println!("Available categories: {:?}", categories);
/// ```
pub fn get_all_categorys() -> Vec<String> {
    let registry = get_registry();
    let mut result: Vec<String> = registry
        .keys()
        .map(|category| category.name().to_string())
        .collect();
    result.sort();
    result
}

/// Register a new driver dynamically at runtime.
///
/// This function inserts or updates a driver in the global registry under the
/// specified category. If the category doesn't exist, it will be created automatically.
/// If a driver with the same name already exists in the category, it will be overwritten.
///
/// # Arguments
/// * `category` - The `DriverCategory` under which to register the driver.
/// * `name` - The unique name identifier for the driver.
/// * `driver` - An `Arc<dyn Driver>` trait object implementing the driver logic.
///
/// # Examples
/// ```
/// use std::sync::Arc;
/// use your_crate::core::register_driver;
/// use your_crate::registry::DriverCategory;
///
/// let driver = Arc::new(MyDriver::new());
/// register_driver(DriverCategory::Basic, "my_driver".to_string(), driver);
/// ```
pub fn register_driver(category: DriverCategory, name: String, driver: Arc<dyn Driver>) {
    let mut registry = get_registry_mut();
    registry
        .entry(category)
        .or_insert_with(HashMap::new)
        .insert(name, driver);
}

/// Retrieve all registered drivers across all categories.
///
/// This function collects and returns a vector containing every driver from
/// every category in the registry.
///
/// # Returns
/// A `Vec<Arc<dyn Driver>>` containing clones of all driver trait objects.
///
/// # Examples
/// ```
/// use your_crate::core::get_all_drivers;
///
/// let all_drivers = get_all_drivers();
/// println!("Total drivers available: {}", all_drivers.len());
/// ```
pub fn get_all_drivers() -> Vec<Arc<dyn Driver>> {
    let registry = get_registry();
    let mut drivers = Vec::new();
    for category_map in registry.values() {
        for driver in category_map.values() {
            drivers.push(driver.clone());
        }
    }
    drivers
}

/// Retrieve a driver by its name, searching across all categories.
///
/// This function performs a linear search through all categories to find a driver
/// with the matching name. If multiple drivers share the same name across different
/// categories, only the first one found is returned.
///
/// # Arguments
/// * `name` - The name of the driver to search for.
///
/// # Returns
/// `Some(Arc<dyn Driver>)` if a driver with the given name exists, otherwise `None`.
///
/// # Examples
/// ```
/// use your_crate::core::get_driver_by_name;
///
/// if let Some(driver) = get_driver_by_name("hello_world") {
///     println!("Found driver: {}", driver.name());
/// } else {
///     println!("Driver not found");
/// }
/// ```
pub fn get_driver_by_name(name: &str) -> Option<Arc<dyn Driver>> {
    let registry = get_registry();
    for category_map in registry.values() {
        if let Some(driver) = category_map.get(name) {
            return Some(driver.clone());
        }
    }
    None
}

/// Retrieve a driver by its name within a specific category.
///
/// This function searches only within the specified category, which is more efficient
/// than searching across all categories when the category is known.
///
/// # Arguments
/// * `name` - The name of the driver to search for.
/// * `category` - The `DriverCategory` to search within.
///
/// # Returns
/// `Some(Arc<dyn Driver>)` if a driver with the given name exists in the specified category,
/// otherwise `None`.
///
/// # Examples
/// ```
/// use your_crate::core::get_driver_by_name_and_category;
/// use your_crate::registry::DriverCategory;
///
/// let driver = get_driver_by_name_and_category("hello_world", DriverCategory::Basic);
/// assert!(driver.is_some());
/// ```
pub fn get_driver_by_name_and_category(
    name: &str,
    category: DriverCategory,
) -> Option<Arc<dyn Driver>> {
    let registry = get_registry();
    registry
        .get(&category)
        .and_then(|map| map.get(name))
        .cloned()
}

/// Check if a driver exists in the registry by its name.
///
/// This function searches across all categories to determine if any driver
/// with the given name is registered.
///
/// # Arguments
/// * `name` - The name of the driver to check.
///
/// # Returns
/// `true` if a driver with the given name exists, otherwise `false`.
///
/// # Examples
/// ```
/// use your_crate::core::has_driver;
///
/// if has_driver("hello_world") {
///     println!("Driver is available!");
/// } else {
///     println!("Driver not found!");
/// }
/// ```
pub fn has_driver(name: &str) -> bool {
    get_driver_by_name(name).is_some()
}

/// List the names of all registered drivers across all categories.
///
/// This function collects and returns a vector of all driver names in the registry.
///
/// # Returns
/// A `Vec<String>` containing the names of all registered drivers.
///
/// # Examples
/// ```
/// use your_crate::core::list_drivers_names;
///
/// let names = list_drivers_names();
/// println!("All driver names: {:?}", names);
/// ```
pub fn list_drivers_names() -> Vec<String> {
    let registry = get_registry();
    let mut names = Vec::new();
    for category_map in registry.values() {
        names.extend(category_map.keys().cloned());
    }
    names
}

/// List the names of all drivers in a specific category.
///
/// # Arguments
/// * `category` - The `DriverCategory` to list drivers from.
///
/// # Returns
/// A `Vec<String>` containing the names of all drivers in the specified category.
/// Returns an empty vector if the category has no registered drivers.
///
/// # Examples
/// ```
/// use your_crate::core::list_drivers_name_by_category;
/// use your_crate::registry::DriverCategory;
///
/// let names = list_drivers_name_by_category(DriverCategory::Basic);
/// println!("Basic drivers: {:?}", names);
/// ```
pub fn list_drivers_name_by_category(category: DriverCategory) -> Vec<String> {
    let registry = get_registry();
    registry
        .get(&category)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

/// Get all driver categories with the count of drivers in each.
///
/// This function returns a list of all categories present in the registry,
/// along with the number of registered drivers in each category.
///
/// # Returns
/// A `Vec<(String, usize)>` where each tuple contains:
/// - The category name as a string
/// - The number of drivers in that category
///
/// The results are sorted alphabetically by category name.
///
/// # Examples
/// ```
/// use your_crate::core::get_driver_category;
///
/// let categories = get_driver_category();
/// for (name, count) in categories {
///     println!("Category '{}' has {} drivers", name, count);
/// }
/// ```
pub fn get_driver_category() -> Vec<(String, usize)> {
    let registry = get_registry();
    let mut result = Vec::new();
    for (category, map) in registry.iter() {
        result.push((category.name().to_string(), map.len()));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Get the names of all driver category.
///
/// This function returns a list of all category names that are present in the registry.
/// The names are the machine-readable identifiers (e.g., "basic", "file", "math").
///
/// # Returns
/// A `Vec<String>` containing the names of all category, sorted alphabetically.
///
/// # Examples
/// ```
/// use your_crate::core::get_driver_category_names;
///
/// let category_names = get_driver_category_names();
/// println!("Available categories: {:?}", category_names);
/// ```
pub fn get_driver_category_names() -> Vec<String> {
    let registry = get_registry();
    let mut result: Vec<String> = registry
        .keys()
        .map(|category| category.name().to_string())
        .collect();
    result.sort();
    result
}

/// Get driver category names with their human-readable display names and descriptions.
///
/// This function returns a list of all categories, pairing each category's
/// machine-readable name with a combined description containing both the
/// display name and the detailed description of the category's purpose.
///
/// # Returns
/// A `Vec<(String, String)>` where each tuple contains:
/// - The category name as a string (machine-readable, e.g., "basic")
/// - A formatted description containing the display name and description
///   (e.g., "Basic Drivers - Basic example drivers for demonstration and testing")
///
/// The results are sorted alphabetically by category name.
///
/// # Examples
/// ```
/// use your_crate::core::get_driver_category_name_and_describe;
///
/// let categories = get_driver_category_name_and_describe();
/// for (name, description) in categories {
///     println!("{}: {}", name, description);
/// }
/// ```
pub fn get_driver_category_name_and_describe() -> Vec<(String, String)> {
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

/// Generate AI-friendly metadata for all registered drivers.
///
/// This function collects metadata from all drivers in the registry,
/// including name, category, description, and parameter schema.
///
/// # Returns
/// A vector of `DriverMetadata` for all registered drivers.
///
/// # Examples
/// ```
/// use your_crate::core::generate_registry_drivers_metadata;
///
/// let metadata = generate_registry_drivers_metadata();
/// for driver in metadata {
///     println!("Driver: {} - {}", driver.name, driver.description);
/// }
/// ```
pub fn generate_registry_drivers_metadata() -> Vec<DriverMetadata> {
    let registry = get_registry();
    let mut metadata = Vec::new();
    for category_map in registry.values() {
        for driver in category_map.values() {
            metadata.push(driver.get_metadata());
        }
    }
    metadata
}

/// Generate a JSON representation of the driver registry.
///
/// This function creates a comprehensive JSON object containing
/// version information, total driver count, and all driver metadata.
///
/// # Returns
/// A `serde_json::Value` containing the complete registry information.
///
/// # JSON Structure
/// ```json
/// {
///   "version": "1.0",
///   "total_drivers": 50,
///   "drivers": [...],
///   "instruction": "You can call a driver by returning a JSON object..."
/// }
/// ```
pub fn generate_total_drivers_registry_table_json() -> Value {
    let metadata = generate_registry_drivers_metadata();
    serde_json::json!({
        "version": "1.0",
        "total_drivers": metadata.len(),
        "drivers": metadata,
        "instruction": r#"You can call a driver by returning a JSON object with 'action' and 'parameters' fields. Example: {"action": "calculator", "parameters": {"expression": "2+3"}}"#
    })
}

/// Generate a pretty-printed JSON string of the driver registry.
///
/// This is convenient for logging, debugging, or sending to LLM APIs.
///
/// # Returns
/// A formatted JSON string containing the complete registry information.
///
/// # Panics
/// Panics if serialization fails.
pub fn generate_driver_registry_table_json_str() -> String {
    serde_json::to_string_pretty(&generate_total_drivers_registry_table_json()).unwrap()
}

/// Retrieve all drivers belonging to a specific category.
///
/// # Arguments
/// * `category` - The `DriverCategory` to filter by.
///
/// # Returns
/// A `Vec<Arc<dyn Driver>>` containing clones of all drivers in the specified category.
/// Returns an empty vector if the category has no registered drivers.
///
/// # Examples
/// ```
/// use your_crate::core::get_drivers_by_category;
/// use your_crate::registry::DriverCategory;
///
/// let basic_drivers = get_drivers_by_category("basic");
/// for driver in basic_drivers {
///     println!("Found basic driver: {}", driver.name());
/// }
/// ```
pub fn get_drivers_by_category(category: &str) -> Vec<Arc<dyn Driver>> {
    let Some(cat_enum) = DriverCategory::from_str(category) else {
        return Vec::new();
    };
    let registry = get_registry();
    registry
        .get(&cat_enum)
        .map(|map| map.values().cloned().collect())
        .unwrap_or_default()
}

/// Get drivers by multiple categories.
///
/// This function filters drivers that belong to any of the specified categories.
///
/// # Arguments
/// * `categories` - A slice of category name strings to filter by.
///
/// # Returns
/// A vector of `Arc<dyn Driver>` matching any of the specified categories.
///
/// # Examples
/// ```
/// let category = vec!["basic".to_string(), "math".to_string()];
/// let drivers = get_drivers_by_category_list(&category);
/// println!("Found {} drivers", drivers.len());
/// ```
pub fn get_drivers_by_category_list(categories: &[String]) -> Vec<Arc<dyn Driver>> {
    let registry = get_registry();
    let mut result = Vec::new();
    let enums: Vec<DriverCategory> = categories
        .iter()
        .filter_map(|cat| DriverCategory::from_str(cat))
        .collect();
    for cat_enum in enums {
        if let Some(driver_map) = registry.get(&cat_enum) {
            result.extend(driver_map.values().cloned());
        }
    }
    result
}

/// List driver names by multiple category.
///
/// This function collects names of drivers that belong to any of the specified category.
///
/// # Arguments
/// * `category` - A slice of category name strings to filter by.
///
/// # Returns
/// A vector of driver names matching any of the specified category.
///
/// # Examples
/// ```
/// let category = vec!["basic".to_string(), "math".to_string()];
/// let names = list_drivers_name_by_category_list(&category);
/// println!("Drivers in basic or math: {:?}", names);
/// ```
pub fn list_drivers_name_by_category_list(categorys: &[String]) -> Vec<String> {
    let registry = get_registry();
    let mut result = Vec::new();
    let enums: Vec<DriverCategory> = categorys
        .iter()
        .filter_map(|cat| DriverCategory::from_str(cat))
        .collect();
    for cat_enum in enums {
        if let Some(driver_map) = registry.get(&cat_enum) {
            result.extend(driver_map.keys().cloned());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_driver_category() {
        let category = get_driver_category_name_and_describe();
        println!("category:{:?}", category);
    }
}
