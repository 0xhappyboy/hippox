//! Terminal commands drivers registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Terminal;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "terminal_commands", feature = "all"))]
    {
        use crate::drivers::ExecCommandDriver;

        map.insert("exec_command".to_string(), Arc::new(ExecCommandDriver));
    }
}
