//! Database skills registration

use std::collections::HashMap;
use std::sync::Arc;

use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Db;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "db", feature = "all"))]
    {
        use crate::skills::mysql::*;
        use crate::skills::postgresql::*;
        use crate::skills::redis::*;
        use crate::skills::sqlite::*;

        map.insert("postgres_query".to_string(), Arc::new(PostgresQuerySkill));
        map.insert(
            "postgres_execute".to_string(),
            Arc::new(PostgresExecuteSkill),
        );
        map.insert(
            "postgres_list_tables".to_string(),
            Arc::new(PostgresListTablesSkill),
        );
        map.insert("mysql_query".to_string(), Arc::new(MysqlQuerySkill));
        map.insert("mysql_execute".to_string(), Arc::new(MysqlExecuteSkill));
        map.insert(
            "mysql_list_tables".to_string(),
            Arc::new(MysqlListTablesSkill),
        );
        map.insert("redis_set".to_string(), Arc::new(RedisSetSkill));
        map.insert("redis_get".to_string(), Arc::new(RedisGetSkill));
        map.insert("redis_del".to_string(), Arc::new(RedisDelSkill));
        map.insert("redis_keys".to_string(), Arc::new(RedisKeysSkill));
        map.insert("redis_hset".to_string(), Arc::new(RedisHSetSkill));
        map.insert("redis_hget".to_string(), Arc::new(RedisHGetSkill));
        map.insert("sqlite_query".to_string(), Arc::new(SqliteQuerySkill));
        map.insert("sqlite_execute".to_string(), Arc::new(SqliteExecuteSkill));
        map.insert(
            "sqlite_list_tables".to_string(),
            Arc::new(SqliteListTablesSkill),
        );
    }
}
