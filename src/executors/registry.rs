/// # Skill Registry Module
///
/// This module provides a central registry for managing all available skills in the system.
/// It maintains a thread-safe, global mapping from skill names to their implementations.
/// Skills can be registered, retrieved, and listed, and the registry can generate
/// AI-friendly metadata for LLM integration.
use crate::executors::Skill;
use crate::executors::types::SkillMetadata;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

/// Global, lazily-initialized, thread-safe registry of all available skills.
/// Uses a read-write lock to allow concurrent reads and exclusive writes.
/// The registry is stored as a HashMap mapping skill names (String) to
/// atomic reference-counted pointers to trait objects implementing the Skill trait.
static SKILL_REGISTRY: Lazy<RwLock<HashMap<String, Arc<dyn Skill>>>> = Lazy::new(|| {
    let mut registry: HashMap<String, Arc<dyn Skill>> = HashMap::new();
    // ==================== Basic Skills ====================
    registry.insert(
        "helloworld".to_string(),
        Arc::new(super::skills::HelloWorldSkill) as Arc<dyn Skill>,
    );
    // ==================== File System Skills ====================
    registry.insert(
        "file_read".to_string(),
        Arc::new(super::skills::file::ReadFileSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "file_write".to_string(),
        Arc::new(super::skills::file::WriteFileSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "file_delete".to_string(),
        Arc::new(super::skills::file::DeleteFileSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "file_list".to_string(),
        Arc::new(super::skills::file::ListDirectorySkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "file_copy".to_string(),
        Arc::new(super::skills::file::CopyFileSkill) as Arc<dyn Skill>,
    );
    // ==================== Mathematics Skills ====================
    registry.insert(
        "math_calculator".to_string(),
        Arc::new(super::skills::CalculatorSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "math_power".to_string(),
        Arc::new(super::skills::PowerSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "math_statistics".to_string(),
        Arc::new(super::skills::StatisticsSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "math_unit_converter".to_string(),
        Arc::new(super::skills::UnitConverterSkill) as Arc<dyn Skill>,
    );
    // ==================== Time Skills ====================
    registry.insert(
        "time_datetime".to_string(),
        Arc::new(super::skills::DateTimeSkill) as Arc<dyn Skill>,
    );
    // ==================== Network Skills ====================
    registry.insert(
        "net_httprequest".to_string(),
        Arc::new(super::skills::HttpRequestSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "read_url".to_string(),
        Arc::new(super::skills::ReadUrlSkill) as Arc<dyn Skill>,
    );
    // ==================== System Skills ====================
    registry.insert(
        "system_systeminfo".to_string(),
        Arc::new(super::skills::SystemInfoSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "exec_command".to_string(),
        Arc::new(super::skills::ExecCommandSkill) as Arc<dyn Skill>,
    );
    // ==================== Document Skills ====================
    registry.insert(
        "markdown_read".to_string(),
        Arc::new(super::skills::document::MarkdownReadSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "markdown_write".to_string(),
        Arc::new(super::skills::document::MarkdownWriteSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "csv_read".to_string(),
        Arc::new(super::skills::document::CsvReadSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "csv_write".to_string(),
        Arc::new(super::skills::document::CsvWriteSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "xml_parse".to_string(),
        Arc::new(super::skills::document::XmlParseSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "xml_to_json".to_string(),
        Arc::new(super::skills::document::XmlToJsonSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "excel_read".to_string(),
        Arc::new(super::skills::document::ExcelReadSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "excel_write".to_string(),
        Arc::new(super::skills::document::ExcelWriteSkill) as Arc<dyn Skill>,
    );
    // ==================== Messaging Skills ====================
    registry.insert(
        "send_email".to_string(),
        Arc::new(super::skills::message::SendEmailSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "send_telegram".to_string(),
        Arc::new(super::skills::message::SendTelegramSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "send_dingding".to_string(),
        Arc::new(super::skills::message::SendDingDingSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "send_feishu".to_string(),
        Arc::new(super::skills::message::SendFeishuSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "send_wecom".to_string(),
        Arc::new(super::skills::message::SendWecomSkill) as Arc<dyn Skill>,
    );
    // ==================== FTP Skills ====================
    registry.insert(
        "ftp_upload".to_string(),
        Arc::new(super::skills::ftp::FtpUploadSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "ftp_download".to_string(),
        Arc::new(super::skills::ftp::FtpDownloadSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "ftp_list".to_string(),
        Arc::new(super::skills::ftp::FtpListSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "ftp_delete".to_string(),
        Arc::new(super::skills::ftp::FtpDeleteSkill) as Arc<dyn Skill>,
    );
    // ==================== TCP Skills ====================
    registry.insert(
        "tcp_send".to_string(),
        Arc::new(super::skills::tcp::TcpSendSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "tcp_receive".to_string(),
        Arc::new(super::skills::tcp::TcpReceiveSkill) as Arc<dyn Skill>,
    );
    // ==================== UDP Skills ====================
    registry.insert(
        "udp_send".to_string(),
        Arc::new(super::skills::udp::UdpSendSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "udp_receive".to_string(),
        Arc::new(super::skills::udp::UdpReceiveSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "udp_broadcast".to_string(),
        Arc::new(super::skills::udp::UdpBroadcastSkill) as Arc<dyn Skill>,
    );
    // ==================== PostgreSQL Skills ====================
    registry.insert(
        "postgres_query".to_string(),
        Arc::new(super::skills::postgresql::PostgresQuerySkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "postgres_execute".to_string(),
        Arc::new(super::skills::postgresql::PostgresExecuteSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "postgres_list_tables".to_string(),
        Arc::new(super::skills::postgresql::PostgresListTablesSkill) as Arc<dyn Skill>,
    );
    // ==================== MySQL Skills ====================
    registry.insert(
        "mysql_query".to_string(),
        Arc::new(super::skills::mysql::MysqlQuerySkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "mysql_execute".to_string(),
        Arc::new(super::skills::mysql::MysqlExecuteSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "mysql_list_tables".to_string(),
        Arc::new(super::skills::mysql::MysqlListTablesSkill) as Arc<dyn Skill>,
    );
    // ==================== Redis Skills ====================
    registry.insert(
        "redis_set".to_string(),
        Arc::new(super::skills::redis::RedisSetSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "redis_get".to_string(),
        Arc::new(super::skills::redis::RedisGetSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "redis_del".to_string(),
        Arc::new(super::skills::redis::RedisDelSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "redis_keys".to_string(),
        Arc::new(super::skills::redis::RedisKeysSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "redis_hset".to_string(),
        Arc::new(super::skills::redis::RedisHSetSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "redis_hget".to_string(),
        Arc::new(super::skills::redis::RedisHGetSkill) as Arc<dyn Skill>,
    );
    // ==================== SQLite Skills ====================
    registry.insert(
        "sqlite_query".to_string(),
        Arc::new(super::skills::sqlite::SqliteQuerySkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "sqlite_execute".to_string(),
        Arc::new(super::skills::sqlite::SqliteExecuteSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "sqlite_list_tables".to_string(),
        Arc::new(super::skills::sqlite::SqliteListTablesSkill) as Arc<dyn Skill>,
    );
    // ==================== GitHub Skills ====================
    registry.insert(
        "github_get_repo".to_string(),
        Arc::new(super::skills::github::GithubGetRepo) as Arc<dyn Skill>,
    );
    registry.insert(
        "github_create_issue".to_string(),
        Arc::new(super::skills::github::GithubCreateIssue) as Arc<dyn Skill>,
    );
    registry.insert(
        "github_list_issues".to_string(),
        Arc::new(super::skills::github::GithubListIssues) as Arc<dyn Skill>,
    );
    registry.insert(
        "github_star_repo".to_string(),
        Arc::new(super::skills::github::GithubStarRepo) as Arc<dyn Skill>,
    );
    registry.insert(
        "github_search_repos".to_string(),
        Arc::new(super::skills::github::GithubSearchRepos) as Arc<dyn Skill>,
    );
    registry.insert(
        "github_get_user".to_string(),
        Arc::new(super::skills::github::GithubGetUser) as Arc<dyn Skill>,
    );
    registry.insert(
        "github_list_prs".to_string(),
        Arc::new(super::skills::github::GithubListPRs) as Arc<dyn Skill>,
    );
    // ==================== Clipboard Skills ====================
    registry.insert(
        "clipboard_get".to_string(),
        Arc::new(super::skills::system::clipboard::ClipboardGetSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "clipboard_set".to_string(),
        Arc::new(super::skills::system::clipboard::ClipboardSetSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "clipboard_clear".to_string(),
        Arc::new(super::skills::system::clipboard::ClipboardClearSkill) as Arc<dyn Skill>,
    );
    // ==================== Scheduler Skills ====================
    registry.insert(
        "schedule_task".to_string(),
        Arc::new(super::skills::task::ScheduleTaskSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "unschedule_task".to_string(),
        Arc::new(super::skills::task::UnscheduleTaskSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "list_scheduled_tasks".to_string(),
        Arc::new(super::skills::task::ListScheduledTasksSkill) as Arc<dyn Skill>,
    );
    RwLock::new(registry)
});

/// Generates a list of metadata for all registered skills.
/// This is useful for AI systems that need to understand available capabilities.
///
/// # Returns
/// A vector of `SkillMetadata` containing information about each registered skill,
/// including its name, category, description, and parameter schema.
///
/// # Example
/// ```
/// let metadata = generate_ai_registry();
/// for skill in metadata {
///     println!("Skill: {} - {}", skill.name, skill.description);
/// }
/// ```
pub fn generate_ai_registry() -> Vec<SkillMetadata> {
    let registry = get_registry();
    registry
        .values()
        .map(|skill| skill.get_metadata())
        .collect()
}

/// Generates a comprehensive JSON representation of the skill registry.
/// This includes version information, total skill count, and all skill metadata,
/// along with instructions for AI systems on how to invoke skills.
///
/// # Returns
/// A `serde_json::Value` containing the complete registry information in JSON format.
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
pub fn generate_skill_registry_table_json() -> Value {
    let metadata = generate_ai_registry();
    serde_json::json!({
        "version": "1.0",
        "total_skills": metadata.len(),
        "skills": metadata,
        "instruction": r#"You can call a skill by returning a JSON object with 'action' and 'parameters' fields. Example: {"action": "calculator", "parameters": {"expression": "2+3"}}"#
    })
}

/// Generates a pretty-printed JSON string representation of the skill registry.
/// This is convenient for logging, debugging, or sending to LLM APIs.
///
/// # Returns
/// A formatted JSON string containing the complete registry information.
/// If serialization fails, the function will panic (which is expected in normal operation).
pub fn generate_skill_registry_table_json_str() -> String {
    serde_json::to_string_pretty(&generate_skill_registry_table_json()).unwrap()
}

/// Acquires a read lock on the global skill registry and returns a guard.
/// This allows concurrent read access to the registry.
///
/// # Returns
/// A read guard that provides access to the underlying HashMap.
///
/// # Panics
/// Will panic if the lock is poisoned (which should not happen under normal operation).
pub fn get_registry() -> std::sync::RwLockReadGuard<'static, HashMap<String, Arc<dyn Skill>>> {
    SKILL_REGISTRY.read().unwrap()
}

/// Acquires a write lock on the global skill registry and returns a guard.
/// This allows exclusive write access to the registry.
///
/// # Returns
/// A write guard that provides mutable access to the underlying HashMap.
///
/// # Panics
/// Will panic if the lock is poisoned (which should not happen under normal operation).
pub fn get_registry_mut() -> std::sync::RwLockWriteGuard<'static, HashMap<String, Arc<dyn Skill>>> {
    SKILL_REGISTRY.write().unwrap()
}

/// Retrieves a skill by name from the registry.
///
/// # Arguments
/// * `name` - The name of the skill to retrieve (e.g., "file_read", "math_calculator")
///
/// # Returns
/// An `Option` containing an `Arc<dyn Skill>` if the skill exists, otherwise `None`.
///
/// # Example
/// ```
/// if let Some(skill) = get_skill("file_read") {
///     println!("Skill found!");
/// }
/// ```
pub fn get_skill(name: &str) -> Option<Arc<dyn Skill>> {
    get_registry().get(name).cloned()
}

/// Dynamically registers a new skill into the global registry.
/// This allows runtime addition of skills after the initial registry initialization.
///
/// # Arguments
/// * `name` - The unique name to associate with the skill
/// * `skill` - An atomic reference-counted pointer to the skill implementation
///
/// # Example
/// ```
/// let my_skill = Arc::new(MyCustomSkill);
/// register_skill("my_custom_skill".to_string(), my_skill);
/// ```
pub fn register_skill(name: String, skill: Arc<dyn Skill>) {
    get_registry_mut().insert(name, skill);
}

/// Checks whether a skill with the given name exists in the registry.
///
/// # Arguments
/// * `name` - The name of the skill to check
///
/// # Returns
/// `true` if the skill exists, `false` otherwise.
pub fn has_skill(name: &str) -> bool {
    get_registry().contains_key(name)
}

/// Returns a list of all registered skill names.
///
/// # Returns
/// A vector containing the names of all skills in the registry.
pub fn list_skills() -> Vec<String> {
    get_registry().keys().cloned().collect()
}

#[cfg(test)]
mod registry_test {
    use super::*;

    /// Test that the AI registry generation returns metadata for all skills
    #[test]
    fn test_generate_ai_registry() {
        let metadata = generate_ai_registry();
        println!("Total skills: {}", metadata.len());
        assert!(
            metadata.len() > 50,
            "Expected at least 50 skills, got {}",
            metadata.len()
        );
        for skill in &metadata {
            println!(
                "  - {} ({}): {}",
                skill.name, skill.category, skill.description
            );
            assert!(!skill.name.is_empty(), "Skill name should not be empty");
            assert!(
                !skill.category.is_empty(),
                "Skill category should not be empty"
            );
        }
        let skill_names: Vec<&str> = metadata.iter().map(|s| s.name.as_str()).collect();
        assert!(
            skill_names.contains(&"file_read"),
            "file_read skill should be present"
        );
        assert!(
            skill_names.contains(&"math_calculator"),
            "math_calculator skill should be present"
        );
    }

    /// Test that the registry JSON generation produces valid output
    #[test]
    fn test_print_all_skill_json() {
        let json_value = generate_skill_registry_table_json();
        println!("Registry JSON: {:?}", json_value);
        assert!(
            json_value["version"].is_string(),
            "version field should be a string"
        );
        assert_eq!(json_value["version"].as_str().unwrap(), "1.0");
        assert!(
            json_value["total_skills"].is_u64(),
            "total_skills field should be a number"
        );
        assert!(
            json_value["total_skills"].as_u64().unwrap() > 0,
            "total_skills should be positive"
        );
        assert!(
            json_value["skills"].is_array(),
            "skills field should be an array"
        );
        assert!(
            json_value["instruction"].is_string(),
            "instruction field should be a string"
        );
        let skills_array = json_value["skills"].as_array().unwrap();
        assert_eq!(
            skills_array.len(),
            json_value["total_skills"].as_u64().unwrap() as usize,
            "skills array length should match total_skills"
        );
    }

    /// Test registry operations: get, has, list, and register
    #[test]
    fn test_registry_operations() {
        let file_read_skill = get_skill("file_read");
        assert!(file_read_skill.is_some(), "file_read skill should exist");
        let non_existent = get_skill("non_existent_skill_12345");
        assert!(
            non_existent.is_none(),
            "Non-existent skill should return None"
        );
        assert!(
            has_skill("file_read"),
            "has_skill should return true for existing skill"
        );
        assert!(
            !has_skill("non_existent_skill_12345"),
            "has_skill should return false for non-existent skill"
        );
        let all_skills = list_skills();
        assert!(
            all_skills.contains(&"file_read".to_string()),
            "list_skills should include file_read"
        );
        assert!(
            all_skills.contains(&"math_calculator".to_string()),
            "list_skills should include math_calculator"
        );
        let skill_count_before = list_skills().len();
        assert!(skill_count_before > 0, "Registry should have skills");
    }

    /// Test that metadata is consistent across different retrieval methods
    #[test]
    fn test_metadata_consistency() {
        let registry_guard = get_registry();
        for (name, skill) in registry_guard.iter() {
            let metadata = skill.get_metadata();
            assert_eq!(
                &metadata.name, name,
                "Skill metadata name should match registry key"
            );
            assert!(
                !metadata.description.is_empty(),
                "Skill {} should have a description",
                name
            );
            assert!(
                !metadata.category.is_empty(),
                "Skill {} should have a category",
                name
            );
        }
    }
}
