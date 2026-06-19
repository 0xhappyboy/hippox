//! Time drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Time;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "time", feature = "all"))]
    {
        use crate::drivers::time::*;
        map.insert("datetime".to_string(), Arc::new(DateTimeDriver));
        map.insert("os_get_time".to_string(), Arc::new(OsGetTimeDriver));
        map.insert("os_set_time".to_string(), Arc::new(OsSetTimeDriver));
    }
}
