//! Email drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Email;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "email", feature = "all"))]
    {
        use crate::drivers::email::*;
        map.insert("send_email".to_string(), Arc::new(SendEmailDriver));
    }
}
