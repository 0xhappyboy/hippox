//! Database drivers registration

use std::collections::HashMap;
use std::sync::Arc;

use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Database;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "database", feature = "all"))]
    {
        use crate::drivers::mysql::*;
        use crate::drivers::postgresql::*;
        use crate::drivers::redis::*;
        use crate::drivers::sqlite::*;

        map.insert("postgres_query".to_string(), Arc::new(PostgresQueryDriver));
        map.insert(
            "postgres_execute".to_string(),
            Arc::new(PostgresExecuteDriver),
        );
        map.insert(
            "postgres_list_tables".to_string(),
            Arc::new(PostgresListTablesDriver),
        );
        map.insert("mysql_query".to_string(), Arc::new(MysqlQueryDriver));
        map.insert("mysql_execute".to_string(), Arc::new(MysqlExecuteDriver));
        map.insert(
            "mysql_list_tables".to_string(),
            Arc::new(MysqlListTablesDriver),
        );
        map.insert("redis_set".to_string(), Arc::new(RedisSetDriver));
        map.insert("redis_get".to_string(), Arc::new(RedisGetDriver));
        map.insert("redis_del".to_string(), Arc::new(RedisDelDriver));
        map.insert("redis_keys".to_string(), Arc::new(RedisKeysDriver));
        map.insert("redis_hset".to_string(), Arc::new(RedisHSetDriver));
        map.insert("redis_hget".to_string(), Arc::new(RedisHGetDriver));
        map.insert("sqlite_query".to_string(), Arc::new(SqliteQueryDriver));
        map.insert("sqlite_execute".to_string(), Arc::new(SqliteExecuteDriver));
        map.insert(
            "sqlite_list_tables".to_string(),
            Arc::new(SqliteListTablesDriver),
        );
    }
}
