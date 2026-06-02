//! Global CRUD functions for instance management

use super::core::HIPPOX_CORE_CONFIG;
use super::instances::*;

pub fn add_postgresql_instance(config: PostgreSQLConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_postgresql_instance(config)
}

pub fn get_postgresql_instance(id: &str) -> Option<PostgreSQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_postgresql_instance(id).cloned()
}

pub fn remove_postgresql_instance(id: &str) -> Option<PostgreSQLConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_postgresql_instance(id)
}

pub fn list_postgresql_instances() -> Vec<PostgreSQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_postgresql_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_mysql_instance(config: MySQLConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_mysql_instance(config)
}

pub fn get_mysql_instance(id: &str) -> Option<MySQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_mysql_instance(id).cloned()
}

pub fn remove_mysql_instance(id: &str) -> Option<MySQLConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_mysql_instance(id)
}

pub fn list_mysql_instances() -> Vec<MySQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_mysql_instances().into_iter().cloned().collect()
}

pub fn add_redis_instance(config: RedisConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_redis_instance(config)
}

pub fn get_redis_instance(id: &str) -> Option<RedisConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_redis_instance(id).cloned()
}

pub fn remove_redis_instance(id: &str) -> Option<RedisConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_redis_instance(id)
}

pub fn list_redis_instances() -> Vec<RedisConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_redis_instances().into_iter().cloned().collect()
}

pub fn add_sqlite_instance(config: SQLiteConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_sqlite_instance(config)
}

pub fn get_sqlite_instance(id: &str) -> Option<SQLiteConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_sqlite_instance(id).cloned()
}

pub fn remove_sqlite_instance(id: &str) -> Option<SQLiteConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_sqlite_instance(id)
}

pub fn list_sqlite_instances() -> Vec<SQLiteConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_sqlite_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_docker_instance(config: DockerConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_docker_instance(config)
}

pub fn get_docker_instance(id: &str) -> Option<DockerConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_docker_instance(id).cloned()
}

pub fn remove_docker_instance(id: &str) -> Option<DockerConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_docker_instance(id)
}

pub fn list_docker_instances() -> Vec<DockerConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_docker_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_k8s_instance(config: K8sConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_k8s_instance(config)
}

pub fn get_k8s_instance(id: &str) -> Option<K8sConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_k8s_instance(id).cloned()
}

pub fn remove_k8s_instance(id: &str) -> Option<K8sConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_k8s_instance(id)
}

pub fn list_k8s_instances() -> Vec<K8sConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_k8s_instances().into_iter().cloned().collect()
}

pub fn add_tcp_instance(config: TCPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_tcp_instance(config)
}

pub fn get_tcp_instance(id: &str) -> Option<TCPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_tcp_instance(id).cloned()
}

pub fn remove_tcp_instance(id: &str) -> Option<TCPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_tcp_instance(id)
}

pub fn list_tcp_instances() -> Vec<TCPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_tcp_instances().into_iter().cloned().collect()
}

pub fn add_udp_instance(config: UDPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_udp_instance(config)
}

pub fn get_udp_instance(id: &str) -> Option<UDPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_udp_instance(id).cloned()
}

pub fn remove_udp_instance(id: &str) -> Option<UDPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_udp_instance(id)
}

pub fn list_udp_instances() -> Vec<UDPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_udp_instances().into_iter().cloned().collect()
}

pub fn add_ftp_instance(config: FTPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_ftp_instance(config)
}

pub fn get_ftp_instance(id: &str) -> Option<FTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_ftp_instance(id).cloned()
}

pub fn remove_ftp_instance(id: &str) -> Option<FTPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_ftp_instance(id)
}

pub fn list_ftp_instances() -> Vec<FTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_ftp_instances().into_iter().cloned().collect()
}

pub fn add_smtp_instance(config: SMTPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_smtp_instance(config)
}

pub fn get_smtp_instance(id: &str) -> Option<SMTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_smtp_instance(id).cloned()
}

pub fn remove_smtp_instance(id: &str) -> Option<SMTPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_smtp_instance(id)
}

pub fn list_smtp_instances() -> Vec<SMTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_smtp_instances().into_iter().cloned().collect()
}

pub fn add_telegram_instance(config: TelegramConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_telegram_instance(config)
}

pub fn get_telegram_instance(id: &str) -> Option<TelegramConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_telegram_instance(id).cloned()
}

pub fn remove_telegram_instance(id: &str) -> Option<TelegramConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_telegram_instance(id)
}

pub fn list_telegram_instances() -> Vec<TelegramConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_telegram_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_dingtalk_instance(config: DingTalkConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_dingtalk_instance(config)
}

pub fn get_dingtalk_instance(id: &str) -> Option<DingTalkConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_dingtalk_instance(id).cloned()
}

pub fn remove_dingtalk_instance(id: &str) -> Option<DingTalkConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_dingtalk_instance(id)
}

pub fn list_dingtalk_instances() -> Vec<DingTalkConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_dingtalk_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_feishu_instance(config: FeishuConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_feishu_instance(config)
}

pub fn get_feishu_instance(id: &str) -> Option<FeishuConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_feishu_instance(id).cloned()
}

pub fn remove_feishu_instance(id: &str) -> Option<FeishuConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_feishu_instance(id)
}

pub fn list_feishu_instances() -> Vec<FeishuConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_feishu_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_wecom_instance(config: WeComConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_wecom_instance(config)
}

pub fn get_wecom_instance(id: &str) -> Option<WeComConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_wecom_instance(id).cloned()
}

pub fn remove_wecom_instance(id: &str) -> Option<WeComConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_wecom_instance(id)
}

pub fn list_wecom_instances() -> Vec<WeComConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_wecom_instances().into_iter().cloned().collect()
}

pub fn add_github_instance(config: GitHubConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.add_github_instance(config)
}

pub fn get_github_instance(id: &str) -> Option<GitHubConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_github_instance(id).cloned()
}

pub fn remove_github_instance(id: &str) -> Option<GitHubConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    global.remove_github_instance(id)
}

pub fn list_github_instances() -> Vec<GitHubConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_github_instances()
        .into_iter()
        .cloned()
        .collect()
}
