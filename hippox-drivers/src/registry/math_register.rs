//! Mathematics drivers registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Math;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "math", feature = "all"))]
    {
        use crate::drivers::*;
        use crate::drivers::math::*;
        
        map.insert("math_calculator".to_string(), Arc::new(CalculatorDriver));
        map.insert("math_power".to_string(), Arc::new(PowerDriver));
        map.insert("math_statistics".to_string(), Arc::new(StatisticsDriver));
        map.insert("math_unit_converter".to_string(), Arc::new(UnitConverterDriver));
        map.insert("hash_md5".to_string(), Arc::new(HashMd5Driver));
        map.insert("hash_sha256".to_string(), Arc::new(HashSha256Driver));
        map.insert("hash_sha512".to_string(), Arc::new(HashSha512Driver));
        map.insert("hash_file".to_string(), Arc::new(HashFileDriver));
        map.insert("base64_encode".to_string(), Arc::new(Base64EncodeDriver));
        map.insert("base64_decode".to_string(), Arc::new(Base64DecodeDriver));
        map.insert("random_number".to_string(), Arc::new(RandomNumberDriver));
        map.insert("random_string".to_string(), Arc::new(RandomStringDriver));
        map.insert("random_uuid".to_string(), Arc::new(RandomUuidDriver));
        map.insert("random_password".to_string(), Arc::new(RandomPasswordDriver));
    }
}