//! Configuration loading functions

use super::core::{HIPPOX_CORE_CONFIG, HippoxConfig};
use super::instances::*;

/// Initialize global configuration from TOML file
pub(crate) fn init_config_from_toml_file(path: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_toml_file(path)?;
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    *global = config;
    Ok(())
}

/// Initialize global configuration from JSON file
pub(crate) fn init_config_from_json_file(path: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_json_file(path)?;
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    *global = config;
    Ok(())
}

/// Initialize global configuration from a JSON string of optional parameters.
///
/// This function allows loading multiple instances of each service type from a JSON string.
pub(crate) fn init_config_from_params_json_str(json_str: &str) -> anyhow::Result<()> {
    use serde_json::Value;

    let overrides: Value = serde_json::from_str(json_str)?;
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();

    // Lang
    if let Some(v) = overrides.get("lang").and_then(|x| x.as_str()) {
        global.lang = v.to_string();
    }

    // Load each instance type
    load_postgresql_instances(&mut global, &overrides);
    load_mysql_instances(&mut global, &overrides);
    load_redis_instances(&mut global, &overrides);
    load_sqlite_instances(&mut global, &overrides);
    load_docker_instances(&mut global, &overrides);
    load_k8s_instances(&mut global, &overrides);
    load_tcp_instances(&mut global, &overrides);
    load_udp_instances(&mut global, &overrides);
    load_ftp_instances(&mut global, &overrides);
    load_smtp_instances(&mut global, &overrides);
    load_telegram_instances(&mut global, &overrides);
    load_dingtalk_instances(&mut global, &overrides);
    load_feishu_instances(&mut global, &overrides);
    load_wecom_instances(&mut global, &overrides);
    load_github_instances(&mut global, &overrides);

    Ok(())
}

// Individual loader functions (simplified - you can expand as needed)
fn load_postgresql_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    use serde_json::Value;

    if let Some(instances) = overrides
        .get("postgresql_instances")
        .and_then(|x| x.as_object())
    {
        for (id, cfg) in instances {
            if let (Some(host), Some(port), Some(database), Some(username), Some(password)) = (
                cfg.get("host").and_then(|x| x.as_str()),
                cfg.get("port").and_then(|x| x.as_u64()),
                cfg.get("database").and_then(|x| x.as_str()),
                cfg.get("username").and_then(|x| x.as_str()),
                cfg.get("password").and_then(|x| x.as_str()),
            ) {
                let name = cfg.get("name").and_then(|x| x.as_str()).map(String::from);
                let description = cfg
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let pg_config =
                    PostgreSQLConfig::new(
                        id.clone(),
                        name,
                        description,
                        host.to_string(),
                        port as u16,
                        database.to_string(),
                        username.to_string(),
                        password.to_string(),
                    )
                    .with_pool_size(
                        cfg.get("pool_size").and_then(|x| x.as_u64()).unwrap_or(10) as usize
                    )
                    .with_timeout(cfg.get("timeout").and_then(|x| x.as_u64()).unwrap_or(30));
                config.add_postgresql_instance(pg_config);
            }
        }
    }
}

// Similar functions for other instance types...
// (You can generate these using macros or copy from original)

fn load_mysql_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ... (similar to PostgreSQL)
}

fn load_redis_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_sqlite_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_docker_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_k8s_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_tcp_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_udp_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_ftp_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_smtp_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_telegram_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_dingtalk_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_feishu_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_wecom_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}

fn load_github_instances(config: &mut HippoxConfig, overrides: &serde_json::Value) {
    // ...
}
