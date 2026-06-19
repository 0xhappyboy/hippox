//! Basic drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Basic;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "helloworld", feature = "all"))]
    {
        use crate::helloworld::HelloWorldDriver;

        map.insert("helloworld".to_string(), Arc::new(HelloWorldDriver));
    }
}
