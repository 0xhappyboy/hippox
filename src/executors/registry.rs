use crate::executors::Skill;
use crate::executors::types::SkillMetadata;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

static SKILL_REGISTRY: Lazy<RwLock<HashMap<String, Arc<dyn Skill>>>> = Lazy::new(|| {
    let mut registry: HashMap<String, Arc<dyn Skill>> = HashMap::new();
    registry.insert(
        "helloworld".to_string(),
        Arc::new(super::skills::HelloWorldSkill) as Arc<dyn Skill>,
    );
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
    registry.insert(
        "time_datetime".to_string(),
        Arc::new(super::skills::DateTimeSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "net_httprequest".to_string(),
        Arc::new(super::skills::HttpRequestSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "system_systeminfo".to_string(),
        Arc::new(super::skills::SystemInfoSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "exec_command".to_string(),
        Arc::new(super::skills::ExecCommandSkill) as Arc<dyn Skill>,
    );
    registry.insert(
        "read_url".to_string(),
        Arc::new(super::skills::ReadUrlSkill) as Arc<dyn Skill>,
    );
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
    RwLock::new(registry)
});

/// Generate AI registry JSON for LLM
pub fn generate_ai_registry() -> Vec<SkillMetadata> {
    let registry = get_registry();
    registry
        .values()
        .map(|skill| skill.get_metadata())
        .collect()
}

/// generate skill registry table json
pub fn generate_skill_registry_table_json() -> Value {
    let metadata = generate_ai_registry();
    serde_json::json!({
        "version": "1.0",
        "total_skills": metadata.len(),
        "skills": metadata,
        "instruction": r#"You can call a skill by returning a JSON object with 'action' and 'parameters' fields. Example: {"action": "calculator", "parameters": {"expression": "2+3"}}"#
    })
}

/// generate skill registry table json string
pub fn generate_skill_registry_table_json_str() -> String {
    serde_json::to_string_pretty(&generate_skill_registry_table_json()).unwrap()
}

pub fn get_registry() -> std::sync::RwLockReadGuard<'static, HashMap<String, Arc<dyn Skill>>> {
    SKILL_REGISTRY.read().unwrap()
}

pub fn get_registry_mut() -> std::sync::RwLockWriteGuard<'static, HashMap<String, Arc<dyn Skill>>> {
    SKILL_REGISTRY.write().unwrap()
}

pub fn get_skill(name: &str) -> Option<Arc<dyn Skill>> {
    get_registry().get(name).cloned()
}

pub fn register_skill(name: String, skill: Arc<dyn Skill>) {
    get_registry_mut().insert(name, skill);
}

pub fn has_skill(name: &str) -> bool {
    get_registry().contains_key(name)
}

pub fn list_skills() -> Vec<String> {
    get_registry().keys().cloned().collect()
}

#[cfg(test)]
mod registry_test {
    use super::*;

    #[test]
    fn test_generate_ai_registry() {
        let metadata = generate_ai_registry();
        println!("Total skills: {}", metadata.len());
        for skill in &metadata {
            println!(
                "  - {} ({}): {}",
                skill.name, skill.category, skill.description
            );
        }
    }

    #[test]
    fn test_print_all_skill_json() {
        let json_str = generate_skill_registry_table_json();
        println!("{:?}", json_str);
    }

    #[test]
    fn test_print_all_skill_json_str() {
        let json_str = generate_skill_registry_table_json_str();
        println!("{:?}", json_str);
    }
}
