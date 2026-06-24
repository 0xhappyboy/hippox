//! Finance drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Finance;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "finance", feature = "all"))]
    {
        use crate::drivers::finance::*;
        use crate::drivers::*;

        map.insert(
            "finance_ohlcv_generator".to_string(),
            Arc::new(OhlcvGeneratorDriver),
        );
    }
}
